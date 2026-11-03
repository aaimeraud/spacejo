//! Critère 14 : `next_round_starter` désigne le bon joueur pour la manche suivante.
mod common;

use common::{FINISHER_5, HIDDEN_30, HIDDEN_72, hidden_grid, play_round_where_zero_finishes};
use spacejo_core::{Action, GRID_ROWS, Game, NextRoundStarter, PenaltyRule, PlayerId, Rules};

/// Grille cachée de somme 3, sans colonne identique.
const HIDDEN_3: [[i32; GRID_ROWS]; 4] = [[1, 2, 0], [0, -1, 0], [1, 0, -1], [0, 1, 0]];

fn rules(next_round_starter: NextRoundStarter) -> Rules {
    Rules {
        next_round_starter,
        ..Rules::default()
    }
}

/// Lance la manche suivante, donne au joueur `highest` la plus grande somme de cartes
/// retournées (pour vérifier que la règle de la première manche n'est PAS utilisée),
/// fait retourner 2 cartes à chacun et renvoie le joueur qui commence.
fn start_next_round(game: &mut Game, highest: PlayerId) -> PlayerId {
    game.apply(0, Action::NextRound).unwrap();
    for p in 0..game.players.len() {
        let top = if p == highest { 12 } else { 1 };
        game.players[p].grid = hidden_grid(&[[top, top, 0], [0, 1, 2], [2, 1, 0], [0, 1, 2]]);
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    common::current(game)
}

#[test]
fn finisher_starts_the_next_round() {
    let mut game = common::playing(3, rules(NextRoundStarter::Finisher));
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_72]);
    assert_eq!(start_next_round(&mut game, 2), 0);
    assert_eq!(game.round_starter, Some(0));
}

#[test]
fn round_winner_starts_the_next_round() {
    let mut game = common::playing(3, rules(NextRoundStarter::RoundWinner));
    // Le finisseur (5) est le plus bas : il gagne la manche.
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_72, HIDDEN_30]);
    assert_eq!(start_next_round(&mut game, 2), 0);
}

#[test]
fn round_winner_uses_the_score_after_penalty_and_join_order_on_ties() {
    let mut game = common::playing(3, rules(NextRoundStarter::RoundWinner));
    // Finisseur à 5, doublé à 10 car battu ; joueurs 1 et 2 à égalité à 3 → le joueur 1 (arrivé avant).
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_3, HIDDEN_3]);
    let scores: Vec<i32> = game.rounds[0].scores.iter().map(|s| s.score).collect();
    assert_eq!(scores, vec![10, 3, 3]);
    assert_eq!(start_next_round(&mut game, 2), 1);
}

#[test]
fn round_winner_tie_with_the_finisher_when_penalty_is_disabled() {
    let rules = Rules {
        next_round_starter: NextRoundStarter::RoundWinner,
        penalty: PenaltyRule::Disabled,
        ..Rules::default()
    };
    let mut game = common::playing(3, rules);
    // 5, 3, 3 : égalité entre 1 et 2 → le joueur 1.
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_3, HIDDEN_3]);
    assert_eq!(start_next_round(&mut game, 2), 1);
}

#[test]
fn rotate_gives_the_start_to_the_next_player_each_round() {
    let mut game = common::playing(3, rules(NextRoundStarter::Rotate));
    let first = game.round_starter.unwrap();

    // HIDDEN_30 pour les deux autres : personne n'atteint 100 en deux manches.
    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_30]);
    let expected = (first + 1) % 3;
    let highest = if expected == 2 { 0 } else { 2 };
    assert_eq!(start_next_round(&mut game, highest), expected);

    play_round_where_zero_finishes(&mut game, &[FINISHER_5, HIDDEN_30, HIDDEN_30]);
    let expected = (first + 2) % 3;
    let highest = if expected == 2 { 0 } else { 2 };
    assert_eq!(start_next_round(&mut game, highest), expected);
}
