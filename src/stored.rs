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
pub const STATE_VERSION: u64 = 1;

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
        // Pour une future version 2 : ajouter ici `1 => migrate_v1_to_v2(value)` puis relire,
        // et garder un exemple figé de la version 1 dans `tests/fixtures/`.
        match version {
            STATE_VERSION => serde_json::from_value(value).map_err(|_| StateError::Corrupt),
            found if found > STATE_VERSION => Err(StateError::TooNew { found }),
            found => Err(StateError::Unsupported { found }),
        }
    }
}
