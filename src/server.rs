//! Serveur HTTP unique (axum) : l'API sous `/api`, et l'interface construite (`web/dist`) pour tout le reste.
//!
//! Un seul processus, toujours démarré (Render, ou `cargo run` en local). Il ne garde pourtant aucun état de
//! partie en mémoire : chaque requête charge la partie en base, comme avant (règle « serverless » conservée,
//! ce qui permettrait de lancer plusieurs serveurs côte à côte).
//!
//! axum associe une route à une fonction asynchrone (« handler ») qui reçoit des « extracteurs » (l'état
//! partagé, la requête…) et renvoie une réponse : https://docs.rs/axum/0.8/axum/

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Query, Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use tower::ServiceExt;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::auth;
use crate::error::ApiError;
use crate::events::Events;
use crate::handlers::{self, ApiRequest, ApiResponse, Endpoint, MAX_BODY_BYTES, TOKEN_HEADER};
use crate::store::Store;

/// Variable d'environnement de l'URL PostgreSQL ; absente, le stockage est en mémoire.
pub const DATABASE_URL_VAR: &str = "DATABASE_URL";
/// Variable d'environnement du port d'écoute (fournie par Render).
pub const PORT_VAR: &str = "PORT";
/// Port par défaut, attendu par le proxy de Vite en développement.
pub const DEFAULT_PORT: u16 = 3000;
/// Variable d'environnement du dossier de l'interface construite.
pub const STATIC_DIR_VAR: &str = "STATIC_DIR";
/// Dossier par défaut de l'interface construite (`npm --prefix web run build`).
pub const DEFAULT_STATIC_DIR: &str = "web/dist";

/// Fichiers hachés par Vite (`assets/index-AbC123.js`) : leur contenu ne change jamais sous ce nom, le
/// navigateur peut les garder un an.
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";
/// Page, manifeste et service worker : toujours revalidés, pour qu'une nouvelle version arrive aussitôt.
const NO_CACHE: &str = "no-cache";
/// Autres fichiers (icônes, image d'aperçu) : un jour.
const DAY_CACHE: &str = "public, max-age=86400";
/// Intervalle des commentaires de maintien d'un flux d'avis : sans eux, un proxy pourrait couper une
/// connexion restée muette.
const EVENTS_KEEP_ALIVE: Duration = Duration::from_secs(15);

/// État partagé par les routes : le stockage des parties et les avis de changement.
#[derive(Clone)]
struct AppState {
    store: Store,
    events: Events,
}

/// L'application : routes de l'API, puis l'interface (si un dossier est donné) pour tout le reste.
/// `events` est partagé avec l'appelant, qui le ferme à l'arrêt du serveur (`Events::close_all`).
pub fn app(store: Store, events: Events, static_dir: Option<PathBuf>) -> Router {
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/events", get(game_events))
        .route("/api/{*route}", any(api))
        .with_state(AppState { store, events })
        // Réponses JSON compressées (gzip) quand le navigateur l'accepte : l'état d'une partie est relu
        // toutes les 2 secondes. Ce calque ne couvre que les routes ci-dessus, pas les fichiers de
        // l'interface, déjà compressés au build.
        .layer(CompressionLayer::new());
    match static_dir {
        // `move` : la fermeture garde sa propre copie du chemin pour toutes les requêtes à venir.
        Some(dir) => router.fallback(move |request: Request| {
            let dir = dir.clone();
            async move { serve_static(&dir, request).await }
        }),
        None => router.fallback(|| async { json_response(ApiError::not_found().into()) }),
    }
}

/// Contrôle de santé pour la plateforme d'hébergement : répond sans toucher à la base.
async fn health() -> Response {
    json_response(ApiResponse {
        status: 200,
        body: serde_json::json!({ "status": "ok" }),
    })
}

/// Toutes les routes `/api/...` : conversion vers `handlers::handle`, puis réponse JSON. Si la partie a
/// une nouvelle version (un coup, une jonction, un tour d'absent passé pendant un polling…), les
/// navigateurs abonnés sont prévenus aussitôt.
async fn api(State(state): State<AppState>, request: Request) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = match Endpoint::from_path(&path) {
        None => ApiError::not_found().into(),
        Some(endpoint) => match read_request(request).await {
            Ok(api_request) => {
                let response = handlers::handle(&state.store, endpoint, api_request.clone()).await;
                announce(&state.events, &api_request, &response);
                response
            }
            Err(error) => error.into(),
        },
    };
    // Journal minimal : méthode, chemin, statut. Jamais de jeton, de corps ni d'état.
    println!("{method} {path} → {}", response.status);
    json_response(response)
}

