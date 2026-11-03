//! L'état d'une partie et la fonction centrale `apply`.
//!
//! Tout se passe en mémoire, sans réseau, sans base et sans horloge : l'API charge la partie,
//! appelle `apply`, puis la sauvegarde. Une opération refusée renvoie une erreur et ne modifie
//! rien (on vérifie tout AVANT de changer l'état).

use serde::{Deserialize, Serialize};

use crate::cards::{CardValue, DECK_SIZE, full_deck};
use crate::error::GameError;
use crate::grid::{CARDS_PER_PLAYER, Grid};
use crate::rng::SplitMix64;
use crate::rules::{EmptyPile, EndTie, FirstPlayerTie, NextRoundStarter, Rules};
use crate::scoring::finisher_round_score;

/// Nombre minimum de joueurs pour lancer une partie.
pub const MIN_PLAYERS: usize = 2;
/// Nombre maximum de joueurs. Le reste du code ne suppose aucun nombre de joueurs.
pub const MAX_PLAYERS: usize = 10;
/// Nombre de tours manqués d'affilée (joueur absent) qui entraîne l'élimination.
pub const MAX_MISSED_TURNS: u32 = 2;
/// Longueur maximale d'un nom de joueur (en caractères, après suppression des espaces autour).
pub const MAX_NAME_CHARS: usize = 20;
/// Nombre de cartes que chaque joueur retourne en début de manche.
pub const INITIAL_REVEALS: usize = 2;
/// L'hôte est toujours le premier joueur (le créateur de la partie).
pub const HOST: PlayerId = 0;

/// Identifiant d'un joueur : sa place dans l'ordre d'arrivée (0 = l'hôte).
pub type PlayerId = usize;

/// Un joueur et sa grille de la manche en cours.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub grid: Grid,
    pub total_score: i32,
    /// Cartes retournées pendant la mise en place de la manche en cours (0 à 2).
    pub initial_reveals: usize,
    /// Tours passés d'affilée parce que le joueur était absent (remis à zéro dès qu'il joue).
    pub missed_turns: u32,
    /// Éliminé après `MAX_MISSED_TURNS` tours manqués : il ne joue plus et ne compte plus.
    pub eliminated: bool,
}

/// Résultat d'un tour passé pour cause d'absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipOutcome {
    /// Le tour est passé ; le joueur reste dans la partie.
    Skipped,
    /// C'était un tour manqué de trop : le joueur est éliminé.
    Eliminated,
}

/// D'où vient la carte en main ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeldSource {
    Pile,
    Discard,
}

/// Carte que le joueur courant a en main (entre « piocher » et « poser »).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    pub value: CardValue,
    pub source: HeldSource,
}

/// État du tour en cours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub current: PlayerId,
    pub held: Option<Held>,
    /// Joueur qui a retourné toutes ses cartes : les autres jouent leur dernier tour.
    pub finisher: Option<PlayerId>,
}

/// Phase de la partie.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Salon : les joueurs arrivent, l'hôte règle la partie.
    Lobby,
    /// Début de manche : chaque joueur retourne 2 cartes, sans ordre de tour.
    InitialReveal,
    /// Manche en cours.
    Playing(Turn),
    /// Manche terminée et comptée ; on attend « manche suivante ».
    RoundOver,
    /// Partie terminée (voir `Game::winners`).
    GameOver,
}

/// Une intention de joueur pendant la partie. Le navigateur envoie une `Action`, jamais un état.
/// `#[serde(tag = "type")]` : en JSON, `{ "type": "SwapHeld", "column": 0, "row": 1 }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Action {
    RevealInitial { column: usize, row: usize },
    DrawFromPile,
    TakeDiscard,
    SwapHeld { column: usize, row: usize },
    DiscardHeldAndReveal { column: usize, row: usize },
    NextRound,
}

/// Score d'un joueur pour une manche.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRoundScore {
    pub player: PlayerId,
    /// Somme des cartes restantes.
    pub raw: i32,
    /// Score retenu (après pénalité éventuelle).
    pub score: i32,
    pub penalized: bool,
}

/// Résultat d'une manche terminée.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: u32,
    pub finisher: Option<PlayerId>,
    pub scores: Vec<PlayerRoundScore>,
}

