//! Critère 4 : les deux voies d'un tour et leurs contraintes.
mod common;

use common::{hidden_grid, set_turn};
use spacejo_core::{Action, Cell, Game, GameError, HeldSource, Phase, Rules};

/// Partie à 2 joueurs, au tour du joueur 0, avec une pioche et une défausse connues.
fn fixture() -> Game {
    let mut game = common::playing(2, Rules::default());
    game.players[0].grid = hidden_grid(&[[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]]);
    game.players[0].grid.cell_mut(0, 0).unwrap().face_up = true; // la carte 1 est visible
    game.draw_pile = vec![0, 0, 0, 9];
    game.discard = vec![-1, -2];
    set_turn(&mut game, 0);
    game
}

fn held(game: &Game) -> Option<(i32, HeldSource)> {
    match &game.phase {
        Phase::Playing(turn) => turn.held.map(|h| (h.value, h.source)),
        _ => None,
    }
}

#[test]
fn draw_then_swap_with_a_hidden_card() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    assert_eq!(held(&game), Some((9, HeldSource::Pile)));
    game.apply(0, Action::SwapHeld { column: 1, row: 2 })
        .unwrap();
    // La carte piochée est posée face visible…
    assert_eq!(
        *game.players[0].grid.cell(1, 2).unwrap(),
        Cell {
            value: 9,
            face_up: true
        }
    );
    // … et la carte remplacée (cachée, 6) part face visible sur la défausse.
    assert_eq!(game.discard.last(), Some(&6));
    assert_eq!(common::current(&game), 1, "le tour passe au joueur suivant");
}

#[test]
fn draw_then_swap_with_a_visible_card() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::SwapHeld { column: 0, row: 0 })
        .unwrap();
    assert_eq!(game.players[0].grid.cell(0, 0).unwrap().value, 9);
    assert_eq!(game.discard.last(), Some(&1));
}

#[test]
fn draw_then_discard_and_reveal() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 2, row: 1 })
        .unwrap();
    assert_eq!(game.discard.last(), Some(&9));
    assert_eq!(
        *game.players[0].grid.cell(2, 1).unwrap(),
        Cell {
            value: 8,
            face_up: true
        }
    );
    assert_eq!(common::current(&game), 1);
}

#[test]
fn discard_and_reveal_needs_a_hidden_card() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    assert_eq!(
        game.apply(0, Action::DiscardHeldAndReveal { column: 0, row: 0 }),
        Err(GameError::AlreadyRevealed)
    );
}

#[test]
fn taking_the_discard_forces_a_swap() {
    let mut game = fixture();
    game.apply(0, Action::TakeDiscard).unwrap();
    assert_eq!(held(&game), Some((-2, HeldSource::Discard)));
    // Impossible de rejeter la carte prise sur la défausse.
    assert_eq!(
        game.apply(0, Action::DiscardHeldAndReveal { column: 1, row: 0 }),
        Err(GameError::MustSwapDiscard)
    );
    assert_eq!(
        game.apply(0, Action::DrawFromPile),
        Err(GameError::AlreadyHolding)
    );
    game.apply(0, Action::SwapHeld { column: 3, row: 0 })
        .unwrap();
    assert_eq!(
        *game.players[0].grid.cell(3, 0).unwrap(),
        Cell {
            value: -2,
            face_up: true
        }
    );
    assert_eq!(game.discard, vec![-1, 10]);
}

#[test]
fn cannot_hold_two_cards_or_swap_with_nothing() {
    let mut game = fixture();
    assert_eq!(
        game.apply(0, Action::SwapHeld { column: 0, row: 0 }),
        Err(GameError::NothingHeld)
    );
    assert_eq!(
        game.apply(0, Action::DiscardHeldAndReveal { column: 1, row: 0 }),
        Err(GameError::NothingHeld)
    );
    game.apply(0, Action::DrawFromPile).unwrap();
    assert_eq!(
        game.apply(0, Action::TakeDiscard),
        Err(GameError::AlreadyHolding)
    );
}

#[test]
fn positions_outside_the_grid_are_refused() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    assert_eq!(
        game.apply(0, Action::SwapHeld { column: 0, row: 3 }),
        Err(GameError::InvalidPosition)
    );
    assert_eq!(
        game.apply(0, Action::SwapHeld { column: 4, row: 0 }),
        Err(GameError::InvalidPosition)
    );
}

#[test]
fn legal_actions_follow_the_turn_state() {
    let mut game = fixture();
    let actions = game.legal_actions(0);
    assert!(actions.contains(&Action::DrawFromPile));
    assert!(actions.contains(&Action::TakeDiscard));
    assert!(game.legal_actions(1).is_empty(), "pas son tour");
    game.apply(0, Action::TakeDiscard).unwrap();
    let actions = game.legal_actions(0);
    assert_eq!(actions.len(), 12, "12 échanges possibles, aucune défausse");
    assert!(actions.iter().all(|a| matches!(a, Action::SwapHeld { .. })));
}
