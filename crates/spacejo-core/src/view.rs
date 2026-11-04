//! Vue filtrée : ce qu'un joueur a le droit de voir de la partie.
//!
//! C'est la SEULE forme de l'état qui sort du serveur. Une carte face cachée devient
//! `PublicSlot::Hidden`, qui ne contient aucune valeur : il est donc impossible, par
//! construction, de la laisser fuir dans le JSON. La pioche n'est donnée que par son nombre
//! de cartes, et l'état du générateur aléatoire n'apparaît pas.

use serde::Serialize;

use crate::cards::CardValue;
use crate::game::{Game, HOST, HeldSource, Phase, PlayerId, RoundResult};
use crate::grid::Column;
use crate::rules::Rules;

/// Une case de grille vue par un joueur.
/// `#[serde(tag = "kind")]` : `{ "kind": "Hidden" }` ou `{ "kind": "Visible", "value": 7 }`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum PublicSlot {
    Hidden,
    Visible { value: CardValue },
}

/// Une colonne vue par un joueur.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum ColumnView {
    Present { slots: Vec<PublicSlot> },
    Removed,
}

/// Un joueur vu par les autres (et par lui-même).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerView {
    pub id: PlayerId,
    pub name: String,
    pub total_score: i32,
    pub initial_reveals: usize,
    pub missed_turns: u32,
    pub eliminated: bool,
    /// A quitté la partie de lui-même (il est aussi éliminé).
    pub abandoned: bool,
    pub grid: Vec<ColumnView>,
}

/// La carte en main du joueur courant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HeldView {
    pub source: HeldSource,
    /// `None` pour les autres joueurs quand la carte vient de la pioche.
    pub value: Option<CardValue>,
}

/// Nom de la phase, sans les données internes du tour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PhaseName {
    Lobby,
    InitialReveal,
    Playing,
    RoundOver,
    GameOver,
}

/// Vue complète de la partie pour un joueur (format décrit dans docs/CONTRACT.md).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameView {
    pub me: PlayerId,
    pub host: PlayerId,
    pub phase: PhaseName,
    pub round: u32,
    pub rules: Rules,
    pub players: Vec<PlayerView>,
    pub current_player: Option<PlayerId>,
    pub finisher: Option<PlayerId>,
    pub held: Option<HeldView>,
    pub draw_pile_count: usize,
    pub discard_top: Option<CardValue>,
    pub discard_count: usize,
    pub rounds: Vec<RoundResult>,
    pub winners: Vec<PlayerId>,
}

/// Construit la vue de `me`. N'emprunte la partie qu'en lecture (`&Game`) : la vue est une
/// copie filtrée, elle ne peut pas modifier la partie.
pub fn view_for(game: &Game, me: PlayerId) -> GameView {
    let players = game
        .players
        .iter()
        .enumerate()
        .map(|(id, player)| PlayerView {
            id,
            name: player.name.clone(),
            total_score: player.total_score,
            initial_reveals: player.initial_reveals,
            missed_turns: player.missed_turns,
            eliminated: player.eliminated,
            abandoned: player.abandoned,
            grid: player
                .grid
                .columns
                .iter()
                .map(|column| match column {
                    Column::Removed => ColumnView::Removed,
                    Column::Present(cells) => ColumnView::Present {
                        slots: cells
                            .iter()
                            .map(|cell| {
                                // LE filtre : une carte cachée perd sa valeur.
                                if cell.face_up {
                                    PublicSlot::Visible { value: cell.value }
                                } else {
                                    PublicSlot::Hidden
                                }
                            })
                            .collect(),
                    },
                })
                .collect(),
        })
        .collect();

    let (phase, current_player, finisher, held) = match &game.phase {
        Phase::Lobby => (PhaseName::Lobby, None, None, None),
        Phase::InitialReveal => (PhaseName::InitialReveal, None, None, None),
        Phase::RoundOver => (PhaseName::RoundOver, None, None, None),
        Phase::GameOver => (PhaseName::GameOver, None, None, None),
        Phase::Playing(turn) => {
            let held = turn.held.map(|held| {
                // Une carte piochée n'est connue que de celui qui l'a en main ; une carte prise
                // sur la défausse était déjà visible de tous.
                let visible = held.source == HeldSource::Discard || turn.current == me;
                HeldView {
                    source: held.source,
                    value: visible.then_some(held.value),
                }
            });
            (PhaseName::Playing, Some(turn.current), turn.finisher, held)
        }
    };

    GameView {
        me,
        host: HOST,
        phase,
        round: game.round,
        rules: game.rules,
        players,
        current_player,
        finisher,
        held,
        draw_pile_count: game.draw_pile.len(),
        discard_top: game.discard.last().copied(),
        discard_count: game.discard.len(),
        rounds: game.rounds.clone(),
        winners: game.winners.clone(),
    }
}
