//! Critère 7 : pénalité du finisseur, pour chaque `PenaltyRule` et chaque `PenaltyScope`.
use spacejo_core::{PenaltyRule, PenaltyScope, Rules, finisher_round_score};

fn rules(penalty: PenaltyRule, penalty_scope: PenaltyScope) -> Rules {
    Rules {
        penalty,
        penalty_scope,
        ..Rules::default()
    }
}

// Le finisseur a {4, 6} = 10 dans tous les cas ci-dessous.
const CARDS: [i32; 2] = [4, 6];

#[test]
fn double_unless_strictly_lowest() {
    let r = rules(
        PenaltyRule::DoubleUnlessStrictlyLowest,
        PenaltyScope::RoundTotal,
    );
    assert_eq!(
        finisher_round_score(&CARDS, &[10, 20], &r),
        20,
        "égalité : doublé"
    );
    assert_eq!(
        finisher_round_score(&CARDS, &[9, 20], &r),
        20,
        "battu : doublé"
    );
    assert_eq!(
        finisher_round_score(&CARDS, &[11, 20], &r),
        10,
        "seul le plus bas : pas doublé"
    );
}

#[test]
fn double_only_if_beaten() {
    let r = rules(PenaltyRule::DoubleOnlyIfBeaten, PenaltyScope::RoundTotal);
    assert_eq!(
        finisher_round_score(&CARDS, &[10, 20], &r),
        10,
        "égalité : épargné"
    );
    assert_eq!(
        finisher_round_score(&CARDS, &[9, 20], &r),
        20,
        "battu : doublé"
    );
    assert_eq!(
        finisher_round_score(&CARDS, &[11, 20], &r),
        10,
        "seul le plus bas : pas doublé"
    );
}

#[test]
fn disabled() {
    let r = rules(PenaltyRule::Disabled, PenaltyScope::RoundTotal);
    assert_eq!(finisher_round_score(&CARDS, &[10, 20], &r), 10, "égalité");
    assert_eq!(finisher_round_score(&CARDS, &[9, 20], &r), 10, "battu");
    assert_eq!(
        finisher_round_score(&CARDS, &[11, 20], &r),
        10,
        "seul le plus bas"
    );
}

#[test]
fn zero_or_negative_totals_are_never_doubled() {
    for scope in [PenaltyScope::RoundTotal, PenaltyScope::PositiveCardsOnly] {
        for penalty in [
            PenaltyRule::DoubleUnlessStrictlyLowest,
            PenaltyRule::DoubleOnlyIfBeaten,
        ] {
            let r = rules(penalty, scope);
            // Total nul, battu par -5 : pas doublé.
            assert_eq!(finisher_round_score(&[3, -2, -1], &[-5], &r), 0);
            // Total négatif, battu par -9 : pas doublé.
            assert_eq!(finisher_round_score(&[2, -2, -2], &[-9], &r), -2);
        }
    }
}

#[test]
fn penalty_scope_on_the_reference_example() {
    // Cartes {10, -2, 4}, total 12, battu par 5.
    let cards = [10, -2, 4];
    let total = rules(
        PenaltyRule::DoubleUnlessStrictlyLowest,
        PenaltyScope::RoundTotal,
    );
    let positives = rules(
        PenaltyRule::DoubleUnlessStrictlyLowest,
        PenaltyScope::PositiveCardsOnly,
    );
    assert_eq!(finisher_round_score(&cards, &[5], &total), 24);
    assert_eq!(finisher_round_score(&cards, &[5], &positives), 26);
}

#[test]
fn alone_the_finisher_is_never_doubled() {
    let r = Rules::default();
    assert_eq!(finisher_round_score(&CARDS, &[], &r), 10);
}
