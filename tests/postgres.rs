//! Tests sur une vraie base PostgreSQL (critères 12 et 18).
//!
//! Ils ont besoin de `DATABASE_URL` (un utilisateur qui a le droit de créer des bases). Chaque
//! test crée sa propre base vide `spacejo_test_<aléatoire>`, y applique les migrations, puis la
//! supprime. Sans `DATABASE_URL`, les tests s'arrêtent avec un message ; si
//! `SPACEJO_REQUIRE_DB=1` (c'est le cas dans la CI), l'absence de base est une erreur.
mod common;

use serde_json::json;
use spacejo_api::store::{StoreError, run_migrations};
use spacejo_api::{Endpoint, PgStore, Store};
use sqlx::postgres::PgConnection;
use sqlx::{AssertSqlSafe, Connection};

/// Base de test jetable.
struct TestDb {
    admin_url: String,
    name: String,
    store: PgStore,
}

impl TestDb {
    async fn create() -> Option<TestDb> {
        let Ok(admin_url) = std::env::var("DATABASE_URL") else {
            if std::env::var("SPACEJO_REQUIRE_DB").as_deref() == Ok("1") {
                panic!("DATABASE_URL est obligatoire quand SPACEJO_REQUIRE_DB=1");
            }
            eprintln!("DATABASE_URL absente : test PostgreSQL non exécuté (voir README).");
            return None;
        };
        let name = format!(
            "spacejo_test_{}",
            spacejo_api::auth::generate_code().unwrap().to_lowercase()
        );
        let mut admin = PgConnection::connect(&admin_url).await.unwrap();
        // Nom tiré de l'alphabet des codes (lettres et chiffres) : pas d'injection SQL possible.
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .execute(&mut admin)
            .await
            .unwrap();
        admin.close().await.unwrap();
        let url = replace_database(&admin_url, &name);
        let store = PgStore::connect_lazy(&url).unwrap();
        Some(TestDb {
            admin_url,
            name,
            store,
        })
    }

    async fn drop(self) {
        self.store.pool().close().await;
        let mut admin = PgConnection::connect(&self.admin_url).await.unwrap();
        sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS {} WITH (FORCE)",
            self.name
        )))
        .execute(&mut admin)
        .await
        .unwrap();
    }
}

/// Remplace le nom de la base dans une URL `postgres://…/base?options`.
fn replace_database(url: &str, name: &str) -> String {
    let (base, options) = match url.split_once('?') {
        Some((base, options)) => (base, format!("?{options}")),
        None => (url, String::new()),
    };
    let prefix = match base.rfind('/') {
        Some(i) if i > "postgres://".len() => &base[..i],
        _ => base,
    };
    format!("{prefix}/{name}{options}")
}

#[tokio::test]
async fn migrations_create_the_schema_on_an_empty_database() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    let pool = db.store.pool().clone();

    let before: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'games'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(before.0, 0, "base vide au départ");

    run_migrations(&pool).await.unwrap();
    // Une seconde exécution ne fait rien (migration déjà notée dans _sqlx_migrations).
    run_migrations(&pool).await.unwrap();

    let columns: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name::text, data_type::text FROM information_schema.columns \
         WHERE table_name = 'games' ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let expected = [
        ("id", "bigint"),
        ("code", "text"),
        ("state", "jsonb"),
        ("version", "integer"),
        ("updated_at", "timestamp with time zone"),
        ("public", "boolean"),
    ];
    let columns: Vec<(&str, &str)> = columns
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    assert_eq!(columns, expected);
    db.drop().await;
}

#[tokio::test]
async fn optimistic_lock_lets_only_one_concurrent_write_succeed() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    run_migrations(db.store.pool()).await.unwrap();
    let store = Store::Postgres(db.store.clone());

    let record = store
        .insert("K7MPQX", &json!({ "n": 0 }), false)
        .await
        .unwrap();
    assert_eq!(record.version, 1);
    assert!(matches!(
        store.insert("K7MPQX", &json!({}), false).await,
        Err(StoreError::CodeTaken)
    ));

    // Deux écritures lancées en même temps sur la même version lue.
    let (state_a, state_b) = (json!({ "n": "a" }), json!({ "n": "b" }));
    let (a, b) = tokio::join!(
        store.save(record.id, &state_a, 1),
        store.save(record.id, &state_b, 1),
    );
    let ok = [&a, &b].iter().filter(|r| r.is_ok()).count();
    let conflicts = [&a, &b]
        .iter()
        .filter(|r| matches!(r, Err(StoreError::Conflict)))
        .count();
    assert_eq!((ok, conflicts), (1, 1), "{a:?} / {b:?}");

    let saved = store.load("K7MPQX").await.unwrap().unwrap();
    assert_eq!(saved.version, 2);
    let winner = if a.is_ok() { "a" } else { "b" };
    assert_eq!(
        saved.state,
        json!({ "n": winner }),
        "seule l'écriture gagnante est en base"
    );
    db.drop().await;
}

