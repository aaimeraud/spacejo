//! Le serveur HTTP (axum) : routes de l'API, contrôle de santé, fichiers de l'interface.
//! Les requêtes sont envoyées directement au routeur (`oneshot`), sans ouvrir de port.

use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use futures_util::StreamExt;
use serde_json::{Value, json};
use spacejo_api::events::Events;
use spacejo_api::handlers::MAX_BODY_BYTES;
use spacejo_api::server::app;
use spacejo_api::{MemoryStore, Store};
use tower::ServiceExt;

fn store() -> Store {
    Store::Memory(MemoryStore::new())
}

async fn send(router: axum::Router, request: Request<Body>) -> Response {
    match router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

async fn body_text(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn body_json(response: Response) -> Value {
    serde_json::from_str(&body_text(response).await).unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

/// Dossier d'interface factice, propre à chaque test (dans le dossier temporaire du système).
fn static_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spacejo-static-{name}-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>SpaceJo</title>",
    )
    .unwrap();
    std::fs::write(dir.join("assets/app-1a2b.js"), "console.log('jeu');").unwrap();
    std::fs::write(dir.join("assets/app-1a2b.js.gz"), [0x1f, 0x8b, 0x08]).unwrap();
    std::fs::write(dir.join("manifest.webmanifest"), "{}").unwrap();
    dir
}

#[tokio::test]
async fn health_answers_without_touching_the_store() {
    let response = send(app(store(), Events::default(), None), get("/api/health")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, json!({ "status": "ok" }));
}

#[tokio::test]
async fn api_routes_reach_the_handlers() {
    let request = Request::post("/api/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let response = send(app(store(), Events::default(), None), request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = body_json(response).await;
    assert_eq!(body["view"]["players"][0]["name"], "Alice");
    assert_eq!(body["token"].as_str().map(str::len), Some(64));
}

#[tokio::test]
async fn api_responses_are_compressed_when_accepted() {
    let request = Request::post("/api/create")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_ENCODING, "gzip")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let response = send(app(store(), Events::default(), None), request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CONTENT_ENCODING], "gzip");
}

#[tokio::test]
async fn unknown_api_route_and_wrong_method_are_refused() {
    let response = send(app(store(), Events::default(), None), get("/api/nope")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["error"]["code"], "not_found");

    let response = send(app(store(), Events::default(), None), get("/api/create")).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn oversized_body_is_refused() {
    let request = Request::post("/api/create")
        .body(Body::from(vec![b'x'; MAX_BODY_BYTES + 1]))
        .unwrap();
    let response = send(app(store(), Events::default(), None), request).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "payload_too_large"
    );
}

