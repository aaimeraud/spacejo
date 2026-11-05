//! Issue #11 : en fin de manche, chaque joueur vote pour la manche suivante ; seul l'hôte la lance, avec les
//! joueurs qui ont voté. Ceux qui n'ont pas voté quittent la partie (ils sont marqués `abandoned`).
mod common;

use spacejo_core::{Action, GameError, HOST, Phase, Rules, view_for};

/// Partie à `n` joueurs arrivée en fin de manche.
fn round_over(n: usize) -> spacejo_core::Game {
    let mut game = common::playing(n, Rules::default());
    game.phase = Phase::RoundOver;
    game
}

#[test]
fn a_vote_does_not_start_the_next_round() {
    let mut game = round_over(3);
    game.apply(1, Action::NextRound).unwrap();
    assert_eq!(game.phase, Phase::RoundOver);
    assert_eq!(game.next_round_votes, vec![1]);
}

#[test]
fn voting_twice_counts_once() {
    let mut game = round_over(3);
    game.apply(2, Action::NextRound).unwrap();
    game.apply(2, Action::NextRound).unwrap();
    assert_eq!(game.next_round_votes, vec![2]);
}

#[test]
fn everyone_sees_who_voted_and_who_decides() {
    let mut game = round_over(3);
    game.apply(2, Action::NextRound).unwrap();
    game.apply(1, Action::NextRound).unwrap();
    let view = view_for(&game, 0);
    assert_eq!(
        view.next_round_votes,
        vec![1, 2],
        "triés dans l'ordre d'arrivée"
    );
    assert_eq!(view.next_round_decider, Some(HOST));
}

#[test]
fn the_host_starts_the_round_with_the_voters_only() {
    let mut game = round_over(4);
    game.apply(1, Action::NextRound).unwrap();
    game.apply(3, Action::NextRound).unwrap();
    game.apply(HOST, Action::NextRound).unwrap();
    assert_eq!(game.phase, Phase::InitialReveal);
    assert_eq!(game.round, 2);
    // Le joueur 2 n'a pas voté : il quitte la partie, comme s'il l'avait quittée lui-même.
    assert!(game.players[2].eliminated);
    assert!(game.players[2].abandoned);
    assert_eq!(
        game.players[2].grid.card_count(),
        0,
        "il ne reçoit pas de cartes"
    );
    for p in [0, 1, 3] {
        assert!(!game.players[p].eliminated);
        assert_eq!(game.players[p].grid.card_count(), 12);
    }
    assert!(
        game.next_round_votes.is_empty(),
        "les votes repartent de zéro"
    );
    assert_eq!(game.card_count(), 150);
}

#[test]
fn the_host_cannot_start_alone() {
    let mut game = round_over(3);
    let before = game.clone();
    assert_eq!(
        game.apply(HOST, Action::NextRound),
        Err(GameError::NotEnoughVotes)
    );
    assert_eq!(game, before, "un refus ne change rien");
}

#[test]
fn a_player_who_left_after_voting_does_not_count() {
    let mut game = round_over(3);
    game.apply(1, Action::NextRound).unwrap();
    game.abandon(1).unwrap();
    assert_eq!(view_for(&game, 0).next_round_votes, Vec::<usize>::new());
    assert_eq!(
        game.apply(HOST, Action::NextRound),
        Err(GameError::NotEnoughVotes)
    );
}

#[test]
fn if_the_host_has_left_the_next_player_in_the_game_decides() {
    let mut game = round_over(3);
    game.abandon(HOST).unwrap();
    assert_eq!(game.next_round_decider(), Some(1));
    game.apply(2, Action::NextRound).unwrap();
    game.apply(1, Action::NextRound).unwrap();
    assert_eq!(game.phase, Phase::InitialReveal);
}

#[test]
fn legal_actions_follow_the_vote() {
    let mut game = round_over(3);
    assert!(
        game.legal_actions(HOST).is_empty(),
        "personne n'a encore voté"
    );
    assert_eq!(game.legal_actions(1), vec![Action::NextRound]);
    game.apply(1, Action::NextRound).unwrap();
    assert!(game.legal_actions(1).is_empty(), "déjà voté");
    assert_eq!(game.legal_actions(HOST), vec![Action::NextRound]);
}

#[test]
fn voting_is_refused_outside_the_end_of_a_round() {
    let mut game = common::playing(3, Rules::default());
    assert_eq!(game.apply(1, Action::NextRound), Err(GameError::WrongPhase));
}
