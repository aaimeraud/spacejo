//! Adaptateur entre le runtime Rust de Vercel (`vercel_runtime`, bêta) et `handlers`.
//!
//! `vercel_runtime::run` démarre un petit serveur HTTP (hyper) qui reçoit les requêtes de la
//! plateforme. Chaque fichier de `api/` appelle `serve` avec sa route ; le serveur local
//! (`src/bin/dev_server.rs`) utilise `router`, qui choisit la route d'après le chemin.

use http_body_util::{BodyExt, Limited};
use vercel_runtime::{Error, Request, Response, ResponseBody, run, service_fn};

use crate::error::ApiError;
use crate::handlers::{self, ApiRequest, ApiResponse, Endpoint, MAX_BODY_BYTES, TOKEN_HEADER};
use crate::store::{PgStore, Store};

/// Nom de la variable d'environnement qui contient l'URL de connexion PostgreSQL.
pub const DATABASE_URL_VAR: &str = "DATABASE_URL";

/// Lance une fonction Vercel pour une route donnée, avec PostgreSQL.
pub async fn serve(endpoint: Endpoint) -> Result<(), Error> {
    let url = std::env::var(DATABASE_URL_VAR)
        .map_err(|_| format!("variable d'environnement {DATABASE_URL_VAR} absente"))?;
    let store = Store::Postgres(
        PgStore::connect_lazy(&url).map_err(|_| "URL de base de données invalide")?,
    );
    // `move` : la fermeture prend possession de son propre clone du stockage.
    run(service_fn(move |request: Request| {
        let store = store.clone();
        async move { respond(&store, Some(endpoint), request).await }
    }))
    .await
}

/// Variante pour le serveur local : la route est déduite du chemin de l'URL.
pub async fn router(store: Store, request: Request) -> Result<Response<ResponseBody>, Error> {
    let endpoint = Endpoint::from_path(request.uri().path());
    respond(&store, endpoint, request).await
}

/// Convertit la requête hyper, appelle `handlers::handle`, puis convertit la réponse.
async fn respond(
    store: &Store,
    endpoint: Option<Endpoint>,
    request: Request,
) -> Result<Response<ResponseBody>, Error> {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    let response = match endpoint {
        None => ApiResponse::from(ApiError::not_found()),
        Some(endpoint) => match read_request(request).await {
            Ok(api_request) => handlers::handle(store, endpoint, api_request).await,
            Err(error) => error.into(),
        },
    };
    // Journal minimal : méthode, chemin, statut. Jamais de jeton, de corps ni d'état.
    println!("{method} {path} → {}", response.status);
    let body = serde_json::to_string(&response.body)?;
    let http_response = Response::builder()
        .status(response.status)
        .header("content-type", "application/json; charset=utf-8")
        // L'état change à chaque coup : aucun cache intermédiaire.
        .header("cache-control", "no-store")
        .body(ResponseBody::from(body))?;
    Ok(http_response)
}

/// Extrait méthode, paramètres d'URL, jeton et corps (avec une taille maximale).
async fn read_request(request: Request) -> Result<ApiRequest, ApiError> {
    let (parts, body) = request.into_parts();
    let token = parts
        .headers
        .get(TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let query = parts.uri.query().unwrap_or("").to_string();
    // `Limited` coupe la lecture au-delà de MAX_BODY_BYTES : un corps énorme est refusé (413)
    // sans être chargé en mémoire.
    let body = Limited::new(body, MAX_BODY_BYTES)
        .collect()
        .await
        .map_err(|_| ApiError::payload_too_large())?
        .to_bytes()
        .to_vec();
    Ok(ApiRequest {
        method: parts.method.as_str().to_string(),
        query,
        token,
        body,
    })
}
