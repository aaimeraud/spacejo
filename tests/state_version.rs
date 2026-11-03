//! Critère 18 (partie format) : versionnage du JSON enregistré dans `games.state`.
//!
//! `tests/fixtures/state_v1.json` et `state_v2.json` sont des exemples FIGÉS de parties enregistrées
//! dans chaque version (deux joueurs, mise en place terminée, une carte piochée en main ; les hachages
//! de jetons sont factices). Ne les régénérez pas : si l'un ne se relit plus, c'est que le format de
//! `Game` a changé, et il faut alors incrémenter `STATE_VERSION` et écrire la migration
//! (voir `src/stored.rs`).
mod common;

use serde_json::{Value, json};
use spacejo_api::stored::StateError;
use spacejo_api::{MemoryStore, STATE_VERSION, Store, StoredGame};
use spacejo_core::Phase;

fn fixture(name: &str) -> Value {
    let text = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn fixture_v1() -> Value {
    fixture("state_v1.json")
}

#[test]
fn the_frozen_v2_example_is_still_readable_and_unchanged() {
    let value = fixture("state_v2.json");
    assert_eq!(value["state_version"], 2);
    let stored = StoredGame::decode(value.clone()).expect("la version 2 doit rester lisible");
    assert_eq!(stored.game.players.len(), 2);
    assert!(matches!(stored.game.phase, Phase::Playing(_)));
    assert_eq!(stored.game.card_count(), 150);
    // Relire puis réécrire redonne exactement le même JSON : le format n'a pas bougé.
    assert_eq!(stored.encode().unwrap(), value);
}

#[test]
fn a_v1_game_is_migrated_to_v2() {
    let value = fixture_v1();
    assert_eq!(value["state_version"], 1);
    let stored = StoredGame::decode(value.clone()).expect("la version 1 doit être migrée");
    assert_eq!(stored.state_version, 2);
    for player in &stored.game.players {
        assert_eq!(player.missed_turns, 0);
        assert!(!player.eliminated);
    }
    assert_eq!(stored.game.card_count(), 150);
    // La migration ne change rien d'autre : même JSON, plus les deux champs et la nouvelle version.
    let mut expected = value;
    expected["state_version"] = json!(2);
    for player in expected["game"]["players"].as_array_mut().unwrap() {
        player["missed_turns"] = json!(0);
        player["eliminated"] = json!(false);
    }
    assert_eq!(stored.encode().unwrap(), expected);
}

#[test]
fn the_current_version_is_2() {
    assert_eq!(STATE_VERSION, 2);
}

#[test]
fn a_state_without_version_is_refused_clearly() {
    let mut value = fixture_v1();
    value.as_object_mut().unwrap().remove("state_version");
    let error = StoredGame::decode(value).unwrap_err();
    assert_eq!(error, StateError::MissingVersion);
    assert!(error.message().contains("state_version"));
}

#[test]
fn an_older_version_without_migration_is_refused_clearly() {
    let mut value = fixture_v1();
    value["state_version"] = json!(0);
    let error = StoredGame::decode(value).unwrap_err();
    assert_eq!(error, StateError::Unsupported { found: 0 });
    assert!(error.message().contains("aucune migration"));
}

#[test]
fn a_newer_version_is_refused_clearly() {
    let mut value = fixture_v1();
    value["state_version"] = json!(3);
    let error = StoredGame::decode(value).unwrap_err();
    assert_eq!(error, StateError::TooNew { found: 3 });
    assert!(error.message().contains("plus récente"));
}

#[test]
fn a_corrupt_state_is_refused() {
    let mut value = fixture_v1();
    value["game"]["players"] = json!("n'importe quoi");
    assert_eq!(StoredGame::decode(value).unwrap_err(), StateError::Corrupt);
}

#[tokio::test]
async fn the_api_answers_with_a_clear_error_for_an_unreadable_state() {
    let memory = MemoryStore::new();
    let mut value = fixture_v1();
    value["state_version"] = json!(7);
    memory.put_raw("K7MPQX", value).unwrap();
    let store = Store::Memory(memory);
    let response = common::get_state(&store, "code=K7MPQX", Some("x")).await;
    assert_eq!(response.status, 500);
    assert_eq!(response.body["error"]["code"], "unsupported_state_version");
}
