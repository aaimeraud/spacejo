//! Stockage des parties, avec verrou optimiste.
//!
//! Deux implémentations derrière un même `enum` :
//! - `Postgres` : la vraie base (table `games`, voir `migrations/`) ;
//! - `Memory` : une table en mémoire, pour les tests et le serveur local sans base.
//!
//! Verrou optimiste : on lit la partie avec son `version`, on applique l'action en mémoire, puis on
//! écrit SEULEMENT si `version` n'a pas changé (`UPDATE ... WHERE id = $1 AND version = $2`). Si
//! quelqu'un a écrit entre-temps, aucune ligne n'est modifiée : c'est un conflit, rien n'est écrit.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Nombre maximal de connexions d'un processus (une fonction Vercel en traite peu à la fois).
pub const MAX_DB_CONNECTIONS: u32 = 5;
/// Délai maximal pour obtenir une connexion.
pub const DB_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Une partie lue en base.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub id: i64,
    pub state: Value,
    pub version: i32,
}

/// Erreurs de stockage.
#[derive(Debug)]
pub enum StoreError {
    /// Le code de partie existe déjà (création : on retire un autre code).
    CodeTaken,
    /// La version a changé depuis la lecture : rien n'a été écrit.
    Conflict,
    /// Erreur de la base. Le détail sert au journal du serveur, jamais au navigateur.
    Database(sqlx::Error),
    /// Verrou de la table en mémoire inutilisable (un autre fil a paniqué en le tenant).
    Poisoned,
}

/// Le stockage utilisé par l'API. `Clone` est bon marché : les deux variantes partagent leurs
/// données (`Arc` pour la mémoire, le pool de connexions pour PostgreSQL).
#[derive(Clone, Debug)]
pub enum Store {
    Memory(MemoryStore),
    Postgres(PgStore),
}

impl Store {
    /// Insère une nouvelle partie, en version 1.
    pub async fn insert(&self, code: &str, state: &Value) -> Result<Record, StoreError> {
        match self {
            Store::Memory(store) => store.insert(code, state),
            Store::Postgres(store) => store.insert(code, state).await,
        }
    }

    /// Lit une partie par son code (`None` si elle n'existe pas).
    pub async fn load(&self, code: &str) -> Result<Option<Record>, StoreError> {
        match self {
            Store::Memory(store) => store.load(code),
            Store::Postgres(store) => store.load(code).await,
        }
    }

    /// Écrit un nouvel état si la version en base vaut toujours `expected_version`.
    /// Renvoie la nouvelle version (`expected_version + 1`).
    pub async fn save(
        &self,
        id: i64,
        state: &Value,
        expected_version: i32,
    ) -> Result<i32, StoreError> {
        match self {
            Store::Memory(store) => store.save(id, state, expected_version),
            Store::Postgres(store) => store.save(id, state, expected_version).await,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Stockage en mémoire
// ---------------------------------------------------------------------------------------------

/// Table en mémoire, partagée entre les clones (`Arc`) et protégée par un verrou (`Mutex`).
/// https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    rows: Arc<Mutex<HashMap<String, Record>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, code: &str, state: &Value) -> Result<Record, StoreError> {
        let mut rows = self.rows.lock().map_err(|_| StoreError::Poisoned)?;
        if rows.contains_key(code) {
            return Err(StoreError::CodeTaken);
        }
        let record = Record {
            id: rows.len() as i64 + 1,
            state: state.clone(),
            version: 1,
        };
        rows.insert(code.to_string(), record.clone());
        Ok(record)
    }

    fn load(&self, code: &str) -> Result<Option<Record>, StoreError> {
        let rows = self.rows.lock().map_err(|_| StoreError::Poisoned)?;
        Ok(rows.get(code).cloned())
    }

    fn save(&self, id: i64, state: &Value, expected_version: i32) -> Result<i32, StoreError> {
        let mut rows = self.rows.lock().map_err(|_| StoreError::Poisoned)?;
        // Même règle que le `WHERE id = $1 AND version = $2` de PostgreSQL.
        match rows
            .values_mut()
            .find(|r| r.id == id && r.version == expected_version)
        {
            Some(record) => {
                record.state = state.clone();
                record.version += 1;
                Ok(record.version)
            }
            None => Err(StoreError::Conflict),
        }
    }

    /// État brut stocké (tests uniquement : vérifier ce qui est réellement enregistré).
    pub fn raw_state(&self, code: &str) -> Option<Value> {
        self.rows.lock().ok()?.get(code).map(|r| r.state.clone())
    }

    /// Remplace l'état brut d'une partie (tests uniquement : préparer une situation précise).
    pub fn put_raw(&self, code: &str, state: Value) -> Option<()> {
        let mut rows = self.rows.lock().ok()?;
        let id = rows.len() as i64 + 1;
        let record = rows.entry(code.to_string()).or_insert(Record {
            id,
            state: Value::Null,
            version: 1,
        });
        record.state = state;
        Some(())
    }
}

// ---------------------------------------------------------------------------------------------
// PostgreSQL
// ---------------------------------------------------------------------------------------------

/// Accès à PostgreSQL par un pool de connexions `sqlx`.
#[derive(Clone, Debug)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// Prépare le pool sans se connecter tout de suite (`connect_lazy`) : la première requête
    /// ouvrira la connexion. L'URL vient de la variable d'environnement `DATABASE_URL`.
    pub fn connect_lazy(database_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_DB_CONNECTIONS)
            .acquire_timeout(DB_ACQUIRE_TIMEOUT)
            .connect_lazy(database_url)
            .map_err(StoreError::Database)?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn insert(&self, code: &str, state: &Value) -> Result<Record, StoreError> {
        let result = sqlx::query_as::<_, (i64, i32)>(
            "INSERT INTO games (code, state, version) VALUES ($1, $2, 1) RETURNING id, version",
        )
        .bind(code)
        .bind(state)
        .fetch_one(&self.pool)
        .await;
        match result {
            Ok((id, version)) => Ok(Record {
                id,
                state: state.clone(),
                version,
            }),
            // Violation de la contrainte UNIQUE sur `code`.
            Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
                Err(StoreError::CodeTaken)
            }
            Err(error) => Err(StoreError::Database(error)),
        }
    }

    async fn load(&self, code: &str) -> Result<Option<Record>, StoreError> {
        let row = sqlx::query_as::<_, (i64, Value, i32)>(
            "SELECT id, state, version FROM games WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        Ok(row.map(|(id, state, version)| Record { id, state, version }))
    }

    async fn save(&self, id: i64, state: &Value, expected_version: i32) -> Result<i32, StoreError> {
        // Le cœur du verrou optimiste : la ligne n'est modifiée que si la version n'a pas bougé.
        let row = sqlx::query_as::<_, (i32,)>(
            "UPDATE games SET state = $1, version = version + 1, updated_at = now() \
             WHERE id = $2 AND version = $3 RETURNING version",
        )
        .bind(state)
        .bind(id)
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        match row {
            Some((version,)) => Ok(version),
            None => Err(StoreError::Conflict),
        }
    }
}

/// Applique les migrations de `migrations/` (embarquées dans le binaire à la compilation).
/// Les migrations déjà appliquées sont notées dans la table `_sqlx_migrations` et sautées.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
