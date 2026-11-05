//! Réactions aux messages du chat : `POST /api/react` et le champ `reactions` de `GET /api/messages`
//! (docs/CONTRACT.md), stockage en mémoire.
mod common;

use common::{Client, call, create, join, start, version};
use serde_json::{Value, json};
use spacejo_api::handlers::CHAT_REACTIONS;
use spacejo_api::{ApiRequest, ApiResponse, Endpoint, MemoryStore, Store, handle};

fn memory() -> Store {
    Store::Memory(MemoryStore::new())
}

/// Partie lancée à deux : Alice (hôte) et Bob.
async fn two_players(store: &Store) -> (Client, Client) {
    let (alice, _) = create(store, "Alice", None).await;
    let (bob, _) = join(store, &alice.code, "Bob").await;
    assert_eq!(start(store, &alice).await.status, 200);
    (alice, bob)
}

/// `client` envoie `text` ; renvoie le numéro du message.
async fn say(store: &Store, client: &Client, text: &str) -> i64 {
    let response = call(
        store,
        Endpoint::Chat,
        json!({ "code": client.code, "text": text }),
        Some(&client.token),
    )
    .await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    // Un nouveau message n'a encore aucune réaction.
    assert_eq!(response.body["message"]["reactions"], json!([]));
    response.body["message"]["id"].as_i64().unwrap()
}

/// `client` réagit au message `message` (`reaction` = `null` pour retirer sa réaction).
async fn react(store: &Store, client: &Client, message: Value, reaction: Value) -> ApiResponse {
    call(
        store,
        Endpoint::React,
        json!({ "code": client.code, "message": message, "reaction": reaction }),
        Some(&client.token),
    )
    .await
}

/// Réactions du message `id`, telles que les lit `client`.
async fn reactions_seen_by(store: &Store, client: &Client, id: i64) -> Value {
    let request = ApiRequest {
        method: "GET".to_string(),
        query: format!("code={}", client.code),
        token: Some(client.token.clone()),
        body: Vec::new(),
    };
    let response = handle(store, Endpoint::Messages, request).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    let messages = response.body["messages"].as_array().unwrap().clone();
    messages
        .into_iter()
        .find(|m| m["id"] == id)
        .map(|m| m["reactions"].clone())
        .unwrap()
}

#[tokio::test]
async fn every_player_sees_the_reactions_of_a_message() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;
    let id = say(&store, &alice, "Bien joué !").await;

    let response = react(&store, &bob, json!(id), json!("heart")).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(response.body["message"]["id"], id);
    assert_eq!(
        response.body["message"]["reactions"],
        json!([{ "player": 1, "reaction": "heart" }])
    );
    // On peut aussi réagir à son propre message ; les réactions sont triées par joueur.
    assert_eq!(
        react(&store, &alice, json!(id), json!("laugh"))
            .await
            .status,
        200
    );
    let expected = json!([
        { "player": 0, "reaction": "laugh" },
        { "player": 1, "reaction": "heart" }
    ]);
    assert_eq!(reactions_seen_by(&store, &alice, id).await, expected);
    assert_eq!(reactions_seen_by(&store, &bob, id).await, expected);
}

#[tokio::test]
async fn one_reaction_per_player_which_can_be_changed_or_removed() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;
    let id = say(&store, &alice, "On en refait une ?").await;

    react(&store, &bob, json!(id), json!("heart")).await;
    react(&store, &bob, json!(id), json!("thumbs_up")).await;
    assert_eq!(
        reactions_seen_by(&store, &alice, id).await,
        json!([{ "player": 1, "reaction": "thumbs_up" }]),
        "la nouvelle réaction remplace l'ancienne"
    );
    let response = react(&store, &bob, json!(id), Value::Null).await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    assert_eq!(reactions_seen_by(&store, &alice, id).await, json!([]));
    // Retirer une réaction absente ne fait rien.
    assert_eq!(
        react(&store, &bob, json!(id), Value::Null).await.status,
        200
    );
}

#[tokio::test]
async fn reacting_does_not_change_the_game_version() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;
    let id = say(&store, &alice, "Salut").await;
    let before = version(&store, &alice).await;
    react(&store, &bob, json!(id), json!("wow")).await;
    assert_eq!(version(&store, &alice).await, before);
}

#[tokio::test]
async fn every_reaction_of_the_list_is_accepted() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;
    let id = say(&store, &alice, "Salut").await;
    for reaction in CHAT_REACTIONS {
        let response = react(&store, &bob, json!(id), json!(reaction)).await;
        assert_eq!(response.status, 200, "{reaction} : {:?}", response.body);
    }
}

#[tokio::test]
async fn unknown_reactions_and_messages_are_refused() {
    let store = memory();
    let (alice, bob) = two_players(&store).await;
    let id = say(&store, &alice, "Salut").await;

    // Réaction inconnue : un emoji brut, une majuscule, un nom inventé.
    for reaction in [json!("❤️"), json!("Heart"), json!("poop"), json!(3)] {
        let response = react(&store, &bob, json!(id), reaction.clone()).await;
        assert_eq!(response.status, 400, "{reaction} : {:?}", response.body);
    }
    // Message inexistant.
    let response = react(&store, &bob, json!(id + 100), json!("heart")).await;
    assert_eq!(response.status, 404, "{:?}", response.body);
    assert_eq!(response.body["error"]["code"], "message_not_found");

    // Message d'une autre partie : introuvable depuis celle-ci.
    let (zoe, _) = create(&store, "Zoé", None).await;
    let elsewhere = say(&store, &zoe, "ailleurs").await;
    let response = react(&store, &bob, json!(elsewhere), json!("heart")).await;
    assert_eq!(response.status, 404, "{:?}", response.body);

    // Champ manquant ou inconnu.
    let response = call(
        &store,
        Endpoint::React,
        json!({ "code": alice.code, "message": id }),
        Some(&bob.token),
    )
    .await;
    assert_eq!(response.status, 400, "{:?}", response.body);
    let response = call(
        &store,
        Endpoint::React,
        json!({ "code": alice.code, "message": id, "reaction": "heart", "extra": 1 }),
        Some(&bob.token),
    )
    .await;
    assert_eq!(response.status, 400, "{:?}", response.body);
}

#[tokio::test]
async fn reacting_is_reserved_to_the_players_of_the_game() {
    let store = memory();
    let (alice, _) = two_players(&store).await;
    let id = say(&store, &alice, "Salut").await;
    let (stranger, _) = create(&store, "Zoé", None).await;
    for token in [None, Some(stranger.token.as_str())] {
        let response = call(
            &store,
            Endpoint::React,
            json!({ "code": alice.code, "message": id, "reaction": "heart" }),
            token,
        )
        .await;
        assert_eq!(response.status, 401, "{:?}", response.body);
    }
}

#[test]
fn the_route_has_its_own_path() {
    assert_eq!(Endpoint::from_path("/api/react"), Some(Endpoint::React));
}
