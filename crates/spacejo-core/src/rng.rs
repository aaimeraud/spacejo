//! Générateur pseudo-aléatoire initialisable par une graine.
//!
//! Pourquoi un générateur écrit ici plutôt qu'une caisse comme `rand` ? Pour que les tests
//! restent identiques quelle que soit la version des dépendances (une mise à jour de `rand`
//! peut changer l'ordre d'un mélange), et pour que le core n'ait aucune dépendance au
//! système. Voir `docs/decisions/2026-11-03-generateur-aleatoire.md`.
//!
//! L'algorithme est SplitMix64 (Steele, Lea et Flood, 2014) : très court, rapide, et de bonne
//! qualité statistique pour mélanger des cartes. Il n'est PAS cryptographique : c'est l'API qui
//! fournit la graine, tirée de l'entropie du système, et elle n'est jamais exposée.

use serde::{Deserialize, Serialize};

/// État du générateur. Il est enregistré avec la partie, pour que la suite des tirages
/// continue d'une requête à l'autre (les fonctions serverless ne gardent rien en mémoire).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitMix64 {
    state: u64,
}

// `Debug` écrit à la main : on n'affiche jamais l'état, qui permettrait de prédire les
// tirages (règle d'hygiène des journaux de CLAUDE.md).
// Traits : https://doc.rust-lang.org/stable/book/ch10-02-traits.html
impl std::fmt::Debug for SplitMix64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SplitMix64 { .. }")
    }
}

impl SplitMix64 {
    /// Crée un générateur à partir d'une graine. Même graine → mêmes tirages.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Tire le prochain nombre de 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        // `wrapping_*` : le dépassement de capacité est voulu (arithmétique modulo 2^64) ;
        // sans cela, Rust paniquerait en mode debug.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Tire un entier uniforme dans `0..n` (renvoie 0 si `n` vaut 0, pour ne jamais paniquer).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        let n = n as u64;
        // Rejet des valeurs de la dernière « tranche » incomplète : sans cela, un simple
        // `% n` favoriserait légèrement les petits nombres (biais du modulo).
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return (x % n) as usize;
            }
        }
    }

    /// Mélange une liste sur place (algorithme de Fisher-Yates : chaque ordre est équiprobable).
    /// `&mut [T]` emprunte la liste en écriture sans en prendre la propriété :
    /// https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}
