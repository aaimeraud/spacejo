//! Critère 18 (partie format) : versionnage du JSON enregistré dans `games.state`.
//!
//! `tests/fixtures/state_v1.json` est un exemple FIGÉ d'une partie enregistrée en version 1
//! (deux joueurs, mise en place terminée, une carte piochée en main ; les hachages de jetons sont
//! factices). Ne le régénérez pas : s'il ne se relit plus, c'est que le format de `Game` a changé,
//! et il faut alors incrémenter `STATE_VERSION` et écrire la migration (voir `src/stored.rs`).
mod common;

use serde_json::{Value, json};
use spacejo_api::stored::StateError;
use spacejo_api::{MemoryStore, STATE_VERSION, Store, StoredGame};
use spacejo_core::Phase;

fn fixture_v1() -> Value {
    let text = std::fs::read_to_string("tests/fixtures/state_v1.json").unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn the_frozen_v1_example_is_still_readable_and_unchanged() {
    let value = fixture_v1();
    assert_eq!(value["state_version"], 1);
    let stored = StoredGame::decode(value.clone()).expect("la version 1 doit rester lisible");
    assert_eq!(stored.game.players.len(), 2);
    assert!(matches!(stored.game.phase, Phase::Playing(_)));
    assert_eq!(stored.game.card_count(), 150);
    // Relire puis réécrire redonne exactement le même JSON : le format n'a pas bougé.
    assert_eq!(stored.encode().unwrap(), value);
}

#[test]
fn the_current_version_is_1() {
    assert_eq!(STATE_VERSION, 1);
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
    value["state_version"] = json!(2);
    let error = StoredGame::decode(value).unwrap_err();
    assert_eq!(error, StateError::TooNew { found: 2 });
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