/// L'état complet d'une partie, cartes cachées comprises. Il ne sort jamais du serveur tel
/// quel : toute réponse passe par `view_for`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub rules: Rules,
    pub players: Vec<Player>,
    pub phase: Phase,
    /// Pioche, face cachée. Le dessus est le DERNIER élément.
    pub draw_pile: Vec<CardValue>,
    /// Défausse, face visible. Le dessus est le DERNIER élément.
    pub discard: Vec<CardValue>,
    /// Numéro de la manche en cours (0 au salon).
    pub round: u32,
    /// Joueur qui a commencé la manche en cours.
    pub round_starter: Option<PlayerId>,
    /// Joueur désigné pour commencer la manche suivante (décidé à la fin de la manche).
    /// `None` : on applique la règle de la première manche (plus grande somme).
    pub next_starter: Option<PlayerId>,
    pub rounds: Vec<RoundResult>,
    pub winners: Vec<PlayerId>,
    pub rng: SplitMix64,
}

/// Fonction centrale demandée par l'architecture : applique l'action d'un joueur.
pub fn apply(game: &mut Game, player: PlayerId, action: Action) -> Result<(), GameError> {
    game.apply(player, action)
}

/// Vérifie et nettoie un nom de joueur.
fn clean_name(name: &str) -> Result<String, GameError> {
    let name = name.trim();
    let length = name.chars().count();
    if length == 0 || length > MAX_NAME_CHARS {
        return Err(GameError::InvalidName);
    }
    Ok(name.to_string())
}

impl Game {
    /// Crée une partie au salon, avec son hôte. `seed` initialise le générateur (en production,
    /// l'API la tire de l'entropie du système).
    pub fn new(host_name: &str, rules: Rules, seed: u64) -> Result<Game, GameError> {
        rules.validate()?;
        let host = Player {
            name: clean_name(host_name)?,
            grid: Grid::default(),
            total_score: 0,
            initial_reveals: 0,
            missed_turns: 0,
            eliminated: false,
        };
        Ok(Game {
            rules,
            players: vec![host],
            phase: Phase::Lobby,
            draw_pile: Vec::new(),
            discard: Vec::new(),
            round: 0,
            round_starter: None,
            next_starter: None,
            rounds: Vec::new(),
            winners: Vec::new(),
            rng: SplitMix64::new(seed),
        })
    }

