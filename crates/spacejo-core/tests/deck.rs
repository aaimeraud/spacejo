//! Critère 1 : 150 cartes, bonne répartition, mélange reproductible avec une graine.
mod common;

use spacejo_core::{DECK_SIZE, Rules, SplitMix64, full_deck};

#[test]
fn deck_has_150_cards_with_the_official_distribution() {
    let deck = full_deck();
    assert_eq!(deck.len(), 150);
    assert_eq!(DECK_SIZE, 150);
    let count = |v: i32| deck.iter().filter(|&&c| c == v).count();
    assert_eq!(count(-2), 5);
    assert_eq!(count(-1), 10);
    assert_eq!(count(0), 15);
    for value in 1..=12 {
        assert_eq!(count(value), 10, "valeur {value}");
    }
    // Aucune autre valeur.
    assert!(deck.iter().all(|&c| (-2..=12).contains(&c)));
}

#[test]
fn shuffle_with_the_same_seed_gives_the_same_order() {
    let mut a = full_deck();
    let mut b = full_deck();
    SplitMix64::new(7).shuffle(&mut a);
    SplitMix64::new(7).shuffle(&mut b);
    assert_eq!(a, b);
    // Le mélange a vraiment changé l'ordre…
    assert_ne!(a, full_deck());
    // … et une autre graine donne un autre ordre.
    let mut c = full_deck();
    SplitMix64::new(8).shuffle(&mut c);
    assert_ne!(a, c);
}

#[test]
fn a_started_game_holds_exactly_150_cards() {
    let mut game = common::lobby(4, Rules::default());
    game.start(0).unwrap();
    assert_eq!(game.card_count(), 150);
    for player in &game.players {
        assert_eq!(player.grid.card_count(), 12);
    }
    assert_eq!(game.discard.len(), 1);
    assert_eq!(game.draw_pile.len(), 150 - 4 * 12 - 1);
}

#[test]
fn two_games_with_the_same_seed_are_dealt_identically() {
    let mut a = common::lobby(3, Rules::default());
    let mut b = common::lobby(3, Rules::default());
    a.start(0).unwrap();
    b.start(0).unwrap();
    assert_eq!(a, b);
}
