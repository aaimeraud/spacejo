//! Serveur local de développement : toutes les routes de `api/` dans un seul processus.
//!
//! - Avec `DATABASE_URL` : PostgreSQL (les migrations sont appliquées au démarrage).
//! - Sans `DATABASE_URL` : stockage en mémoire (tout est perdu à l'arrêt).
//!
//! Port : variable `VERCEL_DEV_PORT` (lue par `vercel_runtime`), 3000 par défaut.
//! Lancement : `cargo run --bin dev_server` (voir README).

use spacejo_api::store::run_migrations;
use spacejo_api::vercel::{DATABASE_URL_VAR, router};
use spacejo_api::{MemoryStore, PgStore, Store};
use vercel_runtime::{Error, Request, run, service_fn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let store = match std::env::var(DATABASE_URL_VAR) {
        Ok(url) => {
            let store =
                PgStore::connect_lazy(&url).map_err(|_| "URL de base de données invalide")?;
            run_migrations(store.pool()).await?;
            println!("Stockage : PostgreSQL (migrations appliquées)");
            Store::Postgres(store)
        }
        Err(_) => {
            println!("Stockage : mémoire ({DATABASE_URL_VAR} absente, rien n'est conservé)");
            Store::Memory(MemoryStore::new())
        }
    };
    run(service_fn(move |request: Request| {
        let store = store.clone();
        async move { router(store, request).await }
    }))
    .await
}