    /// Ajoute un joueur au salon et renvoie son identifiant.
    pub fn join(&mut self, name: &str) -> Result<PlayerId, GameError> {
        if self.phase != Phase::Lobby {
            return Err(GameError::WrongPhase);
        }
        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::GameFull);
        }
        let name = clean_name(name)?;
        self.players.push(Player {
            name,
            grid: Grid::default(),
            total_score: 0,
            initial_reveals: 0,
            missed_turns: 0,
            eliminated: false,
        });
        Ok(self.players.len() - 1)
    }

    /// Remplace les réglages (hôte seulement, au salon seulement).
    pub fn update_rules(&mut self, by: PlayerId, rules: Rules) -> Result<(), GameError> {
        self.check_player(by)?;
        if by != HOST {
            return Err(GameError::NotHost);
        }
        if self.phase != Phase::Lobby {
            return Err(GameError::WrongPhase);
        }
        rules.validate()?;
        self.rules = rules;
        Ok(())
    }

    /// Lance la partie (hôte seulement) : distribue la première manche.
    pub fn start(&mut self, by: PlayerId) -> Result<(), GameError> {
        self.check_player(by)?;
        if by != HOST {
            return Err(GameError::NotHost);
        }
        if self.phase != Phase::Lobby {
            return Err(GameError::WrongPhase);
        }
        if self.players.len() < MIN_PLAYERS {
            return Err(GameError::NotEnoughPlayers);
        }
        self.start_round()
    }

    /// Applique une action de jeu.
    pub fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        self.check_player(player)?;
        if self.players[player].eliminated {
            return Err(GameError::Eliminated);
        }
        self.apply_action(player, action)?;
        // Le joueur est là et vient de jouer : ses tours manqués ne comptent plus.
        self.players[player].missed_turns = 0;
        Ok(())
    }

    /// Passe le tour d'un joueur absent (décidé par l'API, qui seule connaît l'heure).
    /// - Pendant la mise en place : ses cartes restantes à retourner le sont à sa place.
    /// - Pendant la manche : une carte qu'il tenait va sur la défausse, puis le tour passe.
    ///
    /// Au `MAX_MISSED_TURNS`-ième tour manqué d'affilée, il est éliminé (ASSUMPTIONS.md, n° 18 à 21).
    pub fn skip_absent(&mut self, player: PlayerId) -> Result<SkipOutcome, GameError> {
        self.check_player(player)?;
        if self.players[player].eliminated {
            return Err(GameError::Eliminated);
        }
        match &self.phase {
            Phase::InitialReveal => {
                if self.players[player].initial_reveals >= INITIAL_REVEALS {
                    return Err(GameError::InitialRevealsDone);
                }
                // Tour manqué de trop : il est éliminé AVANT de retourner quoi que ce soit, pour ne
                // jamais être désigné pour commencer la manche.
                if self.players[player].missed_turns + 1 >= MAX_MISSED_TURNS {
                    return Ok(self.count_missed_turn(player));
                }
                while self.players[player].initial_reveals < INITIAL_REVEALS {
                    let Some((column, row)) = self.players[player].grid.first_hidden() else {
                        break;
                    };
                    self.reveal_initial(player, column, row)?;
                }
                // Une grille sans carte cachée (cas extrême) : on considère la mise en place faite.
                self.players[player].initial_reveals = INITIAL_REVEALS;
                Ok(self.count_missed_turn(player))
            }
            Phase::Playing(turn) if turn.current == player => {
                let mut turn = *turn;
                if let Some(held) = turn.held.take() {
                    self.discard.push(held.value);
                }
                let outcome = self.count_missed_turn(player);
                if self.phase != Phase::GameOver {
                    self.pass_turn(turn);
                }
                Ok(outcome)
            }
            Phase::Playing(_) => Err(GameError::NotYourTurn),
            _ => Err(GameError::WrongPhase),
        }
    }

    /// Joueurs encore en jeu (non éliminés), dans l'ordre d'arrivée.
    pub fn active_players(&self) -> Vec<PlayerId> {
        (0..self.players.len())
            .filter(|&p| !self.players[p].eliminated)
            .collect()
    }

    fn apply_action(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        match action {
            Action::RevealInitial { column, row } => self.reveal_initial(player, column, row),
            Action::NextRound => {
                if self.phase != Phase::RoundOver {
                    return Err(GameError::WrongPhase);
                }
                self.start_round()
            }
            Action::DrawFromPile => {
                let mut turn = self.turn_of(player)?;
                if turn.held.is_some() {
                    return Err(GameError::AlreadyHolding);
                }
                let value = self.draw_pile.pop().ok_or(GameError::EmptyPile)?;
                turn.held = Some(Held {
                    value,
                    source: HeldSource::Pile,
                });
                self.phase = Phase::Playing(turn);
                Ok(())
            }
            Action::TakeDiscard => {
                let mut turn = self.turn_of(player)?;
                if turn.held.is_some() {
                    return Err(GameError::AlreadyHolding);
                }
                let value = self.discard.pop().ok_or(GameError::EmptyDiscard)?;
                turn.held = Some(Held {
                    value,
                    source: HeldSource::Discard,
                });
                self.phase = Phase::Playing(turn);
                Ok(())
            }
            Action::SwapHeld { column, row } => {
                let turn = self.turn_of(player)?;
                let held = turn.held.ok_or(GameError::NothingHeld)?;
                let cell = self.players[player].grid.cell_mut(column, row)?;
                // La carte remplacée part face visible sur la défausse, même si elle était cachée.
                let replaced = cell.value;
                cell.value = held.value;
                cell.face_up = true;
                self.discard.push(replaced);
                self.end_move(turn)
            }
            Action::DiscardHeldAndReveal { column, row } => {
                let turn = self.turn_of(player)?;
                let held = turn.held.ok_or(GameError::NothingHeld)?;
                // Règle : une carte prise sur la défausse doit être échangée.
                if held.source == HeldSource::Discard {
                    return Err(GameError::MustSwapDiscard);
                }
                let cell = self.players[player].grid.cell_mut(column, row)?;
                if cell.face_up {
                    return Err(GameError::AlreadyRevealed);
                }
                cell.face_up = true;
                self.discard.push(held.value);
                self.end_move(turn)
            }
        }
    }

    /// Nombre total de cartes (pioche, défausse, grilles, carte en main). Les cartes des
    /// colonnes retirées sont sur la défausse. Vaut toujours 150 une fois la partie lancée.
    pub fn card_count(&self) -> usize {
        let in_grids: usize = self.players.iter().map(|p| p.grid.card_count()).sum();
        let held = match &self.phase {
            Phase::Playing(Turn { held: Some(_), .. }) => 1,
            _ => 0,
        };
        self.draw_pile.len() + self.discard.len() + in_grids + held
    }

    /// Joueur dont c'est le tour (pendant la phase de jeu).
    pub fn current_player(&self) -> Option<PlayerId> {
        match &self.phase {
            Phase::Playing(turn) => Some(turn.current),
            _ => None,
        }
    }

    /// Toutes les actions légales d'un joueur dans l'état actuel (utile aux tests et pour
    /// guider l'interface).
    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        if player >= self.players.len() || self.players[player].eliminated {
            return Vec::new();
        }
        let grid = &self.players[player].grid;
        let positions = grid.positions();
        let hidden: Vec<(usize, usize)> = positions
            .iter()
            .copied()
            .filter(|&(c, r)| matches!(grid.cell(c, r), Ok(cell) if !cell.face_up))
            .collect();
        match &self.phase {
            Phase::InitialReveal if self.players[player].initial_reveals < INITIAL_REVEALS => {
                hidden
                    .into_iter()
                    .map(|(column, row)| Action::RevealInitial { column, row })
                    .collect()
            }
            Phase::RoundOver => vec![Action::NextRound],
            Phase::Playing(turn) if turn.current == player => match turn.held {
                None => {
                    let mut actions = Vec::new();
                    if !self.draw_pile.is_empty() {
                        actions.push(Action::DrawFromPile);
                    }
                    if !self.discard.is_empty() {
                        actions.push(Action::TakeDiscard);
                    }
                    actions
                }
                Some(held) => {
                    let mut actions: Vec<Action> = positions
                        .iter()
                        .map(|&(column, row)| Action::SwapHeld { column, row })
                        .collect();
                    if held.source == HeldSource::Pile {
                        actions.extend(
                            hidden
                                .into_iter()
                                .map(|(column, row)| Action::DiscardHeldAndReveal { column, row }),
                        );
                    }
                    actions
                }
            },
            _ => Vec::new(),
        }
    }

    // ----- Fonctions internes -----

    fn check_player(&self, player: PlayerId) -> Result<(), GameError> {
        if player < self.players.len() {
            Ok(())
        } else {
            Err(GameError::UnknownPlayer)
        }
    }

    /// Renvoie une copie du tour si c'est bien au tour de `player`.
    fn turn_of(&self, player: PlayerId) -> Result<Turn, GameError> {
        match &self.phase {
            Phase::Playing(turn) if turn.current == player => Ok(*turn),
            Phase::Playing(_) => Err(GameError::NotYourTurn),
            _ => Err(GameError::WrongPhase),
        }
    }

    /// Distribue une nouvelle manche : paquet complet mélangé, 12 cartes par joueur,
    /// une carte sur la défausse, le reste en pioche.
    fn start_round(&mut self) -> Result<(), GameError> {
        let needed = self.active_players().len() * CARDS_PER_PLAYER + 1;
        if needed > DECK_SIZE {
            return Err(GameError::NotEnoughCards);
        }
        let mut deck = full_deck();
        self.rng.shuffle(&mut deck);
        for player in &mut self.players {
            player.initial_reveals = 0;
            if player.eliminated {
                // Un joueur éliminé ne reçoit plus de cartes.
                player.grid = Grid::default();
                continue;
            }
            // `split_off` coupe le paquet : les 12 dernières cartes partent dans la grille.
            let hand = deck.split_off(deck.len() - CARDS_PER_PLAYER);
            player.grid = Grid::from_cards(hand);
        }
        self.discard = deck.pop().into_iter().collect();
        self.draw_pile = deck;
        self.round += 1;
        self.phase = Phase::InitialReveal;
        Ok(())
    }

    fn reveal_initial(
        &mut self,
        player: PlayerId,
        column: usize,
        row: usize,
    ) -> Result<(), GameError> {
        if self.phase != Phase::InitialReveal {
            return Err(GameError::WrongPhase);
        }
        if self.players[player].initial_reveals >= INITIAL_REVEALS {
            return Err(GameError::InitialRevealsDone);
        }
        let cell = self.players[player].grid.cell_mut(column, row)?;
        if cell.face_up {
            return Err(GameError::AlreadyRevealed);
        }
        cell.face_up = true;
        self.players[player].initial_reveals += 1;
        // Par sécurité (la règle s'applique dès que 3 cartes identiques sont visibles).
        let removed = self.players[player].grid.remove_identical_columns();
        self.discard.extend(removed);

        self.finish_initial_reveal_if_done();
        Ok(())
    }

    /// Quand tous les joueurs en jeu ont retourné leurs 2 cartes, la manche commence.
    fn finish_initial_reveal_if_done(&mut self) {
        let everyone_done = self
            .players
            .iter()
            .filter(|p| !p.eliminated)
            .all(|p| p.initial_reveals >= INITIAL_REVEALS);
        if everyone_done {
            let starter = match self.next_starter {
                // Un joueur désigné entre-temps éliminé ne commence pas.
                Some(p) if !self.players[p].eliminated => p,
                _ => self.highest_visible_sum_player(),
            };
            self.round_starter = Some(starter);
            self.phase = Phase::Playing(Turn {
                current: starter,
                held: None,
                finisher: None,
            });
        }
    }

    /// Règle de la première manche : la plus grande somme de cartes visibles commence ;
    /// égalité départagée par `first_player_tie`.
    fn highest_visible_sum_player(&mut self) -> PlayerId {
        let active = self.active_players();
        let sum = |p: PlayerId| self.players[p].grid.visible_sum();
        let best = active.iter().map(|&p| sum(p)).max().unwrap_or(0);
        // Les ex æquo, dans l'ordre d'arrivée.
        let tied: Vec<PlayerId> = active.into_iter().filter(|&p| sum(p) == best).collect();
        match self.rules.first_player_tie {
            FirstPlayerTie::JoinOrder => tied.first().copied().unwrap_or(HOST),
            FirstPlayerTie::Random => {
                if tied.len() <= 1 {
                    tied.first().copied().unwrap_or(HOST)
                } else {
                    tied[self.rng.below(tied.len())]
                }
            }
        }
    }

    /// Fin d'un coup (échange, ou défausse + retournement) : colonnes identiques, finisseur,
    /// pioche épuisée, puis passage au joueur suivant ou fin de manche.
    fn end_move(&mut self, mut turn: Turn) -> Result<(), GameError> {
        let player = turn.current;
        turn.held = None;

        let removed = self.players[player].grid.remove_identical_columns();
        self.discard.extend(removed);

        // Le premier joueur dont toutes les cartes sont visibles termine la manche.
        if turn.finisher.is_none() && self.players[player].grid.hidden_count() == 0 {
            turn.finisher = Some(player);
        }

        // Pioche épuisée (réglage `empty_pile`). Avec une seule carte sur la défausse, il n'y a
        // rien à remélanger : la manche se termine comme pour `EndRound` (ASSUMPTIONS.md).
        if self.draw_pile.is_empty() {
            let can_reshuffle =
                self.rules.empty_pile == EmptyPile::ReshuffleDiscard && self.discard.len() >= 2;
            if can_reshuffle {
                self.reshuffle_discard();
            } else {
                self.end_round(turn.finisher);
                return Ok(());
            }
        }

        self.pass_turn(turn);
        Ok(())
    }

    /// Joueur suivant encore en jeu, après `player` dans l'ordre du tour.
    fn next_active(&self, player: PlayerId) -> PlayerId {
        let n = self.players.len();
        (1..=n)
            .map(|step| (player + step) % n)
            .find(|&p| !self.players[p].eliminated)
            .unwrap_or(player)
    }

    /// Donne le tour au joueur suivant, ou termine la manche si le tour revient au finisseur
    /// (tous les autres joueurs en jeu ont alors joué leur dernier tour).
    fn pass_turn(&mut self, mut turn: Turn) {
        let player = turn.current;
        let next = self.next_active(player);
        // Le finisseur est-il atteint (ou dépassé) en allant de `player` à `next` ?
        let n = self.players.len();
        let reaches_finisher = turn.finisher.is_some_and(|f| {
            (1..=n)
                .map(|step| (player + step) % n)
                .take_while(|&p| p != next)
                .chain(std::iter::once(next))
                .any(|p| p == f)
        });
        if reaches_finisher {
            self.end_round(turn.finisher);
        } else {
            turn.held = None;
            turn.current = next;
            self.phase = Phase::Playing(turn);
        }
    }

    /// Compte un tour manqué ; élimine le joueur au-delà de la limite.
    fn count_missed_turn(&mut self, player: PlayerId) -> SkipOutcome {
        self.players[player].missed_turns += 1;
        if self.players[player].missed_turns < MAX_MISSED_TURNS {
            if self.phase == Phase::InitialReveal {
                self.finish_initial_reveal_if_done();
            }
            return SkipOutcome::Skipped;
        }
        self.players[player].eliminated = true;
        let active = self.active_players();
        if active.len() <= 1 {
            // Dernier joueur en jeu : il gagne.
            self.winners = active;
            self.phase = Phase::GameOver;
        } else if self.phase == Phase::InitialReveal {
            self.finish_initial_reveal_if_done();
        }
        SkipOutcome::Eliminated
    }

    /// Remélange la défausse, sauf sa carte du dessus qui reste visible, pour former la pioche.
    fn reshuffle_discard(&mut self) {
        let top = self.discard.pop();
        let mut pile = std::mem::take(&mut self.discard);
        self.rng.shuffle(&mut pile);
        self.draw_pile = pile;
        self.discard = top.into_iter().collect();
    }

    /// Termine la manche : tout est retourné, les scores sont comptés, puis on décide de la
    /// suite (manche suivante ou fin de partie).
    fn end_round(&mut self, finisher: Option<PlayerId>) {
        for player in &mut self.players {
            player.grid.reveal_all();
            // Les cartes sont maintenant visibles : la règle de la colonne identique s'applique
            // aussi à ce moment (lecture littérale de docs/RULES.md, voir ASSUMPTIONS.md).
            let removed = player.grid.remove_identical_columns();
            self.discard.extend(removed);
        }

        // Les joueurs éliminés ne sont plus comptés (ni pour eux, ni pour la pénalité des autres).
        let active = self.active_players();
        let raws: Vec<(PlayerId, i32)> = active
            .iter()
            .map(|&p| (p, self.players[p].grid.values().iter().sum()))
            .collect();
        let mut scores = Vec::with_capacity(raws.len());
        for &(p, raw) in &raws {
            let score = if finisher == Some(p) {
                let others: Vec<i32> = raws
                    .iter()
                    .filter(|&&(q, _)| q != p)
                    .map(|&(_, r)| r)
                    .collect();
                finisher_round_score(&self.players[p].grid.values(), &others, &self.rules)
            } else {
                raw
            };
            self.players[p].total_score += score;
            scores.push(PlayerRoundScore {
                player: p,
                raw,
                score,
                penalized: score != raw,
            });
        }

        self.next_starter = self.choose_next_starter(finisher, &scores);
        self.rounds.push(RoundResult {
            round: self.round,
            finisher,
            scores,
        });

        // Fin de partie : un joueur atteint ou dépasse `end_score`.
        let total = |p: PlayerId| self.players[p].total_score;
        let reached = active.iter().any(|&p| total(p) >= self.rules.end_score);
        if reached {
            let lowest = active.iter().map(|&p| total(p)).min().unwrap_or(0);
            let tied: Vec<PlayerId> = active
                .iter()
                .copied()
                .filter(|&p| total(p) == lowest)
                .collect();
            if tied.len() == 1 || self.rules.end_tie == EndTie::SharedVictory {
                self.winners = tied;
                self.phase = Phase::GameOver;
                return;
            }
            // `ExtraRound` : égalité sur le total le plus bas, on rejoue une manche.
        }
        self.phase = Phase::RoundOver;
    }

    /// Joueur qui commencera la manche suivante (réglage `next_round_starter`).
    fn choose_next_starter(
        &self,
        finisher: Option<PlayerId>,
        scores: &[PlayerRoundScore],
    ) -> Option<PlayerId> {
        match self.rules.next_round_starter {
            // Sans finisseur (pioche épuisée), on revient à la règle de la première manche.
            NextRoundStarter::Finisher => finisher.filter(|&f| !self.players[f].eliminated),
            // `min_by_key` garde le PREMIER minimum : en cas d'égalité, l'ordre d'arrivée.
            NextRoundStarter::RoundWinner => {
                scores.iter().min_by_key(|s| s.score).map(|s| s.player)
            }
            NextRoundStarter::Rotate => self.round_starter.map(|starter| self.next_active(starter)),
        }
    }
}
