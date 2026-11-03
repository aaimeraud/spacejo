//! Calcul des scores de manche. Toute règle de score lit `Rules` : rien n'est en dur ici.

use crate::cards::CardValue;
use crate::rules::{PenaltyRule, PenaltyScope, Rules};

/// Score de manche du joueur qui a terminé la manche (le « finisseur »).
///
/// - `cards` : valeurs des cartes restantes du finisseur ;
/// - `others_raw` : totaux de manche (sans pénalité) des autres joueurs.
///
/// Implémentation de référence donnée par la demande (docs/RULES.md, « Pénalité »).
pub fn finisher_round_score(cards: &[CardValue], others_raw: &[i32], rules: &Rules) -> i32 {
    let raw: i32 = cards.iter().sum();
    // `min()` renvoie une `Option` : `None` s'il n'y a aucun autre joueur.
    let best_other = others_raw.iter().copied().min();
    // `match` sur un couple : on traite chaque combinaison (règle, meilleur adversaire).
    // https://doc.rust-lang.org/stable/book/ch06-02-match.html
    let penalized = match (rules.penalty, best_other) {
        (_, None) | (PenaltyRule::Disabled, _) => false,
        (PenaltyRule::DoubleUnlessStrictlyLowest, Some(o)) => raw >= o,
        (PenaltyRule::DoubleOnlyIfBeaten, Some(o)) => raw > o,
    };
    // Officiel : un total nul ou négatif n'est jamais doublé.
    if !penalized || raw <= 0 {
        return raw;
    }
    match rules.penalty_scope {
        PenaltyScope::RoundTotal => raw * 2,
        PenaltyScope::PositiveCardsOnly => {
            let positives: i32 = cards.iter().copied().filter(|v| *v > 0).sum();
            raw + positives // les cartes positives comptent deux fois, les négatives une fois
        }
    }
}
