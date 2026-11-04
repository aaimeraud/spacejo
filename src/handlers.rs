//! Les neuf opérations de l'API (docs/CONTRACT.md), indépendantes du serveur HTTP (voir `server`).
//!
//! Chaque opération suit le même parcours (serverless : rien n'est gardé en mémoire) :
//! valider la requête → charger la partie → identifier le joueur par son jeton → vérifier
//! `expected_version` → appliquer l'opération du core → sauvegarder avec le verrou optimiste →
//! répondre avec la vue filtrée du joueur.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use spacejo_core::{
    Action, Game, GameError, HOST, INITIAL_REVEALS, MAX_MISSED_TURNS, MAX_PLAYERS, Phase, PlayerId,
    Rules, view_for,
};

use crate::auth;
use crate::error::ApiError;
use std::collections::HashMap;

use crate::store::{Record, Store, StoreError};
use crate::stored::StoredGame;

/// Taille maximale d'un corps de requête (16 Kio : largement assez pour nos petits JSON).
pub const MAX_BODY_BYTES: usize = 16 * 1024;
/// Nombre d'essais quand le serveur doit réécrire lui-même après un conflit (création, jonction).
pub const MAX_WRITE_ATTEMPTS: usize = 5;
/// Nom de l'en-tête qui porte le jeton.
pub const TOKEN_HEADER: &str = "X-Player-Token";
/// Un joueur est absent s'il n'a fait aucune requête depuis ce délai (secondes). L'interface interroge
/// le serveur toutes les 2 s : 15 s laissent une marge pour une connexion lente (ASSUMPTIONS.md, n° 18).
pub const PRESENCE_TIMEOUT_SECS: f64 = 15.0;
/// Attente d'« un tour » pour un joueur absent, depuis la dernière modification de la partie
/// (secondes). Passé ce délai, son tour est passé (ASSUMPTIONS.md, n° 19).
pub const ABSENT_TURN_WAIT_SECS: f64 = 45.0;
/// Nombre maximal de parties données par `GET /api/public-games` (les plus récentes).
pub const PUBLIC_GAMES_LIMIT: usize = 20;

/// Les routes de l'API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Create,
    Join,
    Rules,
    Start,
    Action,
    Skip,
    State,
    PublicGames,
    Visibility,
}

impl Endpoint {
    /// Route correspondant à un chemin (`/api/create` → `Create`), pour le serveur local.
    pub fn from_path(path: &str) -> Option<Endpoint> {
        match path.trim_end_matches('/') {
            "/api/create" => Some(Endpoint::Create),
            "/api/join" => Some(Endpoint::Join),
            "/api/rules" => Some(Endpoint::Rules),
            "/api/start" => Some(Endpoint::Start),
            "/api/action" => Some(Endpoint::Action),
            "/api/skip" => Some(Endpoint::Skip),
            "/api/state" => Some(Endpoint::State),
            "/api/public-games" => Some(Endpoint::PublicGames),
            "/api/visibility" => Some(Endpoint::Visibility),
            _ => None,
        }
    }

    fn method(self) -> &'static str {
        match self {
            Endpoint::State | Endpoint::PublicGames => "GET",
            _ => "POST",
        }
    }
}

/// Une requête HTTP réduite à ce dont l'API a besoin.
#[derive(Clone, Debug, Default)]
pub struct ApiRequest {
    pub method: String,
    /// Partie après le `?` de l'URL (vide s'il n'y en a pas).
    pub query: String,
    /// Valeur de l'en-tête `X-Player-Token`, s'il est présent.
    pub token: Option<String>,
    pub body: Vec<u8>,
}

/// Une réponse : statut HTTP et corps JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct ApiResponse {
    pub status: u16,
    pub body: Value,
}

impl ApiResponse {
    fn ok(body: Value) -> Self {
        Self { status: 200, body }
    }
}

impl From<ApiError> for ApiResponse {
    fn from(error: ApiError) -> Self {
        Self {
            status: error.status,
            body: error.body(),
        }
    }
}

