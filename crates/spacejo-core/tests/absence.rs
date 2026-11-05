//! Joueurs absents (tour passé par le serveur), élimination et parties jusqu'à 10 joueurs.
//! Voir ASSUMPTIONS.md, n° 18 à 22.
mod common;

use common::{hidden_grid, play_simple_turn, set_turn};
use spacejo_core::{
    Action, Game, GameError, HeldSource, MAX_MISSED_TURNS, MAX_PLAYERS, Phase, Rules, SkipOutcome,
};

fn current(game: &Game) -> usize {
    common::current(game)
}

#[test]
fn up_to_ten_players_can_play() {
    assert_eq!(MAX_PLAYERS, 10);
    let mut game = common::lobby(10, Rules::default());
    assert_eq!(game.join("Onzième"), Err(GameError::GameFull));
    game.start(0).unwrap();
    assert_eq!(game.card_count(), 150);
    assert_eq!(game.draw_pile.len(), 150 - 10 * 12 - 1);
}

#[test]
fn skipping_an_absent_player_discards_the_held_card_and_passes_the_turn() {
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 1);
    game.apply(1, Action::DrawFromPile).unwrap();
    let held = match &game.phase {
        Phase::Playing(turn) => turn.held.unwrap(),
        _ => unreachable!(),
    };
    assert_eq!(held.source, HeldSource::Pile);
    let before = game.card_count();

    assert_eq!(game.skip_absent(1), Ok(SkipOutcome::Skipped));
    assert_eq!(current(&game), 2, "le tour passe au joueur suivant");
    assert_eq!(
        game.discard.last(),
        Some(&held.value),
        "la carte en main part sur la défausse"
    );
    assert_eq!(game.players[1].missed_turns, 1);
    assert!(!game.players[1].eliminated);
    assert_eq!(game.card_count(), before);
}

#[test]
fn only_the_current_player_can_be_skipped_and_only_during_play() {
    let mut game = common::lobby(2, Rules::default());
    assert_eq!(game.skip_absent(1), Err(GameError::WrongPhase));
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 0);
    assert_eq!(game.skip_absent(2), Err(GameError::NotYourTurn));
    assert_eq!(game.skip_absent(9), Err(GameError::UnknownPlayer));
}

#[test]
fn a_second_missed_turn_eliminates_the_player() {
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 1);
    assert_eq!(MAX_MISSED_TURNS, 2);
    assert_eq!(game.skip_absent(1), Ok(SkipOutcome::Skipped));
    play_simple_turn(&mut game); // joueur 2
    play_simple_turn(&mut game); // joueur 0
    assert_eq!(current(&game), 1);
    assert_eq!(game.skip_absent(1), Ok(SkipOutcome::Eliminated));
    assert!(game.players[1].eliminated);
    assert_eq!(current(&game), 2);

    // L'ordre du tour saute désormais le joueur éliminé.
    play_simple_turn(&mut game); // joueur 2
    assert_eq!(current(&game), 0);
    play_simple_turn(&mut game); // joueur 0
    assert_eq!(current(&game), 2);

    // Un joueur éliminé ne peut plus agir.
    assert_eq!(
        game.apply(1, Action::DrawFromPile),
        Err(GameError::Eliminated)
    );
    assert!(game.legal_actions(1).is_empty());
    assert_eq!(GameError::Eliminated.code(), "eliminated");
}

#[test]
fn playing_again_resets_the_missed_turns() {
    let mut game = common::playing(2, Rules::default());
    set_turn(&mut game, 0);
    game.skip_absent(0).unwrap();
    assert_eq!(game.players[0].missed_turns, 1);
    play_simple_turn(&mut game); // joueur 1
    play_simple_turn(&mut game); // joueur 0 revient et joue
    assert_eq!(game.players[0].missed_turns, 0);
    play_simple_turn(&mut game); // joueur 1
    // De nouveau absent : c'est un premier tour manqué, pas une élimination.
    assert_eq!(game.skip_absent(0), Ok(SkipOutcome::Skipped));
}

#[test]
fn the_last_player_standing_wins() {
    let mut game = common::playing(2, Rules::default());
    set_turn(&mut game, 1);
    game.players[1].missed_turns = MAX_MISSED_TURNS - 1;
    assert_eq!(game.skip_absent(1), Ok(SkipOutcome::Eliminated));
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![0]);
}

