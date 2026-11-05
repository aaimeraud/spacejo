//! Critère 8 : fin de partie à `end_score`, le plus bas total gagne, égalités selon `end_tie`.
mod common;

use common::{FINISHER_5, HIDDEN_30, HIDDEN_72, play_round_where_zero_finishes};
use spacejo_core::{Action, EndTie, Phase, Rules};

#[test]
fn the_game_ends_when_end_score_is_reached_and_the_lowest_total_wins() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].total_score = 95;
    game.players[1].total_score = 50;
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30]);
    // 95 + 5 = 100 : atteint, la partie s'arrête à la fin de cette manche.
    assert_eq!(game.players[0].total_score, 100);
    assert_eq!(game.players[1].total_score, 80);
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![1]);
    // Plus aucune action n'est possible.
    assert!(game.apply(0, Action::NextRound).is_err());
}

#[test]
fn below_end_score_the_game_goes_on() {
    let mut game = common::playing(2, Rules::default());
    game.players[0].total_score = 90;
    game.players[1].total_score = 50;
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30]);
    assert_eq!(game.players[0].total_score, 95);
    assert_eq!(game.phase, Phase::RoundOver);
    assert!(game.winners.is_empty());
}

#[test]
fn a_custom_end_score_is_used() {
    let rules = Rules {
        end_score: 40,
        ..Rules::default()
    };
    let mut game = common::playing(2, rules);
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_72]);
    assert_eq!(game.phase, Phase::GameOver, "72 ≥ 40");
    assert_eq!(game.winners, vec![0]);
}

#[test]
fn shared_victory_names_every_tied_player() {
    let rules = Rules {
        end_tie: EndTie::SharedVictory,
        ..Rules::default()
    };
    let mut game = common::playing(3, rules);
    game.players[0].total_score = 45;
    game.players[1].total_score = 20;
    game.players[2].total_score = 120;
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_72]);
    // Totaux : 50, 50, 192.
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![0, 1]);
}

#[test]
fn extra_round_is_played_until_the_lowest_total_is_unique() {
    let rules = Rules {
        end_tie: EndTie::ExtraRound,
        ..Rules::default()
    };
    let mut game = common::playing(3, rules);
    game.players[0].total_score = 45;
    game.players[1].total_score = 20;
    game.players[2].total_score = 120;
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_72]);
    // Égalité à 50 : pas de gagnant, on rejoue une manche.
    assert_eq!(game.phase, Phase::RoundOver);
    assert!(game.winners.is_empty());

    // Manche supplémentaire encore à égalité (5 + 30 contre 30 + 5 : 85, 85).
    common::everyone_starts_next_round(&mut game);
    game.players[1].total_score = 55; // 55 + 30 = 85, comme 50 + 5 + 30
    game.players[0].total_score = 80; // 80 + 5 = 85
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_72]);
    assert_eq!(game.phase, Phase::RoundOver, "toujours à égalité");

    // Nouvelle manche : le total le plus bas devient unique.
    common::everyone_starts_next_round(&mut game);
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_72]);
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![0]);
    assert_eq!(game.rounds.len(), 3);
}
