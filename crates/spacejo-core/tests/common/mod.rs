//! Outils partagés par les tests d'intégration du core.
//! Chaque fichier de `tests/` est compilé séparément : certains outils ne servent
//! pas partout, d'où l'attribut `dead_code`.
#![allow(dead_code)]

use spacejo_core::{Action, Cell, Column, GRID_ROWS, Game, Grid, Phase, PlayerId, Rules, Turn};

/// Graine fixe : les tests sont déterministes.
pub const SEED: u64 = 42;

/// Partie au salon avec `n` joueurs (« P0 » est l'hôte).
pub fn lobby(n: usize, rules: Rules) -> Game {
    let mut game = Game::new("P0", rules, SEED).unwrap();
    for i in 1..n {
        game.join(&format!("P{i}")).unwrap();
    }
    game
}

/// Partie lancée, chaque joueur ayant retourné ses 2 premières cartes de la colonne 0.
pub fn playing(n: usize, rules: Rules) -> Game {
    let mut game = lobby(n, rules);
    game.start(0).unwrap();
    for p in 0..n {
        game.apply(p, Action::RevealInitial { column: 0, row: 0 })
            .unwrap();
        game.apply(p, Action::RevealInitial { column: 0, row: 1 })
            .unwrap();
    }
    assert!(matches!(game.phase, Phase::Playing(_)));
    game
}

/// Grille construite colonne par colonne, toutes les cartes cachées.
pub fn hidden_grid(columns: &[[i32; GRID_ROWS]]) -> Grid {
    Grid {
        columns: columns
            .iter()
            .map(|col| {
                Column::Present(
                    col.iter()
                        .map(|&value| Cell {
                            value,
                            face_up: false,
                        })
                        .collect(),
                )
            })
            .collect(),
    }
}

/// Même chose, toutes les cartes visibles.
pub fn visible_grid(columns: &[[i32; GRID_ROWS]]) -> Grid {
    let mut grid = hidden_grid(columns);
    for column in &mut grid.columns {
        if let Column::Present(cells) = column {
            for cell in cells {
                cell.face_up = true;
            }
        }
    }
    grid
}

/// Force le tour d'un joueur (sans carte en main, sans finisseur).
pub fn set_turn(game: &mut Game, current: PlayerId) {
    game.phase = Phase::Playing(Turn {
        current,
        held: None,
        finisher: None,
    });
}

/// Joueur courant, ou panique si la partie n'est pas en phase de jeu.
pub fn current(game: &Game) -> PlayerId {
    match &game.phase {
        Phase::Playing(turn) => turn.current,
        other => panic!("pas en phase de jeu : {other:?}"),
    }
}

/// Joue un tour simple : piocher puis défausser et retourner la première carte cachée.
/// Si le joueur n'a plus de carte cachée, il échange avec la première carte présente.
pub fn play_simple_turn(game: &mut Game) {
    let p = current(game);
    game.apply(p, Action::DrawFromPile).unwrap();
    match game.players[p].grid.first_hidden() {
        Some((column, row)) => game
            .apply(p, Action::DiscardHeldAndReveal { column, row })
            .unwrap(),
        None => {
            let (column, row) = game.players[p].grid.first_present().unwrap();
            game.apply(p, Action::SwapHeld { column, row }).unwrap()
        }
    }
}

/// Grille du finisseur type : tout est visible sauf la dernière carte (-1). Somme = 5.
pub const FINISHER_5: [[i32; GRID_ROWS]; 4] = [[0, 1, 0], [1, 0, 1], [0, 2, 0], [1, 0, -1]];
/// Grille cachée de somme 30 (aucune colonne identique).
pub const HIDDEN_30: [[i32; GRID_ROWS]; 4] = [[2, 3, 4], [2, 3, 4], [1, 2, 3], [1, 2, 3]];
/// Grille cachée de somme 72 (aucune colonne identique).
pub const HIDDEN_72: [[i32; GRID_ROWS]; 4] = [[5, 6, 7], [5, 6, 7], [5, 6, 7], [5, 6, 7]];

/// Joue une manche entière que le joueur 0 termine : il reçoit `grids[0]` (tout visible sauf
/// la dernière carte), les autres `grids[p]` face cachée. Les autres jouent ensuite leur
/// dernier tour avec `play_simple_turn`, qui ne change pas la somme de leur grille.
pub fn play_round_where_zero_finishes(game: &mut Game, grids: &[[[i32; GRID_ROWS]; 4]]) {
    game.players[0].grid = visible_grid(&grids[0]);
    game.players[0].grid.cell_mut(3, 2).unwrap().face_up = false;
    for (p, columns) in grids.iter().enumerate().skip(1) {
        game.players[p].grid = hidden_grid(columns);
    }
    game.draw_pile = vec![8; 30];
    game.discard = vec![9];
    set_turn(game, 0);
    game.apply(0, Action::DrawFromPile).unwrap();
    game.apply(0, Action::DiscardHeldAndReveal { column: 3, row: 2 })
        .unwrap();
    while matches!(game.phase, Phase::Playing(_)) {
        play_simple_turn(game);
    }
}

/// Fin de manche : tous les joueurs encore en jeu votent pour la manche suivante, puis celui qui décide (l'hôte, ou à
/// défaut le premier joueur encore en jeu) la lance. Personne n'est donc laissé de côté.
pub fn everyone_starts_next_round(game: &mut Game) {
    let decider = game.next_round_decider().unwrap();
    for p in game.active_players() {
        if p != decider {
            game.apply(p, Action::NextRound).unwrap();
        }
    }
    game.apply(decider, Action::NextRound).unwrap();
}