/// Publie la version renvoyée par une réponse réussie. Le code de la partie est dans la réponse
/// (création, jonction), dans le corps JSON de la requête, ou dans ses paramètres d'URL (`state`).
fn announce(events: &Events, request: &ApiRequest, response: &ApiResponse) {
    if !(200..300).contains(&response.status) {
        return;
    }
    let Some(version) = response.body["version"]
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
    else {
        return;
    };
    let from_body = serde_json::from_slice::<serde_json::Value>(&request.body)
        .ok()
        .and_then(|body| body["code"].as_str().map(str::to_string));
    let from_query = request
        .query
        .split('&')
        .find_map(|pair| pair.strip_prefix("code=").map(str::to_string));
    let code = response.body["code"]
        .as_str()
        .map(str::to_string)
        .or(from_body)
        .or(from_query);
    if let Some(code) = code.and_then(|code| auth::normalize_code(&code).ok()) {
        events.publish(&code, version);
    }
}

/// `GET /api/events?code=…` : flux d'avis (server-sent events) d'une partie. Chaque changement envoie
/// un évènement `version` avec le nouveau numéro ; le navigateur relit alors l'état (avec son jeton).
/// Pas de jeton ici (`EventSource` ne sait pas envoyer d'en-tête) : le flux ne révèle rien d'autre que
/// le numéro de version.
async fn game_events(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(code) = params
        .get("code")
        .and_then(|code| auth::normalize_code(code).ok())
    else {
        return json_response(ApiError::invalid_code().into());
    };
    let receiver = state.events.subscribe(&code);
    // `unfold` transforme le récepteur en flux : chaque avis reçu devient un évènement SSE ; le flux se
    // termine quand le canal est fermé (arrêt du serveur).
    let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(version) => {
                    let event = Event::default().event("version").data(version.to_string());
                    return Some((Ok::<Event, Infallible>(event), receiver));
                }
                // Abonné trop lent : il a manqué des avis, le prochain suffira.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });
    let headers = [
        (header::CACHE_CONTROL, "no-cache"),
        // Demande aux proxys de ne pas retenir les évènements en mémoire tampon.
        (header::HeaderName::from_static("x-accel-buffering"), "no"),
    ];
    let sse = Sse::new(stream).keep_alive(KeepAlive::new().interval(EVENTS_KEEP_ALIVE));
    (headers, sse).into_response()
}

/// Extrait méthode, paramètres d'URL, jeton et corps (au plus `MAX_BODY_BYTES` : au-delà, 413).
async fn read_request(request: Request) -> Result<ApiRequest, ApiError> {
    let (parts, body) = request.into_parts();
    let token = parts
        .headers
        .get(TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = axum::body::to_bytes(body, MAX_BODY_BYTES)
        .await
        .map_err(|_| ApiError::payload_too_large())?;
    Ok(ApiRequest {
        method: parts.method.as_str().to_string(),
        query: parts.uri.query().unwrap_or("").to_string(),
        token,
        body: body.to_vec(),
    })
}

/// Réponse JSON de l'API, jamais mise en cache (l'état change à chaque coup).
fn json_response(response: ApiResponse) -> Response {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let headers = [
        (header::CONTENT_TYPE, "application/json; charset=utf-8"),
        (header::CACHE_CONTROL, "no-store"),
    ];
    (status, headers, response.body.to_string()).into_response()
}

/// Sert un fichier de l'interface avec `tower-http` (types de contenu, `Last-Modified`, requêtes partielles,
/// version `.gz` préparée au build quand le navigateur accepte gzip). Une adresse sans extension (`/`, ou une
/// page de l'application) reçoit `index.html` ; un fichier absent, 404.
async fn serve_static(dir: &Path, request: Request) -> Response {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let path = request.uri().path().to_string();
    let Some(relative) = safe_relative_path(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let result = if relative.extension().is_some() {
        ServeDir::new(dir)
            .precompressed_gzip()
            .oneshot(request)
            .await
    } else {
        ServeFile::new(dir.join("index.html"))
            .precompressed_gzip()
            .oneshot(request)
            .await
    };
    // Ces services ne peuvent pas échouer (`Infallible`) : un fichier absent donne une réponse 404.
    let mut response = match result {
        Ok(response) => response.map(Body::new),
        Err(never) => match never {},
    };
    if response.status().is_success() {
        let headers = response.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(cache_control(&relative)),
        );
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
    }
    response
}

/// Chemin relatif sans danger : uniquement des noms de dossiers et de fichiers (ni `..`, ni chemin
/// absolu), pour qu'aucune requête ne puisse viser un fichier hors du dossier de l'interface.
fn safe_relative_path(path: &str) -> Option<PathBuf> {
    let relative = Path::new(path.trim_start_matches('/'));
    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
        .then(|| relative.to_path_buf())
}

/// Règle de cache selon le fichier demandé (`relative` : chemin de l'URL, sans le `/` initial).
fn cache_control(relative: &Path) -> &'static str {
    let name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if relative.starts_with("assets") {
        IMMUTABLE_CACHE
    } else if relative.extension().is_none()
        || matches!(name, "index.html" | "sw.js" | "manifest.webmanifest")
    {
        NO_CACHE
    } else {
        DAY_CACHE
    }
}
