//! Durées de conservation (demandées par le propriétaire du dépôt, ASSUMPTIONS.md n° 40 et 41) : le chat d'une partie
//! finie (messages et réactions) disparaît `CHAT_KEPT_AFTER_GAME_OVER` après la fin ; toute partie sans écriture
//! depuis `GAME_KEPT_WHEN_IDLE` est supprimée, avec son chat et sa présence. Stockage en mémoire.
mod common;

use std::time::Duration;

use common::{create, join};
use serde_json::{Value, json};
use spacejo_api::handlers::{CHAT_KEPT_AFTER_GAME_OVER, GAME_KEPT_WHEN_IDLE};
use spacejo_api::{MemoryStore, Store};

/// État minimal d'une partie dans la phase `phase` (seul le nom de la phase compte pour l'effacement).
fn state(phase: &str) -> Value {
    json!({ "game": { "phase": phase } })
}

/// Une partie dans la phase `phase`, avec deux messages dont un a une réaction. Renvoie son identifiant.
async fn game_with_chat(store: &Store, code: &str, phase: &str) -> i64 {
    let record = store.insert(code, &state(phase), false).await.unwrap();
    let first = store
        .insert_message(record.id, 0, "Alice", "Bonne partie")
        .await
        .unwrap();
    store
        .insert_message(record.id, 1, "Bob", "Merci !")
        .await
        .unwrap();
    store
        .set_reaction(record.id, first.id, 1, Some("heart"))
        .await
        .unwrap();
    record.id
}

async fn count(store: &Store, game: i64) -> usize {
    store.messages(game, 0, 100).await.unwrap().len()
}

#[tokio::test]
async fn the_chat_of_a_finished_game_is_erased_after_a_short_delay() {
    let memory = MemoryStore::new();
    let store = Store::Memory(memory.clone());
    let over = game_with_chat(&store, "K7MPQX", "GameOver").await;
    let playing = game_with_chat(&store, "ABCDEF", "InitialReveal").await;

    // Juste après la fin : on peut encore se dire « bien joué » sur l'écran de fin.
    assert_eq!(store.purge_chats().await.unwrap(), 0);
    assert_eq!(count(&store, over).await, 2);

    memory.advance(CHAT_KEPT_AFTER_GAME_OVER + Duration::from_secs(1));
    assert_eq!(store.purge_chats().await.unwrap(), 2, "les deux messages");
    assert_eq!(count(&store, over).await, 0);
    // Une partie en cours garde son chat.
    assert_eq!(count(&store, playing).await, 2);
}

#[tokio::test]
async fn a_game_idle_for_a_day_is_deleted_with_its_chat() {
    let memory = MemoryStore::new();
    let store = Store::Memory(memory.clone());
    let game = game_with_chat(&store, "K7MPQX", "InitialReveal").await;
    store.touch(game, 0).await.unwrap();
    memory.advance(GAME_KEPT_WHEN_IDLE - Duration::from_secs(60));
    assert_eq!(store.purge_games().await.unwrap(), 0);
    assert!(store.load("K7MPQX").await.unwrap().is_some());

    memory.advance(Duration::from_secs(120));
    // Une autre partie, toute récente, n'est pas touchée.
    let recent = game_with_chat(&store, "ABCDEF", "Lobby").await;
    assert_eq!(store.purge_games().await.unwrap(), 1);
    assert!(
        store.load("K7MPQX").await.unwrap().is_none(),
        "partie supprimée"
    );
    assert_eq!(count(&store, game).await, 0, "son chat aussi");
    assert!(
        store.presence(game).await.unwrap().is_empty(),
        "et sa présence"
    );
    assert!(store.load("ABCDEF").await.unwrap().is_some());
    assert_eq!(count(&store, recent).await, 2);
}

#[tokio::test]
async fn message_numbers_keep_growing_after_an_erasure() {
    let memory = MemoryStore::new();
    let store = Store::Memory(memory.clone());
    let old = game_with_chat(&store, "K7MPQX", "GameOver").await;
    memory.advance(CHAT_KEPT_AFTER_GAME_OVER + Duration::from_secs(1));
    store.purge_chats().await.unwrap();
    let _ = old;
    // Un numéro déjà donné n'est jamais redonné : un navigateur ne confond pas deux messages.
    let record = store
        .insert("ABCDEF", &state("Lobby"), false)
        .await
        .unwrap();
    let message = store
        .insert_message(record.id, 0, "Zoé", "Salut")
        .await
        .unwrap();
    assert!(message.id > 2, "numéro {}", message.id);
    assert!(message.reactions.is_empty());
}

#[tokio::test]
async fn a_deleted_game_can_no_longer_be_read() {
    let memory = MemoryStore::new();
    let store = Store::Memory(memory.clone());
    let (alice, _) = create(&store, "Alice", None).await;
    memory.advance(GAME_KEPT_WHEN_IDLE + Duration::from_secs(1));
    // Un autre joueur crée une partie : le ménage passe, la partie d'Alice disparaît.
    create(&store, "Zoé", None).await;
    let response =
        common::get_state(&store, &format!("code={}", alice.code), Some(&alice.token)).await;
    assert_eq!(response.status, 404, "{:?}", response.body);
    assert_eq!(response.body["error"]["code"], "game_not_found");
}

#[tokio::test]
async fn creating_or_joining_a_game_erases_the_old_chats() {
    let memory = MemoryStore::new();
    let store = Store::Memory(memory.clone());
    let over = game_with_chat(&store, "K7MPQX", "GameOver").await;
    memory.advance(CHAT_KEPT_AFTER_GAME_OVER + Duration::from_secs(1));
    let (alice, _) = create(&store, "Alice", None).await;
    assert_eq!(count(&store, over).await, 0, "effacé à la création");

    let again = game_with_chat(&store, "ABCDEF", "GameOver").await;
    memory.advance(CHAT_KEPT_AFTER_GAME_OVER + Duration::from_secs(1));
    let (_, response) = join(&store, &alice.code, "Bob").await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(count(&store, again).await, 0, "effacé à la jonction");
}
