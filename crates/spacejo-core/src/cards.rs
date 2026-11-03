//! Le paquet de cartes.

/// Valeur d'une carte (de -2 à 12).
pub type CardValue = i32;

/// Composition du paquet (docs/RULES.md, « Cartes ») : (valeur, nombre d'exemplaires).
pub const DECK_COMPOSITION: [(CardValue, usize); 15] = [
    (-2, 5),
    (-1, 10),
    (0, 15),
    (1, 10),
    (2, 10),
    (3, 10),
    (4, 10),
    (5, 10),
    (6, 10),
    (7, 10),
    (8, 10),
    (9, 10),
    (10, 10),
    (11, 10),
    (12, 10),
];

/// Nombre total de cartes : 150.
pub const DECK_SIZE: usize = 150;

/// Construit le paquet complet, non mélangé.
pub fn full_deck() -> Vec<CardValue> {
    // Itérateurs : `flat_map` produit, pour chaque (valeur, nombre), `nombre` copies de la valeur.
    // https://doc.rust-lang.org/stable/book/ch13-02-iterators.html
    DECK_COMPOSITION
        .iter()
        .flat_map(|&(value, count)| std::iter::repeat_n(value, count))
        .collect()
}
