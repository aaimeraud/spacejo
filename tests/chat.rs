//! Chat pendant les parties : `POST /api/chat` et `GET /api/messages` (docs/CONTRACT.md), stockage en mémoire.
mod common;

use common::{Client, call, create, join, start, version};
use serde_json::{Value, json};
use spacejo_api::handlers::{MAX_CHAT_CHARS, MAX_CHAT_HISTORY};
use spacejo_api::{ApiRequest, ApiResponse, Endpoint, MemoryStore, Store, handle};

fn memory() -> Store {
    Store::Memory(MemoryStore::new())
}

async fn say(store: &Store, client: &Client, text: &str) -> ApiResponse {
    call(
        store,
        Endpoint::Chat,
        json!({ "code": client.code, "text": text }),
        Some(&client.token),
    )
    .await
}

async fn read_query(store: &Store, query: &str, token: Option<&str>) -> ApiResponse {
    let request = ApiRequest {
        method: "GET".to_string(),
        query: query.to_string(),
        token: token.map(str::to_string),
        body: Vec::new(),
    };
    handle(store, Endpoint::Messages, request).await
}

/// Messages lus par `client`, plus récents que `after` (tous si `None`).
async fn messages(store: &Store, client: &Client, after: Option<i64>) -> Vec<Value> {
    let query = match after {
        Some(after) => format!("code={}&after={after}", client.code),
        None => format!("code={}", client.code),
    };
    let response = read_query(store, &query, Some(&client.token)).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    response.body["messages"].as_array().unwrap().clone()
}

/// Partie lancée à deux : Alice (hôte) et Bob.
async fn two_players(store: &Store) -> (Client, Client) {
    let (alice, _) = create(store, "Alice", None).await;
    let (bob, _) = join(store, &alice.code, "Bob").await;
    assert_eq!(start(store, &alice).await.status, 200);
    (alice, bob)
}

#[tokio::test]
async fn a_message_is_read_by_every_player_in_order() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;

    let response = say(&store, &alice, "  Bonne chance !  ").await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    let first = response.body["message"].clone();
    assert_eq!(first["player"], 0);
    assert_eq!(first["name"], "Alice");
    // Les espaces autour sont retirés.
    assert_eq!(first["text"], "Bonne chance !");
    assert!(first["id"].as_i64().unwrap() > 0);

    say(&store, &bob, "Merci, toi aussi").await;
    let seen_by_bob = messages(&store, &bob, None).await;
    let texts: Vec<&str> = seen_by_bob
        .iter()
        .map(|m| m["text"].as_str().unwrap())
        .collect();
    assert_eq!(texts, vec!["Bonne chance !", "Merci, toi aussi"]);
    assert_eq!(seen_by_bob[1]["name"], "Bob");

    // `after` ne renvoie que les messages suivants.
    let newer = messages(&store, &alice, first["id"].as_i64()).await;
    assert_eq!(newer.len(), 1);
    assert_eq!(newer[0]["text"], "Merci, toi aussi");
}

#[tokio::test]
async fn sending_a_message_does_not_change_the_game_version() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    let before = version(&store, &alice).await;
    assert_eq!(say(&store, &alice, "Salut").await.status, 200);
    assert_eq!(version(&store, &alice).await, before);
}

#[tokio::test]
async fn only_the_most_recent_messages_are_kept_in_the_history() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    for i in 0..MAX_CHAT_HISTORY + 5 {
        assert_eq!(
            say(&store, &alice, &format!("message {i}")).await.status,
            200
        );
    }
    let all = messages(&store, &alice, None).await;
    assert_eq!(all.len(), MAX_CHAT_HISTORY);
    // Les plus récents, du plus ancien au plus récent.
    assert_eq!(all[0]["text"], "message 5");
    assert_eq!(
        all[MAX_CHAT_HISTORY - 1]["text"],
        format!("message {}", MAX_CHAT_HISTORY + 4)
    );
}

#[tokio::test]
async fn invalid_messages_are_refused() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    let too_long = "a".repeat(MAX_CHAT_CHARS + 1);
    for text in ["", "   ", too_long.as_str(), "ligne\nsuivante", "son\u{7}"] {
        let response = say(&store, &alice, text).await;
        assert_eq!(response.status, 400, "{text:?} : {:?}", response.body);
        assert_eq!(response.body["error"]["code"], "invalid_message");
    }
    // Exactement la longueur maximale (en caractères, pas en octets) : accepté.
    let longest = "é".repeat(MAX_CHAT_CHARS);
    assert_eq!(say(&store, &alice, &longest).await.status, 200);
    assert_eq!(messages(&store, &alice, None).await.len(), 1);
}

#[tokio::test]
async fn the_chat_is_reserved_to_the_players_of_the_game() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    let (stranger, _) = create(&store, "Zoé", None).await;

    // Sans jeton, ou avec le jeton d'une autre partie.
    let response = call(
        &store,
        Endpoint::Chat,
        json!({ "code": alice.code, "text": "coucou" }),
        None,
    )
    .await;
    assert_eq!(response.status, 401);
    let response = call(
        &store,
        Endpoint::Chat,
        json!({ "code": alice.code, "text": "coucou" }),
        Some(&stranger.token),
    )
    .await;
    assert_eq!(response.status, 401);
    let response = read_query(
        &store,
        &format!("code={}", alice.code),
        Some(&stranger.token),
    )
    .await;
    assert_eq!(response.status, 401);
}

#[tokio::test]
async fn malformed_reads_are_refused() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    for query in [
        format!("code={}&after=abc", alice.code),
        format!("code={}&page=2", alice.code),
        "after=3".to_string(),
    ] {
        let response = read_query(&store, &query, Some(&alice.token)).await;
        assert_eq!(response.status, 400, "{query} : {:?}", response.body);
    }
    // Un champ inconnu à l'envoi.
    let response = call(
        &store,
        Endpoint::Chat,
        json!({ "code": alice.code, "text": "a", "extra": 1 }),
        Some(&alice.token),
    )
    .await;
    assert_eq!(response.status, 400);
}

#[test]
fn the_routes_have_their_own_paths() {
    assert_eq!(Endpoint::from_path("/api/chat"), Some(Endpoint::Chat));
    assert_eq!(
        Endpoint::from_path("/api/messages"),
        Some(Endpoint::Messages)
    );
}
