//! Stockage des parties, avec verrou optimiste, et présence des joueurs.
//!
//! Deux implémentations derrière un même `enum` :
//! - `Postgres` : la vraie base (tables `games` et `presence`, voir `migrations/`) ;
//! - `Memory` : des tables en mémoire, pour les tests et le serveur local sans base.
//!
//! Verrou optimiste : on lit la partie avec son `version`, on applique l'action en mémoire, puis on
//! écrit SEULEMENT si `version` n'a pas changé (`UPDATE ... WHERE id = $1 AND version = $2`). Si
//! quelqu'un a écrit entre-temps, aucune ligne n'est modifiée : c'est un conflit, rien n'est écrit.
//!
//! Présence : chaque requête d'un joueur note l'heure de son passage (`touch`). C'est le seul endroit
//! de l'API qui dépend de l'horloge ; le core, lui, n'en sait rien.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

/// Nombre maximal de connexions à PostgreSQL du serveur (largement assez pour quelques dizaines de joueurs,
/// et sous la limite du pooler de l'offre gratuite de Supabase).
pub const MAX_DB_CONNECTIONS: u32 = 5;
/// Délai maximal pour obtenir une connexion.
pub const DB_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Une partie lue en base.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub id: i64,
    pub state: Value,
    pub version: i32,
    /// Secondes écoulées depuis la dernière écriture de la partie.
    pub idle_seconds: f64,
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

    /// Note qu'un joueur vient de faire une requête. Ne touche ni l'état ni la version de la partie.
    pub async fn touch(&self, id: i64, player: usize) -> Result<(), StoreError> {
        match self {
            Store::Memory(store) => store.touch(id, player),
            Store::Postgres(store) => store.touch(id, player).await,
        }
    }

    /// `touch` puis `presence` en une seule requête (un seul aller-retour vers la base) : c'est ce que
    /// fait chaque requête d'un joueur identifié. Le joueur qui vient d'être noté a 0 seconde d'inactivité.
    pub async fn touch_presence(
        &self,
        id: i64,
        player: usize,
    ) -> Result<HashMap<usize, f64>, StoreError> {
        match self {
            Store::Memory(store) => {
                store.touch(id, player)?;
                store.presence(id)
            }
            Store::Postgres(store) => store.touch_presence(id, player).await,
        }
    }

    /// Secondes écoulées depuis la dernière requête de chaque joueur vu (les autres sont absents).
    pub async fn presence(&self, id: i64) -> Result<HashMap<usize, f64>, StoreError> {
        match self {
            Store::Memory(store) => store.presence(id),
            Store::Postgres(store) => store.presence(id).await,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Stockage en mémoire
// ---------------------------------------------------------------------------------------------

/// Une partie en mémoire : la ligne de `games`, avec l'instant de la dernière écriture.
#[derive(Clone, Debug)]
struct MemoryRow {
    id: i64,
    state: Value,
    version: i32,
    updated_at: Instant,
}

#[derive(Debug, Default)]
struct MemoryTables {
    games: HashMap<String, MemoryRow>,
    presence: HashMap<(i64, usize), Instant>,
    /// Décalage d'horloge, pour simuler le temps qui passe dans les tests (`advance`).
    skew: Duration,
}

/// Tables en mémoire, partagées entre les clones (`Arc`) et protégées par un verrou (`Mutex`).
/// https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    tables: Arc<Mutex<MemoryTables>>,
}

impl MemoryTables {
    fn now(&self) -> Instant {
        Instant::now() + self.skew
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, code: &str, state: &Value) -> Result<Record, StoreError> {
        let mut tables = self.tables.lock().map_err(|_| StoreError::Poisoned)?;
        if tables.games.contains_key(code) {
            return Err(StoreError::CodeTaken);
        }
        let row = MemoryRow {
            id: tables.games.len() as i64 + 1,
            state: state.clone(),
            version: 1,
            updated_at: tables.now(),
        };
        tables.games.insert(code.to_string(), row.clone());
        Ok(Record {
            id: row.id,
            state: row.state,
            version: row.version,
            idle_seconds: 0.0,
        })
    }

    fn load(&self, code: &str) -> Result<Option<Record>, StoreError> {
        let tables = self.tables.lock().map_err(|_| StoreError::Poisoned)?;
        let now = tables.now();
        Ok(tables.games.get(code).map(|row| Record {
            id: row.id,
            state: row.state.clone(),
            version: row.version,
            idle_seconds: now.duration_since(row.updated_at).as_secs_f64(),
        }))
    }

    fn save(&self, id: i64, state: &Value, expected_version: i32) -> Result<i32, StoreError> {
        let mut tables = self.tables.lock().map_err(|_| StoreError::Poisoned)?;
        let now = tables.now();
        // Même règle que le `WHERE id = $1 AND version = $2` de PostgreSQL.
        match tables
            .games
            .values_mut()
            .find(|r| r.id == id && r.version == expected_version)
        {
            Some(row) => {
                row.state = state.clone();
                row.version += 1;
                row.updated_at = now;
                Ok(row.version)
            }
            None => Err(StoreError::Conflict),
        }
    }

    fn touch(&self, id: i64, player: usize) -> Result<(), StoreError> {
        let mut tables = self.tables.lock().map_err(|_| StoreError::Poisoned)?;
        let now = tables.now();
        tables.presence.insert((id, player), now);
        Ok(())
    }

    fn presence(&self, id: i64) -> Result<HashMap<usize, f64>, StoreError> {
        let tables = self.tables.lock().map_err(|_| StoreError::Poisoned)?;
        let now = tables.now();
        Ok(tables
            .presence
            .iter()
            .filter(|((game, _), _)| *game == id)
            .map(|((_, player), seen)| (*player, now.duration_since(*seen).as_secs_f64()))
            .collect())
    }

    /// Fait avancer l'horloge de ce stockage (tests uniquement : simuler une absence).
    pub fn advance(&self, duration: Duration) {
        if let Ok(mut tables) = self.tables.lock() {
            tables.skew += duration;
        }
    }

    /// État brut stocké (tests uniquement : vérifier ce qui est réellement enregistré).
    pub fn raw_state(&self, code: &str) -> Option<Value> {
        self.tables
            .lock()
            .ok()?
            .games
            .get(code)
            .map(|r| r.state.clone())
    }

    /// Remplace l'état brut d'une partie (tests uniquement : préparer une situation précise).
    pub fn put_raw(&self, code: &str, state: Value) -> Option<()> {
        let mut tables = self.tables.lock().ok()?;
        let id = tables.games.len() as i64 + 1;
        let now = tables.now();
        let row = tables.games.entry(code.to_string()).or_insert(MemoryRow {
            id,
            state: Value::Null,
            version: 1,
            updated_at: now,
        });
        row.state = state;
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
    /// Prépare le pool sans se connecter tout de suite (`connect_lazy_with`) : la première requête
    /// ouvrira la connexion. L'URL vient de la variable d'environnement `DATABASE_URL`.
    ///
    /// sqlx garde les requêtes préparées de chaque connexion en cache : une requête déjà vue part en un
    /// seul aller-retour. Cela suppose une connexion PostgreSQL à soi : directe, ou par le pooler de
    /// Supabase en mode **session** (port 5432), pas en mode transaction (port 6543). Voir
    /// docs/decisions/2026-11-04-deploiement-supabase.md.
    pub fn connect_lazy(database_url: &str) -> Result<Self, StoreError> {
        let options = database_url
            .parse::<PgConnectOptions>()
            .map_err(StoreError::Database)?;
        let pool = PgPoolOptions::new()
            .max_connections(MAX_DB_CONNECTIONS)
            .acquire_timeout(DB_ACQUIRE_TIMEOUT)
            .connect_lazy_with(options);
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
                idle_seconds: 0.0,
            }),
            // Violation de la contrainte UNIQUE sur `code`.
            Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
                Err(StoreError::CodeTaken)
            }
            Err(error) => Err(StoreError::Database(error)),
        }
    }

    async fn load(&self, code: &str) -> Result<Option<Record>, StoreError> {
        // L'heure vient de PostgreSQL (`now()`), la même pour toutes les fonctions.
        let row = sqlx::query_as::<_, (i64, Value, i32, f64)>(
            "SELECT id, state, version, EXTRACT(EPOCH FROM now() - updated_at)::float8 \
             FROM games WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        Ok(row.map(|(id, state, version, idle_seconds)| Record {
            id,
            state,
            version,
            idle_seconds,
        }))
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

    async fn touch(&self, id: i64, player: usize) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO presence (game_id, player_id, last_seen) VALUES ($1, $2, now()) \
             ON CONFLICT (game_id, player_id) DO UPDATE SET last_seen = now()",
        )
        .bind(id)
        .bind(player as i32)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        Ok(())
    }

    async fn touch_presence(
        &self,
        id: i64,
        player: usize,
    ) -> Result<HashMap<usize, f64>, StoreError> {
        // Une requête avec une CTE qui écrit (`WITH ... INSERT`) : la mise à jour de la présence et la
        // lecture de celle des autres partent ensemble. La lecture ne voit pas l'écriture de la même
        // requête (même instantané) : le joueur noté est donc ajouté à part, avec 0 seconde.
        let rows = sqlx::query_as::<_, (i32, f64)>(
            "WITH touched AS ( \
               INSERT INTO presence (game_id, player_id, last_seen) VALUES ($1, $2, now()) \
               ON CONFLICT (game_id, player_id) DO UPDATE SET last_seen = now() \
               RETURNING player_id \
             ) \
             SELECT player_id, EXTRACT(EPOCH FROM now() - last_seen)::float8 \
             FROM presence WHERE game_id = $1 AND player_id <> $2 \
             UNION ALL SELECT player_id, 0::float8 FROM touched",
        )
        .bind(id)
        .bind(player as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        Ok(rows
            .into_iter()
            .filter_map(|(player, idle)| usize::try_from(player).ok().map(|p| (p, idle)))
            .collect())
    }

    async fn presence(&self, id: i64) -> Result<HashMap<usize, f64>, StoreError> {
        let rows = sqlx::query_as::<_, (i32, f64)>(
            "SELECT player_id, EXTRACT(EPOCH FROM now() - last_seen)::float8 \
             FROM presence WHERE game_id = $1",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::Database)?;
        Ok(rows
            .into_iter()
            .filter_map(|(player, idle)| usize::try_from(player).ok().map(|p| (p, idle)))
            .collect())
    }
}

/// Applique les migrations de `migrations/` (embarquées dans le binaire à la compilation).
/// Les migrations déjà appliquées sont notées dans la table `_sqlx_migrations` et sautées.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