#[tokio::test]
async fn concurrent_api_actions_on_postgres_only_one_succeeds() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    run_migrations(db.store.pool()).await.unwrap();
    let store = Store::Postgres(db.store.clone());

    let (host, _) = common::create(&store, "Alice", None).await;
    let (guest, _) = common::join(&store, &host.code, "Bob").await;
    assert_eq!(common::start(&store, &host).await.status, 200);
    for client in [&host, &guest] {
        for (column, row) in [(0, 0), (0, 1)] {
            let v = common::version(&store, client).await;
            let action = json!({ "type": "RevealInitial", "column": column, "row": row });
            assert_eq!(common::act(&store, client, v, action).await.status, 200);
        }
    }
    let state = common::get_state(&store, &format!("code={}", host.code), Some(&host.token)).await;
    let v = state.body["version"].as_i64().unwrap();
    let current = if state.body["view"]["current_player"] == 0 {
        &host
    } else {
        &guest
    };

    let (a, b) = tokio::join!(
        common::act(&store, current, v, json!({ "type": "DrawFromPile" })),
        common::act(&store, current, v, json!({ "type": "TakeDiscard" })),
    );
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    assert_eq!(common::version(&store, &host).await, v + 1);

    // Une requête avec une version périmée ne change rien en base.
    let stale = common::call(
        &store,
        Endpoint::Start,
        json!({ "code": host.code, "expected_version": v }),
        Some(&host.token),
    )
    .await;
    assert_eq!(stale.status, 409);
    db.drop().await;
}

#[test]
fn database_name_is_replaced_in_urls() {
    assert_eq!(
        replace_database("postgres://u:p@h:5432/postgres", "x"),
        "postgres://u:p@h:5432/x"
    );
    assert_eq!(
        replace_database("postgres://u@h/db?sslmode=require", "x"),
        "postgres://u@h/x?sslmode=require"
    );
}

#[tokio::test]
async fn presence_is_recorded_without_touching_the_game() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    run_migrations(db.store.pool()).await.unwrap();
    let store = Store::Postgres(db.store.clone());
    let record = store
        .insert("K7MPQX", &json!({ "n": 0 }), false)
        .await
        .unwrap();
    assert!(store.presence(record.id).await.unwrap().is_empty());

    store.touch(record.id, 0).await.unwrap();
    store.touch(record.id, 3).await.unwrap();
    store.touch(record.id, 0).await.unwrap(); // deuxième passage : mise à jour, pas de doublon
    let presence = store.presence(record.id).await.unwrap();
    assert_eq!(presence.len(), 2);
    assert!(presence.values().all(|&idle| (0.0..5.0).contains(&idle)));

    // La partie n'a pas bougé : même version, et son temps d'inactivité est mesuré par PostgreSQL.
    let loaded = store.load("K7MPQX").await.unwrap().unwrap();
    assert_eq!(loaded.version, 1);
    assert!(loaded.idle_seconds >= 0.0);
    db.drop().await;
}

#[tokio::test]
async fn touch_presence_records_and_reads_in_one_query() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    run_migrations(db.store.pool()).await.unwrap();
    let store = Store::Postgres(db.store.clone());
    let record = store
        .insert("K7MPQX", &json!({ "n": 0 }), false)
        .await
        .unwrap();

    // Premier passage : seul le joueur lui-même, à 0 seconde.
    let presence = store.touch_presence(record.id, 2).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert_eq!(presence.get(&2), Some(&0.0));

    // Un autre joueur : il voit le premier, et lui-même à 0.
    let presence = store.touch_presence(record.id, 0).await.unwrap();
    assert_eq!(presence.len(), 2);
    assert_eq!(presence.get(&0), Some(&0.0));
    assert!(
        presence
            .get(&2)
            .is_some_and(|&idle| (0.0..5.0).contains(&idle))
    );

    // Repasser ne crée pas de doublon, et la lecture séparée donne les mêmes joueurs.
    store.touch_presence(record.id, 2).await.unwrap();
    assert_eq!(store.presence(record.id).await.unwrap().len(), 2);
    db.drop().await;
}

#[tokio::test]
async fn only_public_lobbies_with_their_host_present_are_listed() {
    let Some(db) = TestDb::create().await else {
        return;
    };
    run_migrations(db.store.pool()).await.unwrap();
    let store = Store::Postgres(db.store.clone());
    let list = || async {
        let request = spacejo_api::ApiRequest {
            method: "GET".to_string(),
            ..Default::default()
        };
        let response = spacejo_api::handle(&store, Endpoint::PublicGames, request).await;
        assert_eq!(response.status, 200, "{:?}", response.body);
        response.body["games"].as_array().unwrap().clone()
    };

    // Une partie privée et une partie publique : seule la seconde est listée.
    common::create(&store, "Alice", None).await;
    let created = common::call(
        &store,
        Endpoint::Create,
        json!({ "name": "Carla", "public": true }),
        None,
    )
    .await;
    assert_eq!(created.status, 201, "{:?}", created.body);
    let code = created.body["code"].as_str().unwrap().to_string();
    let games = list().await;
    assert_eq!(games.len(), 1);
    assert_eq!(games[0]["code"], code.as_str());
    assert_eq!(games[0]["host"], "Carla");
    assert_eq!(games[0]["players"], 1);

    // L'hôte n'est plus là depuis longtemps : la partie disparaît de la liste.
    sqlx::query("UPDATE presence SET last_seen = now() - interval '1 hour'")
        .execute(db.store.pool())
        .await
        .unwrap();
    assert!(list().await.is_empty());

    // L'hôte revient, puis la partie est lancée : elle n'est plus au salon.
    sqlx::query("UPDATE presence SET last_seen = now()")
        .execute(db.store.pool())
        .await
        .unwrap();
    let (_, joined) = common::join(&store, &code, "Bob").await;
    assert_eq!(joined.body["public"], true);
    assert_eq!(list().await[0]["players"], 2);
    let host = common::Client {
        code: code.clone(),
        token: created.body["token"].as_str().unwrap().to_string(),
        player_id: 0,
    };
    assert_eq!(common::start(&store, &host).await.status, 200);
    assert!(list().await.is_empty());
    db.drop().await;
}