#[tokio::test]
async fn without_an_interface_only_the_api_answers() {
    let response = send(app(store(), Events::default(), None), get("/")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_interface_is_served_with_its_cache_rules() {
    let dir = static_dir("cache");
    let router = app(store(), Events::default(), Some(dir.clone()));

    // La page : toujours revalidée.
    let response = send(router.clone(), get("/")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
    assert!(body_text(response).await.contains("SpaceJo"));

    // Une adresse sans extension est une page de l'application : on renvoie index.html.
    let response = send(router.clone(), get("/partie/ABCDEF")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body_text(response).await.contains("SpaceJo"));

    // Un fichier haché : gardé un an.
    let response = send(router.clone(), get("/assets/app-1a2b.js")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    assert!(response.headers().get(header::CONTENT_ENCODING).is_none());

    // Le manifeste, avec son type.
    let response = send(router, get("/manifest.webmanifest")).await;
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/manifest+json"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn the_gzip_version_is_sent_when_accepted() {
    let dir = static_dir("gzip");
    let request = Request::get("/assets/app-1a2b.js")
        .header(header::ACCEPT_ENCODING, "gzip, deflate, br")
        .body(Body::empty())
        .unwrap();
    let response = send(app(store(), Events::default(), Some(dir.clone())), request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_ENCODING], "gzip");
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn missing_files_and_escapes_are_not_found() {
    let dir = static_dir("escape");
    let router = app(store(), Events::default(), Some(dir.clone()));
    for uri in ["/missing.png", "/../Cargo.toml", "/assets/../../etc/passwd"] {
        let response = send(router.clone(), get(uri)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
    let request = Request::post("/").body(Body::empty()).unwrap();
    let response = send(router, request).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    std::fs::remove_dir_all(dir).unwrap();
}

/// Lit le prochain morceau d'un flux d'évènements (en ignorant les commentaires de maintien).
async fn next_event(body: &mut axum::body::BodyDataStream) -> String {
    loop {
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(2), body.next())
            .await
            .expect("un évènement attendu")
            .expect("flux encore ouvert")
            .unwrap();
        let text = String::from_utf8(chunk.to_vec()).unwrap();
        if !text.starts_with(':') {
            return text;
        }
    }
}

#[tokio::test]
async fn game_changes_are_pushed_to_subscribers() {
    let events = Events::default();
    let router = app(store(), events.clone(), None);

    // Alice crée une partie.
    let request = Request::post("/api/create")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let created = body_json(send(router.clone(), request).await).await;
    let code = created["code"].as_str().unwrap().to_string();

    // Un navigateur s'abonne aux avis de cette partie.
    let response = send(router.clone(), get(&format!("/api/events?code={code}"))).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let mut stream = response.into_body().into_data_stream();

    // Bob rejoint : la partie passe à la version 2, l'avis arrive aussitôt.
    let request = Request::post("/api/join")
        .body(Body::from(
            json!({ "code": code, "name": "Bob" }).to_string(),
        ))
        .unwrap();
    assert_eq!(send(router.clone(), request).await.status(), StatusCode::OK);
    let event = next_event(&mut stream).await;
    assert!(event.contains("event: version"), "{event}");
    assert!(event.contains("data: 2"), "{event}");

    // Un polling (même version) ne republie rien ; une erreur non plus.
    let request = Request::post("/api/join")
        .body(Body::from(json!({ "code": code, "name": "" }).to_string()))
        .unwrap();
    assert_eq!(
        send(router, request).await.status(),
        StatusCode::BAD_REQUEST
    );
    events.publish(&code, 3);
    assert!(next_event(&mut stream).await.contains("data: 3"));

    // À l'arrêt du serveur, le flux se termine.
    events.close_all();
    let end = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next()).await;
    assert!(matches!(end, Ok(None)));
}

#[tokio::test]
async fn events_need_a_valid_code() {
    let response = send(
        app(store(), Events::default(), None),
        get("/api/events?code=0O"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["error"]["code"], "invalid_code");
}

/// Création de partie envoyée depuis l'adresse `ip` (en-tête de Cloudflare, comme en production).
fn create_from(ip: &str) -> Request<Body> {
    Request::post("/api/create")
        .header(header::CONTENT_TYPE, "application/json")
        .header("cf-connecting-ip", ip)
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap()
}

#[tokio::test]
async fn too_many_creations_from_one_address_are_refused_with_retry_after() {
    use spacejo_api::rate_limit::CREATE_PER_MINUTE;
    let router = app(store(), Events::default(), None);
    for _ in 0..CREATE_PER_MINUTE {
        let response = send(router.clone(), create_from("198.51.100.7")).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    let response = send(router.clone(), create_from("198.51.100.7")).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u64 = response.headers()[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=60).contains(&retry_after));
    assert_eq!(body_json(response).await["error"]["code"], "rate_limited");

    // Une autre adresse n'est pas concernée.
    let response = send(router, create_from("198.51.100.8")).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn chat_messages_are_pushed_to_subscribers_without_their_text() {
    let events = Events::default();
    let router = app(store(), events.clone(), None);
    let request = Request::post("/api/create")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let created = body_json(send(router.clone(), request).await).await;
    let code = created["code"].as_str().unwrap().to_string();
    let token = created["token"].as_str().unwrap().to_string();

    let response = send(router.clone(), get(&format!("/api/events?code={code}"))).await;
    let mut stream = response.into_body().into_data_stream();

    let request = Request::post("/api/chat")
        .header("X-Player-Token", &token)
        .body(Body::from(
            json!({ "code": code, "text": "secret bien gardé" }).to_string(),
        ))
        .unwrap();
    let sent = body_json(send(router, request).await).await;
    let id = sent["message"]["id"].as_i64().unwrap();

    // L'avis ne porte que le numéro du message, jamais son texte.
    let event = next_event(&mut stream).await;
    assert!(event.contains("event: chat"), "{event}");
    assert!(event.contains(&format!("data: {id}")), "{event}");
    assert!(!event.contains("secret"), "{event}");
}

#[tokio::test]
async fn reactions_are_pushed_as_reaction_events_not_as_new_messages() {
    let events = Events::default();
    let router = app(store(), events.clone(), None);
    let request = Request::post("/api/create")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let created = body_json(send(router.clone(), request).await).await;
    let code = created["code"].as_str().unwrap().to_string();
    let token = created["token"].as_str().unwrap().to_string();
    let request = Request::post("/api/chat")
        .header("X-Player-Token", &token)
        .body(Body::from(
            json!({ "code": code, "text": "Salut" }).to_string(),
        ))
        .unwrap();
    let sent = body_json(send(router.clone(), request).await).await;
    let id = sent["message"]["id"].as_i64().unwrap();

    // Abonné APRÈS l'envoi du message : le premier avis reçu est celui de la réaction.
    let response = send(router.clone(), get(&format!("/api/events?code={code}"))).await;
    let mut stream = response.into_body().into_data_stream();
    let request = Request::post("/api/react")
        .header("X-Player-Token", &token)
        .body(Body::from(
            json!({ "code": code, "message": id, "reaction": "heart" }).to_string(),
        ))
        .unwrap();
    assert_eq!(send(router, request).await.status(), StatusCode::OK);

    let event = next_event(&mut stream).await;
    assert!(event.contains("event: reaction"), "{event}");
    assert!(event.contains(&format!("data: {id}")), "{event}");
}
