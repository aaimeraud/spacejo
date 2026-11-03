//! # spacejo-core
//!
//! Logique du jeu SpaceJo (règles de Skyjo, voir `docs/RULES.md`), sans aucune dépendance au
//! réseau, à la base de données ni à l'horloge. Tout est déterministe : même graine, mêmes
//! actions → même partie.
//!
//! Points d'entrée :
//! - [`Game::new`], [`Game::join`], [`Game::update_rules`], [`Game::start`] : le salon ;
//! - [`apply`] (ou [`Game::apply`]) : une action de jeu ;
//! - [`view_for`] : la vue filtrée d'un joueur, seule forme de l'état qui sort du serveur.

// Modules : chaque fichier de `src/` est un module déclaré ici.
// https://doc.rust-lang.org/stable/book/ch07-02-defining-modules-to-control-scope-and-privacy.html
mod cards;
mod error;
mod game;
mod grid;
mod rng;
mod rules;
mod scoring;
mod view;

// `pub use` ré-exporte les types pour qu'on écrive `spacejo_core::Game`
// plutôt que `spacejo_core::game::Game`.
pub use cards::{CardValue, DECK_COMPOSITION, DECK_SIZE, full_deck};
pub use error::GameError;
pub use game::{
    Action, Game, HOST, Held, HeldSource, INITIAL_REVEALS, MAX_MISSED_TURNS, MAX_NAME_CHARS,
    MAX_PLAYERS, MIN_PLAYERS, Phase, Player, PlayerId, PlayerRoundScore, RoundResult, SkipOutcome,
    Turn, apply,
};
pub use grid::{CARDS_PER_PLAYER, Cell, Column, GRID_COLUMNS, GRID_ROWS, Grid};
pub use rng::SplitMix64;
pub use rules::{
    DEFAULT_END_SCORE, END_SCORE_MAX, END_SCORE_MIN, EmptyPile, EndTie, FirstPlayerTie,
    NextRoundStarter, PenaltyRule, PenaltyScope, Rules,
};
pub use scoring::finisher_round_score;
pub use view::{ColumnView, GameView, HeldView, PhaseName, PlayerView, PublicSlot, view_for};
