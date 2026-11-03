//! Applique les migrations SQL de `migrations/` sur la base désignée par `DATABASE_URL`.
//! À lancer avant chaque déploiement qui ajoute une migration : `cargo run --bin migrate`.

use spacejo_api::PgStore;
use spacejo_api::store::run_migrations;
use spacejo_api::vercel::DATABASE_URL_VAR;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var(DATABASE_URL_VAR)
        .map_err(|_| format!("variable d'environnement {DATABASE_URL_VAR} absente"))?;
    let store = PgStore::connect_lazy(&url).map_err(|_| "URL de base de données invalide")?;
    run_migrations(store.pool()).await?;
    println!("Migrations appliquées.");
    Ok(())
}
