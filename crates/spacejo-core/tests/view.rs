//! Critère 10 : aucune valeur de carte cachée ne fuit dans la vue d'un joueur.
mod common;

use spacejo_core::{Action, ColumnView, Game, HeldView, PhaseName, PublicSlot, Rules, view_for};

/// Change la valeur d'une carte précise sans toucher au reste.
fn with_card_value(game: &Game, player: usize, column: usize, row: usize, value: i32) -> Game {
    let mut other = game.clone();
    other.players[player]
        .grid
        .cell_mut(column, row)
        .unwrap()
        .value = value;
    other
}

/// Trouve une carte cachée du joueur `player`.
fn some_hidden(game: &Game, player: usize) -> (usize, usize) {
    game.players[player].grid.first_hidden().unwrap()
}

#[test]
fn changing_one_hidden_card_does_not_change_the_other_players_views() {
    let game = common::playing(3, Rules::default());
    let (c, r) = some_hidden(&game, 1);
    let original = game.players[1].grid.cell(c, r).unwrap().value;
    let altered = with_card_value(&game, 1, c, r, if original == 12 { -2 } else { 12 });

    // Pour les joueurs qui ne possèdent pas la carte, les deux vues sont identiques.
    for viewer in [0, 2] {
        assert_eq!(view_for(&game, viewer), view_for(&altered, viewer));
        // Même chose une fois sérialisée en JSON (ce qui part vraiment sur le réseau).
        assert_eq!(
            serde_json::to_string(&view_for(&game, viewer)).unwrap(),
            serde_json::to_string(&view_for(&altered, viewer)).unwrap()
        );
    }
    // Même pour le propriétaire : une carte face cachée n'est connue de personne.
    assert_eq!(view_for(&game, 1), view_for(&altered, 1));
}

#[test]
fn changing_a_draw_pile_card_changes_no_view() {
    let game = common::playing(2, Rules::default());
    let mut altered = game.clone();
    let top = altered.draw_pile.len() - 1;
    altered.draw_pile[top] = if altered.draw_pile[top] == 12 { -2 } else { 12 };
    altered.draw_pile[0] = if altered.draw_pile[0] == 5 { 6 } else { 5 };
    for viewer in 0..2 {
        assert_eq!(view_for(&game, viewer), view_for(&altered, viewer));
    }
}

#[test]
fn hidden_slots_carry_no_value_and_visible_ones_do() {
    let game = common::playing(2, Rules::default());
    let view = view_for(&game, 0);
    for (p, player) in view.players.iter().enumerate() {
        for (c, column) in player.grid.iter().enumerate() {
            let ColumnView::Present { slots } = column else {
                continue;
            };
            for (r, slot) in slots.iter().enumerate() {
                let cell = game.players[p].grid.cell(c, r).unwrap();
                match slot {
                    PublicSlot::Hidden => assert!(!cell.face_up),
                    PublicSlot::Visible { value } => {
                        assert!(cell.face_up);
                        assert_eq!(*value, cell.value);
                    }
                }
            }
        }
    }
    // En JSON, une case cachée n'a pas de champ « value ».
    let json = serde_json::to_value(&view).unwrap();
    let first_column = &json["players"][0]["grid"][1]["slots"][0];
    assert_eq!(first_column, &serde_json::json!({ "kind": "Hidden" }));
}

#[test]
fn a_card_drawn_from_the_pile_is_only_shown_to_its_holder() {
    let mut game = common::playing(2, Rules::default());
    let p = common::current(&game);
    let other = 1 - p;
    game.apply(p, Action::DrawFromPile).unwrap();
    let value = match view_for(&game, p).held {
        Some(HeldView { value: Some(v), .. }) => v,
        held => panic!("le joueur courant doit voir sa carte : {held:?}"),
    };
    assert_eq!(view_for(&game, other).held.unwrap().value, None);

    // Changer la carte piochée ne change pas la vue de l'autre joueur.
    let mut altered = game.clone();
    if let spacejo_core::Phase::Playing(turn) = &mut altered.phase {
        turn.held.as_mut().unwrap().value = if value == 12 { -2 } else { 12 };
    }
    assert_eq!(view_for(&game, other), view_for(&altered, other));
}

#[test]
fn a_card_taken_from_the_discard_stays_public() {
    let mut game = common::playing(2, Rules::default());
    let p = common::current(&game);
    let top = *game.discard.last().unwrap();
    game.apply(p, Action::TakeDiscard).unwrap();
    assert_eq!(view_for(&game, 1 - p).held.unwrap().value, Some(top));
}

#[test]
fn the_view_shows_public_information() {
    let game = common::playing(3, Rules::default());
    let view = view_for(&game, 2);
    assert_eq!(view.me, 2);
    assert_eq!(view.host, 0);
    assert_eq!(view.phase, PhaseName::Playing);
    assert_eq!(view.round, 1);
    assert_eq!(view.rules, Rules::default());
    assert_eq!(view.players.len(), 3);
    assert_eq!(view.players[1].name, "P1");
    assert_eq!(view.draw_pile_count, game.draw_pile.len());
    assert_eq!(view.discard_count, game.discard.len());
    assert_eq!(view.discard_top, game.discard.last().copied());
    assert_eq!(view.current_player, game.current_player());
    assert!(view.winners.is_empty());
    // L'état du générateur aléatoire n'apparaît nulle part.
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains("rng"));
    assert!(!json.contains("state"));
}

#[test]
fn after_the_round_every_card_is_visible_to_everyone() {
    let mut game = common::playing(2, Rules::default());
    common::play_round_where_zero_finishes(&mut game, &[common::FINISHER_5, common::HIDDEN_30]);
    let view = view_for(&game, 1);
    assert_eq!(view.phase, PhaseName::RoundOver);
    for player in &view.players {
        for column in &player.grid {
            if let ColumnView::Present { slots } = column {
                assert!(
                    slots
                        .iter()
                        .all(|s| matches!(s, PublicSlot::Visible { .. }))
                );
            }
        }
    }
    assert_eq!(view.rounds.len(), 1);
}
