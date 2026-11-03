//! Critère 2 : action hors de son tour ou dans la mauvaise phase refusée avec une erreur claire.
mod common;

use spacejo_core::{Action, GameError, Rules};

#[test]
fn acting_out_of_turn_is_refused() {
    let mut game = common::playing(3, Rules::default());
    let current = common::current(&game);
    let other = (current + 1) % 3;
    let before = game.clone();
    let err = game.apply(other, Action::DrawFromPile).unwrap_err();
    assert_eq!(err, GameError::NotYourTurn);
    assert_eq!(err.code(), "not_your_turn");
    assert!(!err.message().is_empty());
    // Une action refusée ne modifie rien.
    assert_eq!(game, before);
}

#[test]
fn playing_during_the_initial_reveal_is_refused() {
    let mut game = common::lobby(2, Rules::default());
    game.start(0).unwrap();
    assert_eq!(
        game.apply(0, Action::DrawFromPile),
        Err(GameError::WrongPhase)
    );
    assert_eq!(
        game.apply(0, Action::TakeDiscard),
        Err(GameError::WrongPhase)
    );
}

#[test]
fn actions_in_the_lobby_are_refused() {
    let mut game = common::lobby(2, Rules::default());
    assert_eq!(
        game.apply(0, Action::RevealInitial { column: 0, row: 0 }),
        Err(GameError::WrongPhase)
    );
    assert_eq!(game.apply(0, Action::NextRound), Err(GameError::WrongPhase));
}

#[test]
fn initial_reveal_after_it_is_over_is_refused() {
    let mut game = common::playing(2, Rules::default());
    let p = common::current(&game);
    assert_eq!(
        game.apply(p, Action::RevealInitial { column: 1, row: 0 }),
        Err(GameError::WrongPhase)
    );
    assert_eq!(game.apply(p, Action::NextRound), Err(GameError::WrongPhase));
}

#[test]
fn joining_or_starting_after_the_start_is_refused() {
    let mut game = common::playing(2, Rules::default());
    assert_eq!(game.join("Tard"), Err(GameError::WrongPhase));
    assert_eq!(game.start(0), Err(GameError::WrongPhase));
}

#[test]
fn unknown_player_is_refused() {
    let mut game = common::playing(2, Rules::default());
    assert_eq!(
        game.apply(9, Action::DrawFromPile),
        Err(GameError::UnknownPlayer)
    );
}

#[test]
fn the_free_function_apply_matches_the_method() {
    let mut a = common::playing(2, Rules::default());
    let mut b = a.clone();
    let p = common::current(&a);
    a.apply(p, Action::DrawFromPile).unwrap();
    spacejo_core::apply(&mut b, p, Action::DrawFromPile).unwrap();
    assert_eq!(a, b);
}
