//! Quitter une partie (`POST /api/leave`) et retirer un joueur du salon (`POST /api/kick`), avec le stockage
//! en mémoire.
mod common;

use common::{Client, act, call, create, get_state, join, start, version};
use serde_json::{Value, json};
use spacejo_api::{ApiResponse, Endpoint, MemoryStore, Store};

fn memory() -> Store {
    Store::Memory(MemoryStore::new())
}

async fn leave(store: &Store, client: &Client) -> ApiResponse {
    call(
        store,
        Endpoint::Leave,
        json!({ "code": client.code }),
        Some(&client.token),
    )
    .await
}

async fn kick(store: &Store, client: &Client, expected_version: i64, player: u64) -> ApiResponse {
    call(
        store,
        Endpoint::Kick,
        json!({ "code": client.code, "expected_version": expected_version, "player": player }),
        Some(&client.token),
    )
    .await
}

async fn state(store: &Store, client: &Client) -> ApiResponse {
    get_state(store, &format!("code={}", client.code), Some(&client.token)).await
}

fn player_names(body: &Value) -> Vec<String> {
    body["view"]["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect()
}

/// Salon de trois joueurs : Alice (hôte), Bob, Carla.
async fn lobby_of_three(store: &Store) -> (Client, Client, Client) {
    let (alice, _) = create(store, "Alice", None).await;
    let (bob, _) = join(store, &alice.code, "Bob").await;
    let (carla, _) = join(store, &alice.code, "Carla").await;
    (alice, bob, carla)
}

// ----- Salon -----

#[tokio::test]
async fn a_player_who_leaves_the_lobby_disappears_and_the_others_keep_playing() {
    let store = memory();
    let (alice, bob, carla) = lobby_of_three(&store).await;
    let v = version(&store, &alice).await;

    let response = leave(&store, &bob).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(response.body["version"], v + 1);

    // Bob n'a plus de place : son jeton est refusé.
    assert_eq!(state(&store, &bob).await.status, 401);
    // Carla a avancé d'une place, et son jeton la reconnaît toujours.
    let seen_by_carla = state(&store, &carla).await;
    assert_eq!(seen_by_carla.status, 200);
    assert_eq!(seen_by_carla.body["view"]["me"], 1);
    assert_eq!(player_names(&seen_by_carla.body), vec!["Alice", "Carla"]);
    // Sa présence a suivi sa nouvelle place.
    let seen_by_alice = state(&store, &alice).await;
    assert_eq!(seen_by_alice.body["presence"][1]["connected"], true);
}

#[tokio::test]
async fn when_the_host_leaves_the_lobby_the_next_player_becomes_host() {
    let store = memory();
    let (alice, bob, _) = lobby_of_three(&store).await;
    assert_eq!(leave(&store, &alice).await.status, 200);
    let seen_by_bob = state(&store, &bob).await;
    assert_eq!(seen_by_bob.body["view"]["me"], 0);
    assert_eq!(seen_by_bob.body["view"]["host"], 0);
    // Bob peut désormais lancer la partie.
    assert_eq!(start(&store, &bob).await.status, 200);
}

#[tokio::test]
async fn the_last_player_leaving_a_lobby_writes_nothing() {
    let store = memory();
    let (alice, _) = create(&store, "Alice", None).await;
    let response = leave(&store, &alice).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(response.body["version"], 1);
    assert_eq!(version(&store, &alice).await, 1);
}

#[tokio::test]
async fn the_host_kicks_a_player_from_the_lobby() {
    let store = memory();
    let (alice, bob, carla) = lobby_of_three(&store).await;
    let v = version(&store, &alice).await;

    let response = kick(&store, &alice, v, 1).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(player_names(&response.body), vec!["Alice", "Carla"]);
    assert_eq!(state(&store, &bob).await.status, 401);
    assert_eq!(state(&store, &carla).await.body["view"]["me"], 1);
}

#[tokio::test]
async fn kicking_is_reserved_to_the_host_in_the_lobby() {
    let store = memory();
    let (alice, bob, _) = lobby_of_three(&store).await;
    let v = version(&store, &alice).await;

    let response = kick(&store, &bob, v, 2).await;
    assert_eq!(response.status, 403);
    assert_eq!(response.body["error"]["code"], "not_host");

    let response = kick(&store, &alice, v, 0).await;
    assert_eq!(response.status, 422);
    assert_eq!(response.body["error"]["code"], "cannot_kick_host");

    let response = kick(&store, &alice, v - 1, 1).await;
    assert_eq!(response.status, 409);

    assert_eq!(start(&store, &alice).await.status, 200);
    let v = version(&store, &alice).await;
    let response = kick(&store, &alice, v, 1).await;
    assert_eq!(response.status, 422);
    assert_eq!(response.body["error"]["code"], "wrong_phase");
}

// ----- Partie lancée -----

#[tokio::test]
async fn a_player_who_leaves_a_started_game_is_shown_as_abandoned_to_everyone() {
    let store = memory();
    let (alice, bob, carla) = lobby_of_three(&store).await;
    assert_eq!(start(&store, &alice).await.status, 200);
    let v = version(&store, &alice).await;

    let response = leave(&store, &carla).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(response.body["version"], v + 1);

    for client in [&alice, &bob] {
        let body = state(&store, client).await.body;
        assert_eq!(body["view"]["players"][2]["abandoned"], true);
        assert_eq!(body["view"]["players"][2]["eliminated"], true);
        // Les places ne bougent pas pendant une partie.
        assert_eq!(player_names(&body), vec!["Alice", "Bob", "Carla"]);
    }
    // Carla ne joue plus.
    let v = version(&store, &alice).await;
    let response = act(
        &store,
        &carla,
        v,
        json!({ "type": "RevealInitial", "column": 0, "row": 0 }),
    )
    .await;
    assert_eq!(response.body["error"]["code"], "eliminated");
}

#[tokio::test]
async fn leaving_a_finished_game_writes_nothing() {
    let store = memory();
    let (alice, _) = create(&store, "Alice", None).await;
    let (bob, _) = join(&store, &alice.code, "Bob").await;
    assert_eq!(start(&store, &alice).await.status, 200);
    // Bob abandonne : Alice gagne, la partie est finie.
    assert_eq!(leave(&store, &bob).await.status, 200);
    let v = version(&store, &alice).await;
    assert_eq!(
        state(&store, &alice).await.body["view"]["phase"],
        "GameOver"
    );
    let response = leave(&store, &alice).await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body["version"], v);
}

#[tokio::test]
async fn leaving_needs_a_valid_token() {
    let store = memory();
    let (alice, _) = create(&store, "Alice", None).await;
    let response = call(&store, Endpoint::Leave, json!({ "code": alice.code }), None).await;
    assert_eq!(response.status, 401);
    let response = call(
        &store,
        Endpoint::Leave,
        json!({ "code": alice.code, "extra": 1 }),
        Some(&alice.token),
    )
    .await;
    assert_eq!(response.status, 400);
}

#[test]
fn the_routes_have_their_own_paths() {
    assert_eq!(Endpoint::from_path("/api/leave"), Some(Endpoint::Leave));
    assert_eq!(Endpoint::from_path("/api/kick"), Some(Endpoint::Kick));
}
