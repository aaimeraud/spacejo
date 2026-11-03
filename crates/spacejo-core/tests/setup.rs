//! Critère 3 : mise en place, 2 cartes retournées par joueur, premier joueur et égalités.
mod common;

use common::hidden_grid;
use spacejo_core::{Action, FirstPlayerTie, Game, GameError, Phase, Rules};

/// Partie lancée où chaque joueur a une grille choisie : la colonne 0 contient
/// `top[p]` et `second[p]` sur ses deux premières lignes, le reste vaut 0.
fn started_with_first_column(values: &[(i32, i32)], rules: Rules) -> Game {
    let mut game = common::lobby(values.len(), rules);
    game.start(0).unwrap();
    for (p, &(a, b)) in values.iter().enumerate() {
        game.players[p].grid = hidden_grid(&[[a, b, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]]);
    }
    game
}

fn reveal_two(game: &mut Game) {
    for p in 0..game.players.len() {
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
}

#[test]
fn every_card_starts_hidden_and_each_player_reveals_exactly_two() {
    let mut game = common::lobby(3, Rules::default());
    game.start(0).unwrap();
    assert_eq!(game.phase, Phase::InitialReveal);
    for player in &game.players {
        assert_eq!(player.grid.hidden_count(), 12);
    }
    // Les joueurs retournent dans n'importe quel ordre (pas de tour pendant cette phase).
    game.apply(2, Action::RevealInitial { column: 3, row: 2 })
        .unwrap();
    game.apply(0, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    game.apply(0, Action::RevealInitial { column: 1, row: 0 })
        .unwrap();
    // Un troisième retournement est refusé.
    assert_eq!(
        game.apply(0, Action::RevealInitial { column: 2, row: 0 }),
        Err(GameError::InitialRevealsDone)
    );
    // Retourner deux fois la même carte est refusé.
    assert_eq!(
        game.apply(2, Action::RevealInitial { column: 3, row: 2 }),
        Err(GameError::AlreadyRevealed)
    );
    // Position hors de la grille.
    assert_eq!(
        game.apply(1, Action::RevealInitial { column: 4, row: 0 }),
        Err(GameError::InvalidPosition)
    );
    game.apply(1, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    assert_eq!(
        game.phase,
        Phase::InitialReveal,
        "le joueur 1 n'a retourné qu'une carte"
    );
    game.apply(1, Action::RevealInitial { column: 0, row: 1 })
        .unwrap();
    game.apply(2, Action::RevealInitial { column: 0, row: 0 })
        .unwrap();
    assert!(matches!(game.phase, Phase::Playing(_)));
    for player in &game.players {
        assert_eq!(player.grid.hidden_count(), 10);
        assert_eq!(player.initial_reveals, 2);
    }
}

#[test]
fn the_highest_sum_starts() {
    let mut game = started_with_first_column(&[(1, 2), (12, -2), (5, 6)], Rules::default());
    reveal_two(&mut game);
    // Sommes : 3, 10, 11 → le joueur 2 commence.
    assert_eq!(common::current(&game), 2);
    assert_eq!(game.round_starter, Some(2));
}

#[test]
fn tie_with_join_order_goes_to_the_first_arrived() {
    let rules = Rules {
        first_player_tie: FirstPlayerTie::JoinOrder,
        ..Rules::default()
    };
    let mut game = started_with_first_column(&[(1, 1), (4, 6), (5, 5)], rules);
    reveal_two(&mut game);
    // Joueurs 1 et 2 à égalité (10) : le joueur 1 est arrivé avant.
    assert_eq!(common::current(&game), 1);
}

#[test]
fn tie_with_random_is_reproducible_and_picks_a_tied_player() {
    let rules = Rules {
        first_player_tie: FirstPlayerTie::Random,
        ..Rules::default()
    };
    let values = [(1, 1), (4, 6), (5, 5), (10, 0)];
    let mut a = started_with_first_column(&values, rules);
    let mut b = started_with_first_column(&values, rules);
    reveal_two(&mut a);
    reveal_two(&mut b);
    assert_eq!(
        common::current(&a),
        common::current(&b),
        "même graine, même tirage"
    );
    assert!([1, 2, 3].contains(&common::current(&a)));

    // Sur plusieurs graines, le tirage ne désigne pas toujours le même joueur.
    let mut starters = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let mut game = Game::new("P0", rules, seed).unwrap();
        for i in 1..4 {
            game.join(&format!("P{i}")).unwrap();
        }
        game.start(0).unwrap();
        for (p, &(x, y)) in values.iter().enumerate() {
            game.players[p].grid = hidden_grid(&[[x, y, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]]);
        }
        reveal_two(&mut game);
        starters.insert(common::current(&game));
    }
    assert!(starters.len() > 1, "tirages obtenus : {starters:?}");
    assert!(starters.iter().all(|p| [1, 2, 3].contains(p)));
}
