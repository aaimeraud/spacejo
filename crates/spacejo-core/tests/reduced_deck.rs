//! Réglage `deck` : pioche réduite, proportionnelle au nombre de joueurs (docs/RULES.md, « Règles maison »).
mod common;

use std::collections::HashMap;

use common::current;
use spacejo_core::Action;
use spacejo_core::{
    CARDS_PER_PLAYER, DECK_COMPOSITION, DECK_SIZE, DeckSize, Game, Phase,
    REDUCED_DECK_CARDS_PER_PLAYER, Rules,
};

fn reduced() -> Rules {
    Rules {
        deck: DeckSize::Reduced,
        ..Rules::default()
    }
}

/// Toutes les cartes de la partie (grilles, pioche, défausse), comptées par valeur.
fn counts(game: &Game) -> HashMap<i32, usize> {
    let mut counts = HashMap::new();
    let grids = game.players.iter().flat_map(|p| p.grid.values());
    for value in grids
        .chain(game.draw_pile.iter().copied())
        .chain(game.discard.iter().copied())
    {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

#[test]
fn the_official_deck_is_the_default() {
    assert_eq!(Rules::default().deck, DeckSize::Full);
    let mut game = common::lobby(2, Rules::default());
    game.start(0).unwrap();
    assert_eq!(game.card_count(), DECK_SIZE);
}

#[test]
fn a_reduced_deck_has_forty_cards_per_player() {
    assert_eq!(REDUCED_DECK_CARDS_PER_PLAYER, 40);
    // À partir de 4 joueurs, 40 cartes chacun dépasseraient les 150 : le paquet entier est alors gardé.
    for players in [2, 3, 4, 7] {
        let mut game = common::lobby(players, reduced());
        game.start(0).unwrap();
        let total = (players * REDUCED_DECK_CARDS_PER_PLAYER).min(DECK_SIZE);
        assert_eq!(game.card_count(), total, "{players} joueurs");
        // 12 cartes dans chaque grille, 1 sur la défausse, le reste dans la pioche.
        assert_eq!(game.draw_pile.len(), total - players * CARDS_PER_PLAYER - 1);
    }
}

#[test]
fn a_reduced_deck_never_exceeds_the_full_deck() {
    let mut game = common::lobby(10, reduced());
    game.start(0).unwrap();
    assert_eq!(game.card_count(), DECK_SIZE);
}

#[test]
fn the_kept_cards_are_taken_from_the_full_deck() {
    let mut game = common::lobby(3, reduced());
    game.start(0).unwrap();
    let counts = counts(&game);
    for (value, count) in counts {
        let available = DECK_COMPOSITION
            .iter()
            .find(|(v, _)| *v == value)
            .map(|(_, n)| *n)
            .unwrap_or(0);
        assert!(count <= available, "{count} cartes {value} sur {available}");
    }
}

#[test]
fn the_same_seed_gives_the_same_reduced_deck() {
    let mut first = common::lobby(2, reduced());
    let mut second = common::lobby(2, reduced());
    first.start(0).unwrap();
    second.start(0).unwrap();
    assert_eq!(first.draw_pile, second.draw_pile);
}

#[test]
fn with_a_reduced_deck_the_discard_pile_is_reshuffled_during_the_round() {
    let mut game = common::playing(2, reduced());
    let total = game.card_count();
    let mut reshuffled = false;
    // Chaque tour, le joueur pioche et échange la carte avec sa première carte présente : il ne retourne jamais de
    // carte cachée, donc la manche dure assez longtemps pour vider la pioche (55 cartes à 2 joueurs).
    let mut turns = 0;
    while matches!(game.phase, Phase::Playing(_)) && !reshuffled && turns < 200 {
        turns += 1;
        let before = game.draw_pile.len();
        play_swap_turn(&mut game);
        // Aucune carte ne disparaît ni n'apparaît.
        assert_eq!(game.card_count(), total);
        if game.draw_pile.len() > before {
            reshuffled = true;
        }
    }
    assert!(reshuffled, "la défausse aurait dû redevenir la pioche");
}

/// Un tour sans retourner de carte cachée : piocher, puis échanger avec la première carte présente.
fn play_swap_turn(game: &mut Game) {
    let p = current(game);
    game.apply(p, Action::DrawFromPile).unwrap();
    let (column, row) = game.players[p].grid.first_present().unwrap();
    game.apply(p, Action::SwapHeld { column, row }).unwrap();
}

#[test]
fn each_round_is_dealt_again_from_a_reduced_deck() {
    let mut game = common::playing(3, reduced());
    game.phase = Phase::RoundOver;
    game.apply(0, spacejo_core::Action::NextRound).unwrap();
    assert_eq!(game.card_count(), 3 * REDUCED_DECK_CARDS_PER_PLAYER);
}
