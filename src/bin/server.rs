//! Serveur SpaceJo : l'API et l'interface dans un seul processus, en local comme en production (Render).
//!
//! - Avec `DATABASE_URL` : PostgreSQL ; les migrations sont appliquées au démarrage.
//! - Sans `DATABASE_URL` : stockage en mémoire (tout est perdu à l'arrêt), pratique en développement.
//! - `PORT` : port d'écoute (3000 par défaut, fourni par Render en production).
//! - `STATIC_DIR` : dossier de l'interface construite (`web/dist` par défaut) ; absent, seule l'API répond.
//!
//! Lancement : `cargo run` (voir README).

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use spacejo_api::events::Events;
use spacejo_api::server::{
    DATABASE_URL_VAR, DEFAULT_PORT, DEFAULT_STATIC_DIR, PORT_VAR, STATIC_DIR_VAR, app,
};
use spacejo_api::store::run_migrations;
use spacejo_api::{MemoryStore, PgStore, Store};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    // Ménage au démarrage : les chats des parties finies ou abandonnées sont effacés (ASSUMPTIONS.md, n° 40).
    spacejo_api::handlers::purge_old_chats(&store).await;

    let static_dir = PathBuf::from(
        std::env::var(STATIC_DIR_VAR).unwrap_or_else(|_| DEFAULT_STATIC_DIR.to_string()),
    );
    let static_dir = if static_dir.join("index.html").is_file() {
        println!("Interface : {}", static_dir.display());
        Some(static_dir)
    } else {
        println!(
            "Interface : non servie ({} introuvable)",
            static_dir.display()
        );
        None
    };

    let port = match std::env::var(PORT_VAR) {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|_| format!("{PORT_VAR} invalide : {value}"))?,
        Err(_) => DEFAULT_PORT,
    };
    // 0.0.0.0 : accepte les connexions venues de l'extérieur du conteneur (obligatoire sur Render).
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("SpaceJo écoute sur le port {port}");
    let events = Events::default();
    let closing = events.clone();
    // `into_make_service_with_connect_info` : chaque requête connaît l'adresse de la connexion (limitation de
    // débit, quand l'en-tête de Cloudflare est absent, en local).
    let service =
        app(store, events, static_dir).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            // Termine les flux d'avis encore ouverts : sinon l'arrêt les attendrait sans fin.
            closing.close_all();
        })
        .await?;
    println!("Arrêt du serveur.");
    Ok(())
}

/// Attend Ctrl+C ou SIGTERM (envoyé par Render avant d'arrêter une instance) : les requêtes en cours se
/// terminent avant l'arrêt.
async fn shutdown_signal() {
    let ctrl_c = async {
        // Si l'écoute du signal échoue, on attend indéfiniment : SIGTERM reste possible.
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
