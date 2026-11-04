//! # spacejo-api
//!
//! API HTTP de SpaceJo : routes (`handlers`), jetons et codes (`auth`), format enregistré
//! (`stored`), stockage avec verrou optimiste (`store`) et serveur HTTP (`server`, axum).
//! La logique du jeu, elle, est entièrement dans la caisse `spacejo-core`.

pub mod auth;
pub mod error;
pub mod handlers;
pub mod server;
pub mod store;
pub mod stored;

pub use error::ApiError;
pub use handlers::{ApiRequest, ApiResponse, Endpoint, handle};
pub use store::{MemoryStore, PgStore, Store};
pub use stored::{STATE_VERSION, StoredGame};
