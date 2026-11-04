//! Le serveur HTTP (axum) : routes de l'API, contrôle de santé, fichiers de l'interface.
//! Les requêtes sont envoyées directement au routeur (`oneshot`), sans ouvrir de port.

use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use serde_json::{Value, json};
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
    let response = send(app(store(), None), get("/api/health")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, json!({ "status": "ok" }));
}

#[tokio::test]
async fn api_routes_reach_the_handlers() {
    let request = Request::post("/api/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "name": "Alice" }).to_string()))
        .unwrap();
    let response = send(app(store(), None), request).await;
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
    let response = send(app(store(), None), request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CONTENT_ENCODING], "gzip");
}

#[tokio::test]
async fn unknown_api_route_and_wrong_method_are_refused() {
    let response = send(app(store(), None), get("/api/nope")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["error"]["code"], "not_found");

    let response = send(app(store(), None), get("/api/create")).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn oversized_body_is_refused() {
    let request = Request::post("/api/create")
        .body(Body::from(vec![b'x'; MAX_BODY_BYTES + 1]))
        .unwrap();
    let response = send(app(store(), None), request).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "payload_too_large"
    );
}

#[tokio::test]
async fn without_an_interface_only_the_api_answers() {
    let response = send(app(store(), None), get("/")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_interface_is_served_with_its_cache_rules() {
    let dir = static_dir("cache");
    let router = app(store(), Some(dir.clone()));

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
    let response = send(app(store(), Some(dir.clone())), request).await;
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
    let router = app(store(), Some(dir.clone()));
    for uri in ["/missing.png", "/../Cargo.toml", "/assets/../../etc/passwd"] {
        let response = send(router.clone(), get(uri)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
    let request = Request::post("/").body(Body::empty()).unwrap();
    let response = send(router, request).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    std::fs::remove_dir_all(dir).unwrap();
}