// Corps des requêtes. `deny_unknown_fields` : tout champ inattendu est refusé.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRequest {
    name: String,
    rules: Option<Rules>,
    /// Partie publique (listée sur l'accueil) ; absent = privée.
    public: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JoinRequest {
    code: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesRequest {
    code: String,
    expected_version: i32,
    rules: Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartRequest {
    code: String,
    expected_version: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionRequest {
    code: String,
    expected_version: i32,
    action: Action,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkipRequest {
    code: String,
    expected_version: i32,
    player: PlayerId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VisibilityRequest {
    code: String,
    expected_version: i32,
    public: bool,
}

/// Point d'entrée : traite une requête et renvoie toujours une réponse (succès ou erreur).
pub async fn handle(store: &Store, endpoint: Endpoint, request: ApiRequest) -> ApiResponse {
    let result = if request.method != endpoint.method() {
        Err(ApiError::method_not_allowed())
    } else {
        match endpoint {
            Endpoint::Create => create(store, &request).await,
            Endpoint::Join => join(store, &request).await,
            Endpoint::Rules => update_rules(store, &request).await,
            Endpoint::Start => start(store, &request).await,
            Endpoint::Action => action(store, &request).await,
            Endpoint::Skip => skip(store, &request).await,
            Endpoint::State => state(store, &request).await,
            Endpoint::PublicGames => public_games(store, &request).await,
            Endpoint::Visibility => visibility(store, &request).await,
        }
    };
    match result {
        Ok(response) => response,
        Err(error) => error.into(),
    }
}

// ----- Opérations -----

async fn create(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: CreateRequest = parse_body(&request.body)?;
    // Sans réglages : règles officielles. Les réglages complets sont enregistrés dans la partie.
    let rules = body.rules.unwrap_or_default();
    // Sans précision, la partie est privée : on ne la rend visible de tous que si l'hôte le demande.
    let public = body.public.unwrap_or(false);
    let game = Game::new(&body.name, rules, auth::generate_seed()?)?;
    let token = auth::generate_token()?;
    let stored = StoredGame::new(game, vec![auth::hash_token(&token)]);
    let state = encode(&stored)?;

    // Un code déjà pris (très improbable) : on en tire un autre.
    for _ in 0..MAX_WRITE_ATTEMPTS {
        let code = auth::generate_code()?;
        match store.insert(&code, &state, public).await {
            Ok(record) => {
                let presence = store
                    .touch_presence(record.id, 0)
                    .await
                    .map_err(storage_error)?;
                let mut body = game_payload(&record, &stored, 0, &presence);
                body["code"] = json!(code);
                body["token"] = json!(token);
                body["player_id"] = json!(0);
                return Ok(ApiResponse { status: 201, body });
            }
            Err(StoreError::CodeTaken) => continue,
            Err(error) => return Err(storage_error(error)),
        }
    }
    Err(ApiError::internal())
}

async fn join(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: JoinRequest = parse_body(&request.body)?;
    let code = auth::normalize_code(&body.code)?;
    let token = auth::generate_token()?;
    // Le joueur qui rejoint ne connaît pas encore la version : en cas d'écriture concurrente
    // (deux joueurs rejoignent en même temps), on relit et on réessaie.
    for _ in 0..MAX_WRITE_ATTEMPTS {
        let (record, mut stored) = load(store, &code).await?;
        let player = stored.game.join(&body.name)?;
        stored.token_hashes.push(auth::hash_token(&token));
        match store
            .save(record.id, &encode(&stored)?, record.version)
            .await
        {
            Ok(version) => {
                let presence = store
                    .touch_presence(record.id, player)
                    .await
                    .map_err(storage_error)?;
                let saved = Record {
                    version,
                    idle_seconds: 0.0,
                    ..record
                };
                let mut body = game_payload(&saved, &stored, player, &presence);
                body["code"] = json!(code);
                body["token"] = json!(token);
                body["player_id"] = json!(player);
                return Ok(ApiResponse::ok(body));
            }
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(storage_error(error)),
        }
    }
    Err(ApiError::version_conflict())
}

async fn update_rules(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: RulesRequest = parse_body(&request.body)?;
    write_as_player(
        store,
        request,
        &body.code,
        body.expected_version,
        |game, player| game.update_rules(player, body.rules),
    )
    .await
}

async fn start(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: StartRequest = parse_body(&request.body)?;
    write_as_player(
        store,
        request,
        &body.code,
        body.expected_version,
        |game, player| game.start(player),
    )
    .await
}

async fn action(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: ActionRequest = parse_body(&request.body)?;
    write_as_player(
        store,
        request,
        &body.code,
        body.expected_version,
        |game, player| spacejo_core::apply(game, player, body.action),
    )
    .await
}

/// L'hôte passe, sans attendre la fin du délai, le tour du joueur absent qui bloque la partie.
/// Mêmes conséquences qu'un passage automatique : tour manqué compté, élimination au tour de trop.
async fn skip(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: SkipRequest = parse_body(&request.body)?;
    let code = auth::normalize_code(&body.code)?;
    let token = require_token(request)?;
    let (record, mut stored) = load(store, &code).await?;
    let player = identify(&stored, token)?;
    if body.expected_version != record.version {
        return Err(ApiError::version_conflict());
    }
    if player != HOST {
        return Err(GameError::NotHost.into());
    }
    // Seul le joueur absent qui bloque la partie peut être passé : jamais un joueur présent.
    let presence = store.presence(record.id).await.map_err(storage_error)?;
    if blocking_absent_player(&stored.game, &presence) != Some(body.player) {
        return Err(ApiError::player_not_absent());
    }
    stored.game.skip_absent(body.player)?;
    save_as_player(store, record, &stored, player).await
}

async fn state(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let mut code = None;
    let mut known_version = None;
    for pair in request.query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "code" => code = Some(auth::normalize_code(value)?),
            "known_version" => {
                let parsed = value
                    .parse::<i32>()
                    .map_err(|_| ApiError::invalid_request("known_version doit être un entier"))?;
                known_version = Some(parsed);
            }
            other => {
                return Err(ApiError::invalid_request(format!(
                    "paramètre inconnu « {other} »"
                )));
            }
        }
    }
    let code = code.ok_or_else(|| ApiError::invalid_request("paramètre « code » manquant"))?;
    let token = require_token(request)?;
    let (record, stored) = load(store, &code).await?;
    let player = identify(&stored, token)?;
    let presence = store
        .touch_presence(record.id, player)
        .await
        .map_err(storage_error)?;
    // Le polling des joueurs présents fait avancer la partie quand un absent bloque son tour.
    let (record, stored) = skip_absent_player(store, &code, record, stored, &presence).await?;
    let mut body = game_payload(&record, &stored, player, &presence);
    // Rien n'a changé depuis la dernière lecture du client : réponse minimale (sans la vue), mais
    // avec la présence, qui change sans modifier la partie.
    let changed = known_version != Some(record.version);
    body["changed"] = json!(changed);
    if !changed && let Some(object) = body.as_object_mut() {
        object.remove("view");
    }
    Ok(ApiResponse::ok(body))
}

/// L'hôte rend la partie publique ou privée, au salon seulement. La visibilité n'est pas une règle du jeu
/// (elle vit dans la colonne `games.public`, pas dans `Game`) : les vérifications d'hôte et de phase sont donc
/// faites ici, avec les mêmes erreurs que pour les réglages.
async fn visibility(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let body: VisibilityRequest = parse_body(&request.body)?;
    let code = auth::normalize_code(&body.code)?;
    let token = require_token(request)?;
    let (record, stored) = load(store, &code).await?;
    let player = identify(&stored, token)?;
    if body.expected_version != record.version {
        return Err(ApiError::version_conflict());
    }
    if player != HOST {
        return Err(GameError::NotHost.into());
    }
    if stored.game.phase != Phase::Lobby {
        return Err(GameError::WrongPhase.into());
    }
    let version = match store
        .set_public(record.id, body.public, record.version)
        .await
    {
        Ok(version) => version,
        Err(StoreError::Conflict) => return Err(ApiError::version_conflict()),
        Err(error) => return Err(storage_error(error)),
    };
    let presence = store
        .touch_presence(record.id, player)
        .await
        .map_err(storage_error)?;
    let saved = Record {
        version,
        idle_seconds: 0.0,
        public: body.public,
        ..record
    };
    Ok(ApiResponse::ok(game_payload(
        &saved, &stored, player, &presence,
    )))
}

/// Parties publiques que l'on peut rejoindre, pour l'accueil. Sans jeton : on n'y donne que le code, le nom
/// de l'hôte et le nombre de joueurs (aucune carte, aucun jeton). Seul l'hôte présent garde sa partie dans
/// la liste : un salon abandonné disparaît de lui-même.
async fn public_games(store: &Store, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    if !request.query.is_empty() {
        return Err(ApiError::invalid_request(
            "cette route n'accepte aucun paramètre",
        ));
    }
    let rows = store
        .list_public(PRESENCE_TIMEOUT_SECS, MAX_PLAYERS, PUBLIC_GAMES_LIMIT)
        .await
        .map_err(storage_error)?;
    // Une partie illisible (format inconnu) est simplement omise de la liste.
    let games: Vec<Value> = rows
        .into_iter()
        .filter_map(|row| {
            let stored = StoredGame::decode(row.state).ok()?;
            let host = stored.game.players.get(HOST)?;
            Some(json!({
                "code": row.code,
                "host": host.name,
                "players": stored.game.players.len(),
                "max_players": MAX_PLAYERS,
            }))
        })
        .collect();
    Ok(ApiResponse::ok(json!({ "games": games })))
}

// ----- Briques communes -----

/// Parcours commun de `rules`, `start` et `action` : jeton, version attendue, opération du
/// core, sauvegarde avec verrou optimiste, puis vue filtrée.
///
/// `operation` est une fermeture (closure) qui reçoit la partie à modifier et le joueur.
/// https://doc.rust-lang.org/stable/book/ch13-01-closures.html
async fn write_as_player<F>(
    store: &Store,
    request: &ApiRequest,
    code: &str,
    expected_version: i32,
    operation: F,
) -> Result<ApiResponse, ApiError>
where
    F: FnOnce(&mut Game, PlayerId) -> Result<(), spacejo_core::GameError>,
{
    let code = auth::normalize_code(code)?;
    let token = require_token(request)?;
    let (record, mut stored) = load(store, &code).await?;
    let player = identify(&stored, token)?;
    // Écran périmé ou requête rejouée : refus, rien n'est écrit.
    if expected_version != record.version {
        return Err(ApiError::version_conflict());
    }
    operation(&mut stored.game, player)?;
    save_as_player(store, record, &stored, player).await
}

/// Sauvegarde la partie modifiée (verrou optimiste), note la présence du joueur et lui renvoie sa vue.
async fn save_as_player(
    store: &Store,
    record: Record,
    stored: &StoredGame,
    player: PlayerId,
) -> Result<ApiResponse, ApiError> {
    let version = match store
        .save(record.id, &encode(stored)?, record.version)
        .await
    {
        Ok(version) => version,
        Err(StoreError::Conflict) => return Err(ApiError::version_conflict()),
        Err(error) => return Err(storage_error(error)),
    };
    let presence = store
        .touch_presence(record.id, player)
        .await
        .map_err(storage_error)?;
    let saved = Record {
        version,
        idle_seconds: 0.0,
        ..record
    };
    Ok(ApiResponse::ok(game_payload(
        &saved, stored, player, &presence,
    )))
}

// ----- Présence et joueurs absents -----

/// Le joueur est-il absent ? (Jamais vu, ou aucune requête depuis `PRESENCE_TIMEOUT_SECS`.)
fn is_absent(presence: &HashMap<usize, f64>, player: PlayerId) -> bool {
    presence
        .get(&player)
        .is_none_or(|&idle| idle > PRESENCE_TIMEOUT_SECS)
}

/// Joueur absent qui bloque la partie : celui dont c'est le tour, ou, pendant la mise en place, le
/// premier qui n'a pas encore retourné ses cartes.
fn blocking_absent_player(game: &Game, presence: &HashMap<usize, f64>) -> Option<PlayerId> {
    match &game.phase {
        Phase::Playing(turn) => Some(turn.current).filter(|&p| is_absent(presence, p)),
        Phase::InitialReveal => (0..game.players.len()).find(|&p| {
            let player = &game.players[p];
            !player.eliminated && player.initial_reveals < INITIAL_REVEALS && is_absent(presence, p)
        }),
        _ => None,
    }
}

/// Si un joueur absent bloque la partie depuis plus de `ABSENT_TURN_WAIT_SECS`, passe son tour
/// (et l'élimine au tour manqué de trop). Une seule fois par requête ; en cas d'écriture concurrente,
/// on relit simplement la partie.
async fn skip_absent_player(
    store: &Store,
    code: &str,
    record: Record,
    stored: StoredGame,
    presence: &HashMap<usize, f64>,
) -> Result<(Record, StoredGame), ApiError> {
    if record.idle_seconds < ABSENT_TURN_WAIT_SECS {
        return Ok((record, stored));
    }
    let Some(absent) = blocking_absent_player(&stored.game, presence) else {
        return Ok((record, stored));
    };
    let mut next = stored.clone();
    if next.game.skip_absent(absent).is_err() {
        return Ok((record, stored));
    }
    match store.save(record.id, &encode(&next)?, record.version).await {
        Ok(version) => Ok((
            Record {
                version,
                idle_seconds: 0.0,
                ..record
            },
            next,
        )),
        Err(StoreError::Conflict) => load(store, code).await,
        Err(error) => Err(storage_error(error)),
    }
}

/// Corps commun des réponses qui donnent l'état : version, vue filtrée, présence de chaque joueur et
/// compte à rebours si l'on attend un joueur absent (format : docs/CONTRACT.md). `presence` vient de
/// `Store::touch_presence`, lue avec la présence du joueur lui-même (un seul aller-retour vers la base).
fn game_payload(
    record: &Record,
    stored: &StoredGame,
    player: PlayerId,
    presence: &HashMap<usize, f64>,
) -> Value {
    let connected: Vec<Value> = (0..stored.game.players.len())
        .map(|p| json!({ "player": p, "connected": !is_absent(presence, p) }))
        .collect();
    let waiting = blocking_absent_player(&stored.game, presence).map(|absent| {
        let left = (ABSENT_TURN_WAIT_SECS - record.idle_seconds)
            .max(0.0)
            .ceil();
        json!({
            "player": absent,
            "seconds_left": left as u64,
            "missed_turns": stored.game.players[absent].missed_turns,
            "max_missed_turns": MAX_MISSED_TURNS,
        })
    });
    json!({
        "version": record.version,
        "public": record.public,
        "view": view_for(&stored.game, player),
        "presence": connected,
        "waiting": waiting,
    })
}

/// Lit un corps JSON dans le type attendu ; toute erreur (JSON invalide, champ inconnu ou
/// manquant, valeur d'enum inconnue) devient un `400 invalid_request`.
fn parse_body<T: DeserializeOwned>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body).map_err(ApiError::invalid_request)
}

fn require_token(request: &ApiRequest) -> Result<&str, ApiError> {
    match request.token.as_deref() {
        Some(token) if !token.is_empty() => Ok(token),
        _ => Err(ApiError::missing_token()),
    }
}

/// Joueur correspondant au jeton, dans CETTE partie.
fn identify(stored: &StoredGame, token: &str) -> Result<PlayerId, ApiError> {
    auth::find_player(&stored.token_hashes, token).ok_or_else(ApiError::invalid_token)
}

/// Charge et décode une partie.
async fn load(store: &Store, code: &str) -> Result<(Record, StoredGame), ApiError> {
    let record = store
        .load(code)
        .await
        .map_err(storage_error)?
        .ok_or_else(ApiError::game_not_found)?;
    match StoredGame::decode(record.state.clone()) {
        Ok(stored) => Ok((record, stored)),
        Err(error) => {
            // Journal : le message ne contient aucune donnée de la partie.
            eprintln!(
                "état illisible pour la partie {}: {}",
                record.id,
                error.message()
            );
            Err(ApiError::new(
                500,
                "unsupported_state_version",
                "Cette partie a été enregistrée dans un format que ce serveur ne sait pas lire.",
            ))
        }
    }
}

fn encode(stored: &StoredGame) -> Result<Value, ApiError> {
    stored.encode().map_err(|_| ApiError::internal())
}

/// Erreur de stockage → erreur HTTP. Le détail technique va au journal, jamais au navigateur.
fn storage_error(error: StoreError) -> ApiError {
    match error {
        StoreError::Conflict => ApiError::version_conflict(),
        StoreError::CodeTaken => ApiError::internal(),
        StoreError::Database(error) => {
            // On ne journalise que le type d'erreur sqlx, pas les valeurs envoyées.
            eprintln!(
                "erreur de base de données : {}",
                database_error_kind(&error)
            );
            ApiError::internal()
        }
        StoreError::Poisoned => {
            eprintln!("erreur de stockage en mémoire : verrou empoisonné");
            ApiError::internal()
        }
    }
}

fn database_error_kind(error: &sqlx::Error) -> &'static str {
    match error {
        sqlx::Error::Configuration(_) => "configuration",
        sqlx::Error::Database(_) => "erreur renvoyée par PostgreSQL",
        sqlx::Error::Io(_) => "entrée/sortie réseau",
        sqlx::Error::Tls(_) => "TLS",
        sqlx::Error::PoolTimedOut => "délai d'attente du pool dépassé",
        sqlx::Error::PoolClosed => "pool fermé",
        _ => "autre",
    }
}
