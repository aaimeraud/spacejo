//! Quitter le salon, être retiré par l'hôte, abandonner une partie lancée (PLAN.md, ASSUMPTIONS.md n° 26 à 28).
mod common;

use common::{play_simple_turn, set_turn};
use spacejo_core::{Action, Game, GameError, HOST, INITIAL_REVEALS, Phase, Rules, view_for};

fn names(game: &Game) -> Vec<&str> {
    game.players.iter().map(|p| p.name.as_str()).collect()
}

// ----- Salon -----

#[test]
fn a_player_who_leaves_the_lobby_is_removed_and_the_others_move_up() {
    let mut game = common::lobby(4, Rules::default());
    let before = names(&game)
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>();
    game.leave_lobby(1).unwrap();
    assert_eq!(names(&game), vec![&before[0], &before[2], &before[3]]);
    assert_eq!(game.phase, Phase::Lobby);
}

#[test]
fn when_the_host_leaves_the_lobby_the_next_player_becomes_host() {
    let mut game = common::lobby(3, Rules::default());
    let second = game.players[1].name.clone();
    game.leave_lobby(HOST).unwrap();
    assert_eq!(game.players.len(), 2);
    assert_eq!(game.players[HOST].name, second);
}

#[test]
fn the_last_player_cannot_be_removed_from_the_lobby() {
    let mut game = common::lobby(1, Rules::default());
    assert_eq!(game.leave_lobby(0), Err(GameError::LastPlayer));
    assert_eq!(game.players.len(), 1);
}

#[test]
fn leaving_the_lobby_is_refused_once_the_game_has_started_or_for_an_unknown_player() {
    let mut game = common::lobby(3, Rules::default());
    assert_eq!(game.leave_lobby(7), Err(GameError::UnknownPlayer));
    game.start(HOST).unwrap();
    assert_eq!(game.leave_lobby(1), Err(GameError::WrongPhase));
}

#[test]
fn only_the_host_kicks_and_never_himself() {
    let mut game = common::lobby(3, Rules::default());
    assert_eq!(game.kick(1, 2), Err(GameError::NotHost));
    assert_eq!(game.kick(HOST, HOST), Err(GameError::CannotKickHost));
    assert_eq!(game.kick(HOST, 9), Err(GameError::UnknownPlayer));
    let third = game.players[2].name.clone();
    game.kick(HOST, 1).unwrap();
    assert_eq!(game.players.len(), 2);
    assert_eq!(game.players[1].name, third);
}

#[test]
fn kicking_is_refused_after_the_start() {
    let mut game = common::lobby(3, Rules::default());
    game.start(HOST).unwrap();
    assert_eq!(game.kick(HOST, 1), Err(GameError::WrongPhase));
}

// ----- Partie lancée -----

#[test]
fn abandoning_eliminates_the_player_and_marks_it_for_everyone() {
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 0);
    game.abandon(2).unwrap();
    assert!(game.players[2].abandoned);
    assert!(game.players[2].eliminated);
    // Tout le monde le voit dans sa vue.
    for viewer in 0..3 {
        let view = serde_json::to_value(view_for(&game, viewer)).unwrap();
        assert_eq!(view["players"][2]["abandoned"], true);
    }
    // Il ne joue plus ; le tour saute sa place.
    assert_eq!(
        game.apply(2, Action::DrawFromPile),
        Err(GameError::Eliminated)
    );
    play_simple_turn(&mut game);
    play_simple_turn(&mut game);
    assert_eq!(
        common::current(&game),
        0,
        "le tour revient à 0 sans passer par 2"
    );
}

#[test]
fn abandoning_during_ones_turn_discards_the_held_card_and_passes_the_turn() {
    let mut game = common::playing(3, Rules::default());
    set_turn(&mut game, 1);
    game.apply(1, Action::DrawFromPile).unwrap();
    let before = game.card_count();
    game.abandon(1).unwrap();
    assert_eq!(common::current(&game), 2);
    assert_eq!(game.card_count(), before, "aucune carte ne disparaît");
}

#[test]
fn when_only_one_player_remains_he_wins() {
    let mut game = common::playing(2, Rules::default());
    game.abandon(0).unwrap();
    assert_eq!(game.phase, Phase::GameOver);
    assert_eq!(game.winners, vec![1]);
}

#[test]
fn abandoning_during_the_initial_reveal_lets_the_round_start() {
    let mut game = common::lobby(3, Rules::default());
    game.start(HOST).unwrap();
    for player in [0, 1] {
        for (column, row) in [(0, 0), (1, 0)].into_iter().take(INITIAL_REVEALS) {
            game.apply(player, Action::RevealInitial { column, row })
                .unwrap();
        }
    }
    assert_eq!(
        game.phase,
        Phase::InitialReveal,
        "on attend encore le joueur 2"
    );
    game.abandon(2).unwrap();
    assert!(matches!(game.phase, Phase::Playing(_)));
    assert_ne!(common::current(&game), 2);
}

#[test]
fn abandoning_between_rounds_keeps_the_player_out_of_the_next_round() {
    let mut game = common::playing(3, Rules::default());
    game.phase = Phase::RoundOver;
    game.abandon(1).unwrap();
    assert_eq!(game.phase, Phase::RoundOver);
    game.apply(0, Action::NextRound).unwrap();
    assert!(game.players[1].eliminated);
    assert_eq!(
        game.players[1].grid,
        Default::default(),
        "il ne reçoit plus de cartes"
    );
}

#[test]
fn abandoning_is_refused_in_the_lobby_after_the_end_or_twice() {
    let mut game = common::lobby(2, Rules::default());
    assert_eq!(game.abandon(1), Err(GameError::WrongPhase));
    let mut game = common::playing(3, Rules::default());
    game.abandon(1).unwrap();
    assert_eq!(game.abandon(1), Err(GameError::Eliminated));
    let mut over = common::playing(2, Rules::default());
    over.abandon(1).unwrap();
    assert_eq!(over.abandon(0), Err(GameError::WrongPhase));
}
