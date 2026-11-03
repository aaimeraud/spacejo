//! Critères 9 et 16 (partie core) : réglages par défaut, validation, figés après le lancement.
mod common;

use spacejo_core::{
    END_SCORE_MAX, END_SCORE_MIN, EmptyPile, EndTie, FirstPlayerTie, Game, GameError,
    NextRoundStarter, PenaltyRule, PenaltyScope, Rules,
};

#[test]
fn default_rules_are_the_official_rules() {
    let rules = Rules::default();
    assert_eq!(rules.penalty, PenaltyRule::DoubleUnlessStrictlyLowest);
    assert_eq!(rules.penalty_scope, PenaltyScope::RoundTotal);
    assert_eq!(rules.first_player_tie, FirstPlayerTie::JoinOrder);
    assert_eq!(rules.next_round_starter, NextRoundStarter::Finisher);
    assert_eq!(rules.empty_pile, EmptyPile::ReshuffleDiscard);
    assert_eq!(rules.end_tie, EndTie::SharedVictory);
    assert_eq!(rules.end_score, 100);
}

#[test]
fn end_score_bounds_are_10_and_500() {
    assert_eq!(END_SCORE_MIN, 10);
    assert_eq!(END_SCORE_MAX, 500);
    for ok in [10, 100, 500] {
        let rules = Rules {
            end_score: ok,
            ..Rules::default()
        };
        assert_eq!(rules.validate(), Ok(()));
    }
    for bad in [9, 0, -100, 501, i32::MAX] {
        let rules = Rules {
            end_score: bad,
            ..Rules::default()
        };
        assert_eq!(rules.validate(), Err(GameError::InvalidRules));
        assert_eq!(Game::new("Hôte", rules, 1), Err(GameError::InvalidRules));
    }
}

#[test]
fn rules_serialize_with_every_field() {
    let json = serde_json::to_value(Rules::default()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "penalty": "DoubleUnlessStrictlyLowest",
            "penalty_scope": "RoundTotal",
            "first_player_tie": "JoinOrder",
            "next_round_starter": "Finisher",
            "empty_pile": "ReshuffleDiscard",
            "end_tie": "SharedVictory",
            "end_score": 100
        })
    );
}

#[test]
fn unknown_values_and_fields_are_refused() {
    let valid = serde_json::to_value(Rules::default()).unwrap();
    for field in [
        "penalty",
        "penalty_scope",
        "first_player_tie",
        "next_round_starter",
        "empty_pile",
        "end_tie",
    ] {
        let mut json = valid.clone();
        json[field] = serde_json::json!("Inconnu");
        assert!(
            serde_json::from_value::<Rules>(json).is_err(),
            "valeur inconnue acceptée pour {field}"
        );
        // Un champ manquant est aussi refusé : les réglages sont toujours complets.
        let mut json = valid.clone();
        json.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Rules>(json).is_err());
    }
    let mut json = valid.clone();
    json["end_score"] = serde_json::json!("cent");
    assert!(serde_json::from_value::<Rules>(json).is_err());
    let mut json = valid;
    json["bonus"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<Rules>(json).is_err(),
        "champ inconnu"
    );
}

#[test]
fn only_the_host_changes_rules_and_only_in_the_lobby() {
    let mut game = common::lobby(2, Rules::default());
    let variant = Rules {
        penalty: PenaltyRule::Disabled,
        end_score: 50,
        ..Rules::default()
    };
    assert_eq!(game.update_rules(1, variant), Err(GameError::NotHost));
    assert_eq!(game.rules, Rules::default());
    let invalid = Rules {
        end_score: 5,
        ..Rules::default()
    };
    assert_eq!(game.update_rules(0, invalid), Err(GameError::InvalidRules));
    game.update_rules(0, variant).unwrap();
    assert_eq!(game.rules, variant);

    // Seul l'hôte lance la partie.
    assert_eq!(game.start(1), Err(GameError::NotHost));
    game.start(0).unwrap();
    // Après le lancement, les réglages sont figés, même pour l'hôte.
    assert_eq!(
        game.update_rules(0, Rules::default()),
        Err(GameError::WrongPhase)
    );
    assert_eq!(game.rules, variant);
}

#[test]
fn players_and_names_are_validated() {
    assert_eq!(
        Game::new("   ", Rules::default(), 1),
        Err(GameError::InvalidName)
    );
    let long = "x".repeat(21);
    assert_eq!(
        Game::new(&long, Rules::default(), 1),
        Err(GameError::InvalidName)
    );
    let mut game = Game::new("  Alice  ", Rules::default(), 1).unwrap();
    assert_eq!(game.players[0].name, "Alice", "espaces retirés");
    assert_eq!(game.start(0), Err(GameError::NotEnoughPlayers));
    assert_eq!(game.join("Bob"), Ok(1));
    assert_eq!(game.join("Chloé"), Ok(2));
    assert_eq!(game.join("Dan"), Ok(3));
    for i in 4..spacejo_core::MAX_PLAYERS {
        assert_eq!(game.join(&format!("Joueur {i}")), Ok(i));
    }
    assert_eq!(game.join("En trop"), Err(GameError::GameFull));
}
