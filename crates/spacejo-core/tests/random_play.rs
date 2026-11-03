//! Critère 13 : des parties complètes jouées au hasard (coups toujours légaux, graine fixée)
//! se terminent toujours, et le nombre total de cartes reste 150 à chaque étape.
use spacejo_core::{
    Action, EmptyPile, EndTie, FirstPlayerTie, Game, NextRoundStarter, PenaltyRule, PenaltyScope,
    Phase, Rules, SplitMix64,
};

/// Garde-fou : une partie qui dépasse ce nombre d'actions est considérée comme bloquée.
const MAX_ACTIONS: usize = 200_000;

/// Joue une partie entière avec des coups tirés au hasard parmi les coups légaux.
/// Renvoie le nombre d'actions jouées.
fn play_random_game(players: usize, rules: Rules, seed: u64) -> usize {
    let mut game = Game::new("P0", rules, seed).unwrap();
    for i in 1..players {
        game.join(&format!("P{i}")).unwrap();
    }
    game.start(0).unwrap();
    // Générateur séparé pour choisir les coups (la partie a le sien).
    let mut chooser = SplitMix64::new(seed ^ 0xDEAD_BEEF);
    let mut actions = 0;
    while game.phase != Phase::GameOver {
        assert!(actions < MAX_ACTIONS, "partie bloquée (graine {seed})");
        assert_eq!(game.card_count(), 150, "cartes perdues (graine {seed})");
        // Tous les coups légaux de tous les joueurs (pendant la mise en place, plusieurs
        // joueurs peuvent jouer).
        let mut legal: Vec<(usize, Action)> = Vec::new();
        for p in 0..players {
            for action in game.legal_actions(p) {
                legal.push((p, action));
            }
        }
        assert!(!legal.is_empty(), "aucun coup légal (graine {seed})");
        let (p, action) = legal[chooser.below(legal.len())];
        game.apply(p, action)
            .unwrap_or_else(|e| panic!("coup légal refusé : {action:?} → {e:?}"));
        actions += 1;
    }
    assert_eq!(game.card_count(), 150);
    assert!(!game.winners.is_empty());
    // Les gagnants ont bien le total le plus bas.
    let lowest = game.players.iter().map(|p| p.total_score).min().unwrap();
    for &w in &game.winners {
        assert_eq!(game.players[w].total_score, lowest);
    }
    actions
}

#[test]
fn random_games_with_default_rules_always_finish() {
    for seed in 0..30 {
        for players in 2..=4 {
            play_random_game(players, Rules::default(), seed);
        }
    }
}

#[test]
fn random_games_with_variants_always_finish() {
    let variants = [
        Rules {
            penalty: PenaltyRule::DoubleOnlyIfBeaten,
            penalty_scope: PenaltyScope::PositiveCardsOnly,
            first_player_tie: FirstPlayerTie::Random,
            next_round_starter: NextRoundStarter::RoundWinner,
            empty_pile: EmptyPile::EndRound,
            end_tie: EndTie::ExtraRound,
            end_score: 50,
        },
        Rules {
            penalty: PenaltyRule::Disabled,
            next_round_starter: NextRoundStarter::Rotate,
            end_tie: EndTie::ExtraRound,
            end_score: 10,
            ..Rules::default()
        },
    ];
    for rules in variants {
        for seed in 0..20 {
            for players in 2..=4 {
                play_random_game(players, rules, seed);
            }
        }
    }
}

#[test]
fn the_same_seed_replays_the_same_game() {
    let a = play_random_game(3, Rules::default(), 1234);
    let b = play_random_game(3, Rules::default(), 1234);
    assert_eq!(a, b);
}
