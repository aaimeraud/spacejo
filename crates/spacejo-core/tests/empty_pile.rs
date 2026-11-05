//! Critère 15 : pioche épuisée, selon `empty_pile`.
mod common;

use common::hidden_grid;
use spacejo_core::{Action, EmptyPile, Game, Phase, Rules};

fn rules(empty_pile: EmptyPile) -> Rules {
    Rules {
        empty_pile,
        ..Rules::default()
    }
}

/// Partie réelle à 2 joueurs (150 cartes) où la pioche ne contient plus qu'une carte :
/// toutes les autres ont été déplacées sur la défausse.
fn last_card_in_pile(rules: Rules) -> Game {
    let mut game = common::playing(2, rules);
    let rest: Vec<i32> = game.draw_pile.drain(1..).collect();
    game.discard.extend(rest);
    assert_eq!(game.draw_pile.len(), 1);
    assert_eq!(game.card_count(), 150);
    game
}

/// Le joueur courant pioche la dernière carte et la défausse en retournant une carte cachée.
fn draw_last_card_and_discard_it(game: &mut Game) -> i32 {
    let p = common::current(game);
    let drawn = *game.draw_pile.last().unwrap();
    game.apply(p, Action::DrawFromPile).unwrap();
    let (column, row) = game.players[p].grid.first_hidden().unwrap();
    game.apply(p, Action::DiscardHeldAndReveal { column, row })
        .unwrap();
    drawn
}

#[test]
fn reshuffle_keeps_the_top_discard_card_visible_and_150_cards() {
    let mut game = last_card_in_pile(rules(EmptyPile::ReshuffleDiscard));
    let discard_before = game.discard.len();
    let first = common::current(&game);
    let drawn = draw_last_card_and_discard_it(&mut game);

    // La carte du dessus (celle qui vient d'être défaussée) reste seule, visible, sur la défausse.
    assert_eq!(game.discard, vec![drawn]);
    // Tout le reste de la défausse forme la nouvelle pioche.
    assert_eq!(game.draw_pile.len(), discard_before);
    assert_eq!(game.card_count(), 150);
    // La partie continue normalement avec le joueur suivant.
    assert_eq!(common::current(&game), (first + 1) % 2);
    common::play_simple_turn(&mut game);
    assert_eq!(game.card_count(), 150);
}

#[test]
fn end_round_stops_the_round_immediately_without_penalty() {
    let mut game = last_card_in_pile(rules(EmptyPile::EndRound));
    draw_last_card_and_discard_it(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);
    let round = &game.rounds[0];
    assert_eq!(round.finisher, None, "personne n'a terminé");
    assert!(round.scores.iter().all(|s| !s.penalized));
    assert!(round.scores.iter().all(|s| s.score == s.raw));
    for player in &game.players {
        assert_eq!(player.grid.hidden_count(), 0, "tout est retourné");
    }
    assert_eq!(game.card_count(), 150);
}

#[test]
fn empty_pile_with_a_single_discard_card_ends_the_round_without_error() {
    let mut game = common::playing(2, rules(EmptyPile::ReshuffleDiscard));
    common::set_turn(&mut game, 0);
    game.draw_pile = vec![4];
    game.discard = vec![];
    game.apply(0, Action::DrawFromPile).unwrap();
    // L'échange pose la carte remplacée sur la défausse : elle n'a qu'une carte, la pioche est vide.
    game.apply(0, Action::SwapHeld { column: 1, row: 1 })
        .unwrap();
    assert!(game.draw_pile.is_empty());
    assert_eq!(game.discard.len(), 1);
    assert_eq!(game.phase, Phase::RoundOver);
    assert_eq!(game.rounds[0].finisher, None);
}

#[test]
fn end_round_after_someone_finished_still_penalizes_the_finisher() {
    // Hypothèse (ASSUMPTIONS.md) : si un joueur a déjà terminé, il reste le finisseur.
    let mut game = common::playing(2, rules(EmptyPile::EndRound));
    game.players[0].grid = common::visible_grid(&common::FINISHER_5);
    game.players[0].grid.cell_mut(3, 2).unwrap().face_up = false;
    game.players[1].grid = hidden_grid(&[[0, 0, 1], [0, 1, 0], [1, 0, 0], [0, 0, 1]]);
    game.draw_pile = vec![8, 8];
    game.discard = vec![9];
    common::set_turn(&mut game, 0);
    // Le joueur 0 termine (il reste une carte dans la pioche).
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    // Le joueur 1 pioche la dernière carte : la manche s'arrête aussitôt.
    common::play_simple_turn(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);
    assert_eq!(game.rounds[0].finisher, Some(0));
    // 5 contre 4 : le finisseur est doublé.
    assert_eq!(game.rounds[0].scores[0].score, 10);
}

#[test]
fn without_a_finisher_the_finisher_rule_falls_back_to_the_highest_sum() {
    // Hypothèse (ASSUMPTIONS.md) : `Finisher` sans finisseur → règle de la première manche.
    let mut game = last_card_in_pile(rules(EmptyPile::EndRound));
    draw_last_card_and_discard_it(&mut game);
    common::everyone_starts_next_round(&mut game);
    for p in 0..2 {
        let top = if p == 1 { 12 } else { 1 };
        game.players[p].grid = hidden_grid(&[[top, top, 0], [0, 1, 2], [2, 1, 0], [0, 1, 2]]);
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    assert_eq!(common::current(&game), 1);
}
