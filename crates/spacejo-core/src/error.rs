//! Erreurs du jeu. Chaque erreur a un code stable (repris tel quel par l'API, voir
//! docs/CONTRACT.md) et un message en français pour l'interface.

use std::fmt;

/// Toutes les raisons pour lesquelles une opération sur la partie peut être refusée.
/// Une opération refusée ne modifie jamais la partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    InvalidName,
    InvalidRules,
    UnknownPlayer,
    NotHost,
    WrongPhase,
    NotYourTurn,
    GameFull,
    NotEnoughPlayers,
    InvalidPosition,
    AlreadyRevealed,
    InitialRevealsDone,
    AlreadyHolding,
    NothingHeld,
    MustSwapDiscard,
    EmptyDiscard,
    EmptyPile,
    /// Le joueur a été éliminé (trop de tours manqués) : il ne peut plus jouer.
    Eliminated,
    /// L'hôte ne peut pas se retirer lui-même du salon (il peut le quitter).
    CannotKickHost,
    /// Le dernier joueur d'un salon ne peut pas en être retiré : un salon n'est jamais vide.
    LastPlayer,
    /// Plus de joueurs que le paquet ne peut en servir (impossible avec 4 joueurs maximum).
    NotEnoughCards,
}

impl GameError {
    /// Code stable, en `snake_case`, envoyé au navigateur.
    pub fn code(&self) -> &'static str {
        match self {
            GameError::InvalidName => "invalid_name",
            GameError::InvalidRules => "invalid_rules",
            GameError::UnknownPlayer => "unknown_player",
            GameError::NotHost => "not_host",
            GameError::WrongPhase => "wrong_phase",
            GameError::NotYourTurn => "not_your_turn",
            GameError::GameFull => "game_full",
            GameError::NotEnoughPlayers => "not_enough_players",
            GameError::InvalidPosition => "invalid_position",
            GameError::AlreadyRevealed => "already_revealed",
            GameError::InitialRevealsDone => "initial_reveals_done",
            GameError::AlreadyHolding => "already_holding",
            GameError::NothingHeld => "nothing_held",
            GameError::MustSwapDiscard => "must_swap_discard",
            GameError::EmptyDiscard => "empty_discard",
            GameError::EmptyPile => "empty_pile",
            GameError::Eliminated => "eliminated",
            GameError::NotEnoughCards => "not_enough_cards",
            GameError::CannotKickHost => "cannot_kick_host",
            GameError::LastPlayer => "last_player",
        }
    }

    /// Message lisible, en français.
    pub fn message(&self) -> &'static str {
        match self {
            GameError::InvalidName => "Le nom doit contenir de 1 à 20 caractères.",
            GameError::InvalidRules => {
                "Réglage invalide : le score de fin doit être entre 10 et 500."
            }
            GameError::UnknownPlayer => "Joueur inconnu dans cette partie.",
            GameError::NotHost => "Seul l'hôte de la partie peut faire cela.",
            GameError::WrongPhase => "Cette action n'est pas possible à ce moment de la partie.",
            GameError::NotYourTurn => "Ce n'est pas votre tour.",
            GameError::GameFull => "La partie est complète.",
            GameError::NotEnoughPlayers => "Il faut au moins 2 joueurs pour lancer la partie.",
            GameError::InvalidPosition => "Cette case n'existe pas (ou sa colonne a été retirée).",
            GameError::AlreadyRevealed => "Cette carte est déjà visible.",
            GameError::InitialRevealsDone => "Vous avez déjà retourné vos 2 cartes.",
            GameError::AlreadyHolding => "Vous avez déjà une carte en main.",
            GameError::NothingHeld => "Piochez ou prenez d'abord une carte.",
            GameError::MustSwapDiscard => {
                "Une carte prise sur la défausse doit être échangée avec une carte de votre grille."
            }
            GameError::EmptyDiscard => "La défausse est vide.",
            GameError::EmptyPile => "La pioche est vide.",
            GameError::Eliminated => {
                "Vous ne jouez plus dans cette partie (éliminé·e ou partie abandonnée)."
            }
            GameError::NotEnoughCards => "Pas assez de cartes pour autant de joueurs.",
            GameError::CannotKickHost => "L'hôte ne peut pas se retirer lui-même de la partie.",
            GameError::LastPlayer => "Le dernier joueur du salon ne peut pas en être retiré.",
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for GameError {}
