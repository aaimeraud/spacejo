//! Parties publiques et privées, avec le stockage en mémoire (docs/CONTRACT.md, `public` et
//! `GET /api/public-games`).
mod common;

use std::time::Duration;

use common::{call, create, get_state, join, start};
use serde_json::{Value, json};
use spacejo_api::handlers::PRESENCE_TIMEOUT_SECS;
use spacejo_api::{ApiRequest, ApiResponse, Endpoint, MemoryStore, Store, handle};
use spacejo_core::MAX_PLAYERS;

fn memory() -> (Store, MemoryStore) {
    let memory = MemoryStore::new();
    (Store::Memory(memory.clone()), memory)
}

/// `GET /api/public-games`, avec des paramètres d'URL éventuels.
async fn list_with(store: &Store, method: &str, query: &str) -> ApiResponse {
    let request = ApiRequest {
        method: method.to_string(),
        query: query.to_string(),
        token: None,
        body: Vec::new(),
    };
    handle(store, Endpoint::PublicGames, request).await
}

/// Liste des parties publiques (réponse 200 attendue).
async fn public_games(store: &Store) -> Vec<Value> {
    let response = list_with(store, "GET", "").await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    response.body["games"].as_array().unwrap().clone()
}

/// Crée une partie publique et renvoie son hôte.
async fn create_public(store: &Store, name: &str) -> common::Client {
    let response = call(
        store,
        Endpoint::Create,
        json!({ "name": name, "public": true }),
        None,
    )
    .await;
    assert_eq!(response.status, 201, "{:?}", response.body);
    assert_eq!(response.body["public"], true);
    common::Client {
        code: response.body["code"].as_str().unwrap().to_string(),
        token: response.body["token"].as_str().unwrap().to_string(),
        player_id: 0,
    }
}

#[tokio::test]
async fn a_game_is_private_by_default() {
    let (store, _) = memory();
    let (host, response) = create(&store, "Alice", None).await;
    assert_eq!(response.body["public"], false);
    // La visibilité figure aussi dans les réponses suivantes (jonction, lecture de l'état).
    let (_, joined) = join(&store, &host.code, "Bob").await;
    assert_eq!(joined.body["public"], false);
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    assert_eq!(state.body["public"], false);
    // Une partie privée n'est jamais listée.
    assert!(public_games(&store).await.is_empty());
}

#[tokio::test]
async fn a_public_lobby_is_listed_with_its_host_and_player_count() {
    let (store, _) = memory();
    let host = create_public(&store, "Alice").await;
    let state = get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    assert_eq!(state.body["public"], true);

    let games = public_games(&store).await;
    assert_eq!(
        games,
        vec![json!({
            "code": host.code,
            "host": "Alice",
            "players": 1,
            "max_players": MAX_PLAYERS,
        })]
    );

    join(&store, &host.code, "Bob").await;
    assert_eq!(public_games(&store).await[0]["players"], 2);
}

#[tokio::test]
async fn the_list_shows_the_most_recent_games_first() {
    let (store, _) = memory();
    let first = create_public(&store, "Alice").await;
    let second = create_public(&store, "Carla").await;
    // Une jonction rend la première partie la plus récemment modifiée.
    join(&store, &first.code, "Bob").await;
    let codes: Vec<Value> = public_games(&store)
        .await
        .iter()
        .map(|game| game["code"].clone())
        .collect();
    assert_eq!(codes, vec![json!(first.code), json!(second.code)]);
}

#[tokio::test]
async fn started_and_full_games_are_not_listed() {
    let (store, _) = memory();
    let started = create_public(&store, "Alice").await;
    join(&store, &started.code, "Bob").await;
    assert_eq!(start(&store, &started).await.status, 200);

    let full = create_public(&store, "Carla").await;
    for i in 1..MAX_PLAYERS {
        join(&store, &full.code, &format!("Joueur {i}")).await;
    }
    assert!(public_games(&store).await.is_empty());
}

#[tokio::test]
async fn a_lobby_whose_host_left_is_not_listed() {
    let (store, memory) = memory();
    let host = create_public(&store, "Alice").await;
    memory.advance(Duration::from_secs_f64(PRESENCE_TIMEOUT_SECS + 1.0));
    assert!(public_games(&store).await.is_empty());

    // L'hôte revient (son polling le note présent) : la partie réapparaît.
    get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    assert_eq!(public_games(&store).await.len(), 1);
}

#[tokio::test]
async fn the_list_never_contains_cards_or_tokens() {
    let (store, _) = memory();
    let host = create_public(&store, "Alice").await;
    let response = list_with(&store, "GET", "").await;
    let text = response.body.to_string();
    assert!(!text.contains(&host.token));
    assert!(!text.contains("token"));
    assert!(!text.contains("grid"));
}

#[tokio::test]
async fn invalid_requests_are_refused() {
    let (store, _) = memory();
    // `public` doit être un booléen.
    let response = call(
        &store,
        Endpoint::Create,
        json!({ "name": "Alice", "public": "oui" }),
        None,
    )
    .await;
    assert_eq!(response.status, 400);
    assert_eq!(response.body["error"]["code"], "invalid_request");

    // La liste se lit en GET, sans paramètre.
    assert_eq!(list_with(&store, "POST", "").await.status, 405);
    let response = list_with(&store, "GET", "limit=1000").await;
    assert_eq!(response.status, 400);
    assert_eq!(response.body["error"]["code"], "invalid_request");
}

#[test]
fn the_route_has_its_own_path() {
    assert_eq!(
        Endpoint::from_path("/api/public-games"),
        Some(Endpoint::PublicGames)
    );
}
