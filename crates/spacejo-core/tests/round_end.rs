//! Critère 6 : fin de manche, dernier tour des autres joueurs, puis toutes les cartes sont comptées.
mod common;

use common::{hidden_grid, set_turn, visible_grid};
use spacejo_core::{Action, Game, Phase, Rules, Turn};

/// 3 joueurs. Le joueur 0 n'a plus qu'une carte cachée (la dernière de sa grille, valeur -1).
fn fixture() -> Game {
    let mut game = common::playing(3, Rules::default());
    game.players[0].grid = visible_grid(&[[0, 1, 0], [1, 0, 1], [0, 2, 0], [1, 0, -1]]);
    game.players[0].grid.cell_mut(3, 2).unwrap().face_up = false;
    game.players[1].grid = hidden_grid(&[[5, 6, 7], [5, 6, 7], [5, 6, 7], [5, 6, 7]]);
    game.players[2].grid = hidden_grid(&[[2, 3, 4], [2, 3, 4], [2, 3, 4], [2, 3, 4]]);
    game.draw_pile = vec![8; 20];
    game.discard = vec![9];
    set_turn(&mut game, 0);
    game
}

#[test]
fn the_others_play_one_last_turn_then_everything_is_counted() {
    let mut game = fixture();

    // Le joueur 0 retourne sa dernière carte : il termine la manche.
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    assert_eq!(
        game.phase,
        Phase::Playing(Turn {
            current: 1,
            held: None,
            finisher: Some(0)
        })
    );

    // Dernier tour du joueur 1.
    common::play_simple_turn(&mut game);
    assert_eq!(common::current(&game), 2);

    // Dernier tour du joueur 2 : la manche se termine, le joueur 0 ne rejoue pas.
    common::play_simple_turn(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);

    // Toutes les cartes restantes sont retournées.
    for player in &game.players {
        assert_eq!(player.grid.hidden_count(), 0);
    }

    // Score de manche = somme des cartes restantes (le finisseur est seul le plus bas : pas doublé).
    let round = &game.rounds[0];
    assert_eq!(round.round, 1);
    assert_eq!(round.finisher, Some(0));
    let raws: Vec<i32> = round.scores.iter().map(|s| s.raw).collect();
    assert_eq!(raws, vec![5, 72, 36]);
    let scores: Vec<i32> = round.scores.iter().map(|s| s.score).collect();
    assert_eq!(scores, vec![5, 72, 36]);
    assert!(round.scores.iter().all(|s| !s.penalized));
    let totals: Vec<i32> = game.players.iter().map(|p| p.total_score).collect();
    assert_eq!(totals, vec![5, 72, 36]);
}

#[test]
fn the_finisher_is_doubled_when_not_strictly_lowest() {
    let mut game = fixture();
    // Le joueur 2 a une grille très basse : le finisseur (5) est battu.
    game.players[2].grid = hidden_grid(&[[0, 0, -2], [0, 0, -1], [0, 0, 0], [-1, 0, 0]]);
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    common::play_simple_turn(&mut game);
    common::play_simple_turn(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);
    let finisher = &game.rounds[0].scores[0];
    assert_eq!(finisher.raw, 5);
    assert_eq!(finisher.score, 10);
    assert!(finisher.penalized);
    assert_eq!(game.players[0].total_score, 10);
}

#[test]
fn next_round_deals_a_fresh_full_deck() {
    let mut game = fixture();
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    common::play_simple_turn(&mut game);
    common::play_simple_turn(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);
    // N'importe quel joueur peut lancer la manche suivante.
    game.apply(2, Action::NextRound).unwrap();
    assert_eq!(game.phase, Phase::InitialReveal);
    assert_eq!(game.round, 2);
    assert_eq!(game.card_count(), 150);
    for player in &game.players {
        assert_eq!(player.grid.hidden_count(), 12);
        assert_eq!(player.initial_reveals, 0);
    }
    // Les totaux sont conservés.
    assert_eq!(game.players[1].total_score, 72);
}
