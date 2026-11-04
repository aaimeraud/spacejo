//! Critère 13 : des parties complètes jouées au hasard (coups toujours légaux, graine fixée)
//! se terminent toujours, et le nombre total de cartes reste 150 à chaque étape.
use spacejo_core::{
    Action, DeckSize, EmptyPile, EndTie, FirstPlayerTie, Game, NextRoundStarter, PenaltyRule,
    PenaltyScope, Phase, Rules, SplitMix64,
};

/// Garde-fou : une partie qui dépasse ce nombre d'actions est considérée comme bloquée.
const MAX_ACTIONS: usize = 200_000;

/// Joue une partie entière avec des coups tirés au hasard parmi les coups légaux.
/// Renvoie le nombre d'actions jouées.
fn play_random_game(players: usize, rules: Rules, seed: u64) -> usize {
    play_random_game_with_absences(players, rules, seed, 0)
}

/// Variante : avec une probabilité de `skip_percent` %, le joueur dont c'est le tour (ou qui doit encore
/// retourner ses cartes) est considéré comme absent et son tour est passé par le serveur.
fn play_random_game_with_absences(
    players: usize,
    rules: Rules,
    seed: u64,
    skip_percent: usize,
) -> usize {
    let mut game = Game::new("P0", rules, seed).unwrap();
    for i in 1..players {
        game.join(&format!("P{i}")).unwrap();
    }
    game.start(0).unwrap();
    // Générateur séparé pour choisir les coups (la partie a le sien).
    let mut chooser = SplitMix64::new(seed ^ 0xDEAD_BEEF);
    let mut actions = 0;
    // Aucune carte ne se perd pendant une manche. Avec le paquet complet, il y en a toujours 150 ; avec la pioche
    // réduite, le paquet est refait à chaque manche (40 cartes par joueur encore en jeu).
    let full = rules.deck == DeckSize::Full;
    let mut round = game.round;
    let mut expected = game.card_count();
    while game.phase != Phase::GameOver {
        assert!(actions < MAX_ACTIONS, "partie bloquée (graine {seed})");
        if game.round != round {
            round = game.round;
            expected = game.card_count();
        }
        assert_eq!(
            game.card_count(),
            expected,
            "cartes perdues (graine {seed})"
        );
        if full {
            assert_eq!(expected, 150, "paquet incomplet (graine {seed})");
        }
        // Tous les coups légaux de tous les joueurs (pendant la mise en place, plusieurs
        // joueurs peuvent jouer).
        let mut legal: Vec<(usize, Action)> = Vec::new();
        for p in 0..players {
            for action in game.legal_actions(p) {
                legal.push((p, action));
            }
        }
        assert!(!legal.is_empty(), "aucun coup légal (graine {seed})");
        if skip_percent > 0 && chooser.below(100) < skip_percent {
            let absent = match &game.phase {
                Phase::Playing(turn) => Some(turn.current),
                Phase::InitialReveal => (0..players)
                    .find(|&p| !game.players[p].eliminated && game.players[p].initial_reveals < 2),
                _ => None,
            };
            if let Some(p) = absent {
                game.skip_absent(p)
                    .unwrap_or_else(|e| panic!("tour non passé : {e:?}"));
                actions += 1;
                continue;
            }
        }
        let (p, action) = legal[chooser.below(legal.len())];
        game.apply(p, action)
            .unwrap_or_else(|e| panic!("coup légal refusé : {action:?} → {e:?}"));
        actions += 1;
    }
    if full {
        assert_eq!(game.card_count(), 150);
    }
    assert!(!game.winners.is_empty());
    // Les gagnants ont bien le total le plus bas.
    let lowest = game
        .players
        .iter()
        .filter(|p| !p.eliminated)
        .map(|p| p.total_score)
        .min()
        .unwrap();
    for &w in &game.winners {
        assert_eq!(game.players[w].total_score, lowest);
    }
    actions
}

#[test]
fn random_games_with_default_rules_always_finish() {
    for seed in 0..30 {
        for players in [2, 3, 4, 7, 10] {
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
            deck: DeckSize::Full,
        },
        Rules {
            penalty: PenaltyRule::Disabled,
            next_round_starter: NextRoundStarter::Rotate,
            end_tie: EndTie::ExtraRound,
            end_score: 10,
            ..Rules::default()
        },
        // Pioche réduite : la défausse est souvent remélangée en pioche.
        Rules {
            deck: DeckSize::Reduced,
            end_score: 30,
            ..Rules::default()
        },
    ];
    for rules in variants {
        for seed in 0..20 {
            for players in [2, 3, 4, 7, 10] {
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

#[test]
fn random_games_with_absent_players_always_finish() {
    for seed in 0..20 {
        for players in [2, 3, 5, 10] {
            play_random_game_with_absences(players, Rules::default(), seed, 15);
        }
    }
}
