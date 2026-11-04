//! Applique les migrations SQL de `migrations/` sur la base désignée par `DATABASE_URL`.
//! Le serveur les applique aussi à son démarrage ; cet outil sert à les appliquer sans le lancer :
//! `cargo run --bin migrate`.

use spacejo_api::PgStore;
use spacejo_api::server::DATABASE_URL_VAR;
use spacejo_api::store::run_migrations;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var(DATABASE_URL_VAR)
        .map_err(|_| format!("variable d'environnement {DATABASE_URL_VAR} absente"))?;
    let store = PgStore::connect_lazy(&url).map_err(|_| "URL de base de données invalide")?;
    run_migrations(store.pool()).await?;
    println!("Migrations appliquées.");
    Ok(())
}
