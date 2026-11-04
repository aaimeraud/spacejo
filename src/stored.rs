//! Format de la partie enregistrée dans la colonne `state` (JSONB), et sa version.
//!
//! Toute évolution du format de `Game` (ou de `StoredGame`) doit incrémenter `STATE_VERSION` et
//! ajouter dans `decode` la migration depuis la version précédente, sinon une mise à jour du
//! serveur casserait les parties en cours. Le test `tests/state_version.rs` relit un exemple
//! figé de chaque version : il échoue si le format change sans migration.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use spacejo_core::Game;

/// Version actuelle du format enregistré.
/// - 1 : format initial ;
/// - 2 : ajout de `missed_turns` et `eliminated` à chaque joueur (absences et élimination) ;
/// - 3 : ajout de `abandoned` à chaque joueur (joueur qui a quitté la partie) ;
/// - 4 : ajout du réglage `deck` (taille du paquet) dans `game.rules`.
pub const STATE_VERSION: u64 = 4;

/// Ce qui est stocké pour une partie : le jeu complet et les hachages des jetons
/// (indice = identifiant du joueur). Jamais envoyé tel quel au navigateur.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredGame {
    pub state_version: u64,
    pub game: Game,
    pub token_hashes: Vec<String>,
}

/// Raisons pour lesquelles un état stocké ne peut pas être relu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateError {
    /// Pas de champ `state_version` : format antérieur au versionnage, inconnu de ce serveur.
    MissingVersion,
    /// Version plus ancienne pour laquelle aucune migration n'existe.
    Unsupported { found: u64 },
    /// Version plus récente que ce serveur (par exemple après un retour à une ancienne version).
    TooNew { found: u64 },
    /// Le JSON annonce la bonne version mais ne correspond pas au format.
    Corrupt,
}

impl StateError {
    /// Message clair (journalisé côté serveur ; ne contient aucune donnée de la partie).
    pub fn message(&self) -> String {
        match self {
            StateError::MissingVersion => {
                "partie stockée sans state_version : format inconnu, refusé".to_string()
            }
            StateError::Unsupported { found } => format!(
                "partie stockée en state_version {found}, aucune migration vers {STATE_VERSION} : refusée"
            ),
            StateError::TooNew { found } => format!(
                "partie stockée en state_version {found}, plus récente que ce serveur ({STATE_VERSION}) : refusée"
            ),
            StateError::Corrupt => {
                format!("partie stockée en state_version {STATE_VERSION} mais illisible : refusée")
            }
        }
    }
}

impl StoredGame {
    pub fn new(game: Game, token_hashes: Vec<String>) -> Self {
        Self {
            state_version: STATE_VERSION,
            game,
            token_hashes,
        }
    }

    /// Convertit en JSON pour la base.
    pub fn encode(&self) -> Result<Value, StateError> {
        serde_json::to_value(self).map_err(|_| StateError::Corrupt)
    }

    /// Relit un état stocké, en appliquant les migrations nécessaires.
    pub fn decode(value: Value) -> Result<StoredGame, StateError> {
        let version = value
            .get("state_version")
            .and_then(Value::as_u64)
            .ok_or(StateError::MissingVersion)?;
        // Chaque migration fait passer le JSON d'une version à la suivante ; un exemple figé de chaque
        // ancienne version est relu par `tests/state_version.rs`.
        let value = match version {
            1 => migrate_v3_to_v4(migrate_v2_to_v3(migrate_v1_to_v2(value)?)?)?,
            2 => migrate_v3_to_v4(migrate_v2_to_v3(value)?)?,
            3 => migrate_v3_to_v4(value)?,
            STATE_VERSION => value,
            found if found > STATE_VERSION => return Err(StateError::TooNew { found }),
            found => return Err(StateError::Unsupported { found }),
        };
        serde_json::from_value(value).map_err(|_| StateError::Corrupt)
    }
}

/// v1 → v2 : chaque joueur reçoit `missed_turns: 0` et `eliminated: false` (personne n'était absent).
fn migrate_v1_to_v2(mut value: Value) -> Result<Value, StateError> {
    let players = value
        .get_mut("game")
        .and_then(|game| game.get_mut("players"))
        .and_then(Value::as_array_mut)
        .ok_or(StateError::Corrupt)?;
    for player in players {
        let object = player.as_object_mut().ok_or(StateError::Corrupt)?;
        object.insert("missed_turns".to_string(), Value::from(0));
        object.insert("eliminated".to_string(), Value::from(false));
    }
    value["state_version"] = Value::from(2);
    Ok(value)
}

/// v2 → v3 : chaque joueur reçoit `abandoned: false` (le départ volontaire n'existait pas).
fn migrate_v2_to_v3(mut value: Value) -> Result<Value, StateError> {
    let players = value
        .get_mut("game")
        .and_then(|game| game.get_mut("players"))
        .and_then(Value::as_array_mut)
        .ok_or(StateError::Corrupt)?;
    for player in players {
        let object = player.as_object_mut().ok_or(StateError::Corrupt)?;
        object.insert("abandoned".to_string(), Value::from(false));
    }
    value["state_version"] = Value::from(3);
    Ok(value)
}

/// v3 → v4 : les parties existantes gardent le paquet complet (`deck: "Full"`, règle officielle).
fn migrate_v3_to_v4(mut value: Value) -> Result<Value, StateError> {
    let rules = value
        .get_mut("game")
        .and_then(|game| game.get_mut("rules"))
        .and_then(Value::as_object_mut)
        .ok_or(StateError::Corrupt)?;
    rules.insert("deck".to_string(), Value::from("Full"));
    value["state_version"] = Value::from(4);
    Ok(value)
}
