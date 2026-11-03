//! Outils partagés par les tests de l'API.
#![allow(dead_code)]

use serde_json::{Value, json};
use spacejo_api::{ApiRequest, ApiResponse, Endpoint, Store, handle};

/// Appelle une route comme le ferait le navigateur.
pub async fn call(
    store: &Store,
    endpoint: Endpoint,
    body: Value,
    token: Option<&str>,
) -> ApiResponse {
    let method = if endpoint == Endpoint::State {
        "GET"
    } else {
        "POST"
    };
    let request = ApiRequest {
        method: method.to_string(),
        query: String::new(),
        token: token.map(str::to_string),
        body: serde_json::to_vec(&body).unwrap(),
    };
    handle(store, endpoint, request).await
}

/// Lit l'état (`GET /api/state`).
pub async fn get_state(store: &Store, query: &str, token: Option<&str>) -> ApiResponse {
    let request = ApiRequest {
        method: "GET".to_string(),
        query: query.to_string(),
        token: token.map(str::to_string),
        body: Vec::new(),
    };
    handle(store, Endpoint::State, request).await
}

/// Un joueur côté client : code, jeton, identifiant.
#[derive(Clone, Debug)]
pub struct Client {
    pub code: String,
    pub token: String,
    pub player_id: u64,
}

pub async fn create(store: &Store, name: &str, rules: Option<Value>) -> (Client, ApiResponse) {
    let mut body = json!({ "name": name });
    if let Some(rules) = rules {
        body["rules"] = rules;
    }
    let response = call(store, Endpoint::Create, body, None).await;
    assert_eq!(response.status, 201, "{:?}", response.body);
    let client = Client {
        code: response.body["code"].as_str().unwrap().to_string(),
        token: response.body["token"].as_str().unwrap().to_string(),
        player_id: response.body["player_id"].as_u64().unwrap(),
    };
    (client, response)
}

pub async fn join(store: &Store, code: &str, name: &str) -> (Client, ApiResponse) {
    let response = call(
        store,
        Endpoint::Join,
        json!({ "code": code, "name": name }),
        None,
    )
    .await;
    assert_eq!(response.status, 200, "{:?}", response.body);
    let client = Client {
        code: code.to_string(),
        token: response.body["token"].as_str().unwrap().to_string(),
        player_id: response.body["player_id"].as_u64().unwrap(),
    };
    (client, response)
}

/// Version actuelle de la partie, lue avec le jeton d'un joueur.
pub async fn version(store: &Store, client: &Client) -> i64 {
    let response = get_state(store, &format!("code={}", client.code), Some(&client.token)).await;
    response.body["version"].as_i64().unwrap()
}

pub async fn start(store: &Store, host: &Client) -> ApiResponse {
    let v = version(store, host).await;
    call(
        store,
        Endpoint::Start,
        json!({ "code": host.code, "expected_version": v }),
        Some(&host.token),
    )
    .await
}

pub async fn act(store: &Store, client: &Client, version: i64, action: Value) -> ApiResponse {
    call(
        store,
        Endpoint::Action,
        json!({ "code": client.code, "expected_version": version, "action": action }),
        Some(&client.token),
    )
    .await
}

/// Vérifie récursivement qu'aucune case cachée ne porte de valeur dans un JSON de réponse.
pub fn assert_no_hidden_values(value: &Value) {
    match value {
        Value::Object(map) => {
            if map.get("kind") == Some(&json!("Hidden")) {
                assert_eq!(
                    map.len(),
                    1,
                    "case cachée avec des champs en trop : {value}"
                );
            }
            map.values().for_each(assert_no_hidden_values);
        }
        Value::Array(items) => items.iter().for_each(assert_no_hidden_values),
        _ => {}
    }
}

/// Hachages de jetons enregistrés pour une partie (dans l'état brut de la base en mémoire).
pub fn stored_hashes(state: &Value) -> Vec<String> {
    state["token_hashes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h.as_str().unwrap().to_string())
        .collect()
}
