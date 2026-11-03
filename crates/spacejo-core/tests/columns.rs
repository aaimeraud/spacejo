//! Critère 5 : une colonne de 3 cartes visibles identiques est retirée, ses cartes vont à la défausse.
mod common;

use common::{hidden_grid, set_turn};
use spacejo_core::{Action, Column, GameError, Rules};

#[test]
fn three_identical_visible_cards_are_removed_to_the_discard() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].grid = hidden_grid(&[[7, 7, 3], [1, 2, 3], [4, 5, 6], [8, 9, 10]]);
    game.players[0].grid.cell_mut(0, 0).unwrap().face_up = true;
    game.players[0].grid.cell_mut(0, 1).unwrap().face_up = true;
    game.draw_pile = vec![0, 7];
    game.discard = vec![-1];
    set_turn(&mut game, 0);
    let before = game.card_count();

    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::SwapHeld { column: 0, row: 2 })
        .unwrap();

    assert_eq!(game.players[0].grid.columns[0], Column::Removed);
    // Défausse : -1, puis la carte remplacée (3), puis les trois 7 de la colonne.
    assert_eq!(game.discard, vec![-1, 3, 7, 7, 7]);
    assert_eq!(game.players[0].grid.card_count(), 9);
    assert_eq!(game.card_count(), before, "aucune carte perdue");
}

#[test]
fn a_column_with_a_hidden_card_stays() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].grid = hidden_grid(&[[7, 7, 7], [1, 2, 3], [4, 5, 6], [8, 9, 10]]);
    game.players[0].grid.cell_mut(0, 0).unwrap().face_up = true;
    game.draw_pile = vec![0, 4];
    set_turn(&mut game, 0);
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 0, row: 1 })
        .unwrap();
    // Deux 7 visibles, un caché : la colonne reste.
    assert!(matches!(
        game.players[0].grid.columns[0],
        Column::Present(_)
    ));
}

#[test]
fn revealing_the_third_identical_card_also_removes_the_column() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].grid = hidden_grid(&[[1, 2, 3], [-1, -1, -1], [4, 5, 6], [8, 9, 10]]);
    game.players[0].grid.cell_mut(1, 0).unwrap().face_up = true;
    game.players[0].grid.cell_mut(1, 2).unwrap().face_up = true;
    game.draw_pile = vec![0, 12];
    game.discard = vec![5];
    set_turn(&mut game, 0);
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 1, row: 1 })
        .unwrap();
    assert_eq!(game.players[0].grid.columns[1], Column::Removed);
    assert_eq!(game.discard, vec![5, 12, -1, -1, -1]);
}

#[test]
fn a_removed_column_cannot_be_targeted() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].grid = hidden_grid(&[[1, 2, 3], [4, 5, 6], [4, 5, 6], [8, 9, 10]]);
    game.players[0].grid.columns[1] = Column::Removed;
    game.draw_pile = vec![0, 12];
    set_turn(&mut game, 0);
    game.apply(0, Action::DrawFromPile).unwrap();
    assert_eq!(
        game.apply(0, Action::SwapHeld { column: 1, row: 0 }),
        Err(GameError::InvalidPosition)
    );
}