#[test]
fn eliminated_players_get_no_cards_in_the_next_round() {
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 2);
    game.players[2].missed_turns = MAX_MISSED_TURNS - 1;
    game.skip_absent(2).unwrap();
    assert!(game.players[2].eliminated);
    common::play_round_where_zero_finishes(&mut game, &[common::FINISHER_5, common::HIDDEN_30]);
    assert_eq!(game.phase, Phase::RoundOver);
    common::everyone_starts_next_round(&mut game);
    assert_eq!(game.players[2].grid.card_count(), 0);
    assert_eq!(game.players[0].grid.card_count(), 12);
    assert_eq!(game.card_count(), 150);
    // La mise en place n'attend pas le joueur éliminé.
    for p in 0..2 {
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    assert!(matches!(game.phase, Phase::Playing(_)));
    assert_ne!(current(&game), 2);
}

#[test]
fn eliminated_players_are_left_out_of_scoring_and_ranking() {
    let mut game = common::playing(3, Rules::default());
    // Le joueur 2 a une grille très basse, mais il est éliminé.
    set_turn(&mut game, 2);
    game.players[2].missed_turns = MAX_MISSED_TURNS - 1;
    game.skip_absent(2).unwrap();
    game.players[2].grid = hidden_grid(&[[-2, -2, 0], [-1, -1, 0], [0, 0, -2], [-1, 0, 0]]);
    game.players[0].total_score = 95;
    game.players[1].total_score = 90;
    common::play_round_where_zero_finishes(&mut game, &[common::FINISHER_5, common::HIDDEN_30]);
    let round = game.rounds.last().unwrap();
    // Le finisseur (5) n'est comparé qu'au joueur 1 (30) : il n'est pas doublé.
    assert_eq!(
        round.scores.iter().map(|s| s.player).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(round.scores[0].score, 5);
    // 100 et 120 : fin de partie ; le joueur éliminé ne peut pas gagner.
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![0]);
}

#[test]
fn an_absent_player_loses_their_last_turn_and_the_round_still_ends() {
    let mut game = common::playing(3, Rules::default());
    game.players[0].grid = common::visible_grid(&common::FINISHER_5);
    game.players[0].grid.cell_mut(3, 2).unwrap().face_up = false;
    game.players[1].grid = hidden_grid(&common::HIDDEN_30);
    game.players[2].grid = hidden_grid(&common::HIDDEN_72);
    game.players[1].missed_turns = MAX_MISSED_TURNS - 1;
    game.draw_pile = vec![8; 20];
    game.discard = vec![9];
    set_turn(&mut game, 0);
    // Le joueur 0 termine la manche.
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    // Le joueur 1, absent, perd son dernier tour et il est éliminé.
    assert_eq!(game.skip_absent(1), Ok(SkipOutcome::Eliminated));
    assert_eq!(current(&game), 2);
    // Le joueur 2 joue son dernier tour : la manche se termine.
    play_simple_turn(&mut game);
    assert_eq!(game.phase, Phase::RoundOver);
    let players: Vec<usize> = game.rounds[0].scores.iter().map(|s| s.player).collect();
    assert_eq!(players, vec![0, 2], "le joueur éliminé n'est pas compté");
}

#[test]
fn an_absent_player_is_skipped_during_the_initial_reveal() {
    let mut game = common::lobby(3, Rules::default());
    game.start(0).unwrap();
    game.apply(0, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    game.apply(0, Action::RevealInitial { column: 0, row: 1 })
        .unwrap();
    game.apply(1, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    game.apply(1, Action::RevealInitial { column: 0, row: 1 })
        .unwrap();
    // Le joueur 2 est parti avant de retourner ses cartes : le serveur les retourne à sa place.
    assert_eq!(game.skip_absent(2), Ok(SkipOutcome::Skipped));
    assert_eq!(game.players[2].initial_reveals, 2);
    assert_eq!(game.players[2].grid.hidden_count(), 10);
    assert_eq!(game.players[2].missed_turns, 1);
    assert!(matches!(game.phase, Phase::Playing(_)));
    // Un joueur qui a déjà retourné ses cartes ne peut pas être « passé » pendant la mise en place.
    let mut other = common::lobby(2, Rules::default());
    other.start(0).unwrap();
    other
        .apply(0, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    other
        .apply(0, Action::RevealInitial { column: 0, row: 1 })
        .unwrap();
    assert_eq!(other.skip_absent(0), Err(GameError::InitialRevealsDone));
}

#[test]
fn a_player_eliminated_during_the_initial_reveal_never_starts() {
    let mut game = common::lobby(3, Rules::default());
    game.start(0).unwrap();
    // Le joueur 2 aurait la plus grande somme, mais il est éliminé pendant la mise en place.
    game.players[2].grid = hidden_grid(&[[12, 12, 0], [0, 1, 2], [2, 1, 0], [0, 1, 2]]);
    game.players[2].missed_turns = MAX_MISSED_TURNS - 1;
    for p in 0..2 {
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    assert_eq!(game.skip_absent(2), Ok(SkipOutcome::Eliminated));
    assert!(matches!(game.phase, Phase::Playing(_)));
    assert_ne!(current(&game), 2);
}
