//! Erreurs de l'API : un statut HTTP, un code stable et un message en français.
//! Le format JSON est décrit dans docs/CONTRACT.md (section « Erreurs »).

use serde_json::{Value, json};
use spacejo_core::GameError;

/// Une erreur prête à être renvoyée au navigateur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    /// Corps JSON : `{ "error": { "code": ..., "message": ... } }`.
    pub fn body(&self) -> Value {
        json!({ "error": { "code": self.code, "message": self.message } })
    }

    pub fn invalid_request(detail: impl std::fmt::Display) -> Self {
        Self::new(
            400,
            "invalid_request",
            format!("Requête invalide : {detail}"),
        )
    }

    pub fn invalid_code() -> Self {
        Self::new(400, "invalid_code", "Code de partie invalide.")
    }

    pub fn missing_token() -> Self {
        Self::new(
            401,
            "missing_token",
            "Jeton de joueur manquant (en-tête X-Player-Token).",
        )
    }

    pub fn invalid_token() -> Self {
        Self::new(
            401,
            "invalid_token",
            "Jeton de joueur invalide pour cette partie.",
        )
    }

    pub fn game_not_found() -> Self {
        Self::new(
            404,
            "game_not_found",
            "Aucune partie ne correspond à ce code.",
        )
    }

    pub fn not_found() -> Self {
        Self::new(404, "not_found", "Route inconnue.")
    }

    pub fn method_not_allowed() -> Self {
        Self::new(405, "method_not_allowed", "Méthode HTTP non autorisée.")
    }

    pub fn version_conflict() -> Self {
        Self::new(
            409,
            "version_conflict",
            "La partie a changé entre-temps : l'écran va se mettre à jour, réessayez.",
        )
    }

    /// L'hôte veut passer le tour d'un joueur qui est présent, ou qui ne bloque pas la partie.
    pub fn player_not_absent() -> Self {
        Self::new(
            422,
            "player_not_absent",
            "Ce joueur n'est pas absent, ou ce n'est pas lui que l'on attend.",
        )
    }

    pub fn payload_too_large() -> Self {
        Self::new(413, "payload_too_large", "Requête trop volumineuse.")
    }

    /// Message de chat vide, trop long, ou avec des caractères de contrôle.
    pub fn invalid_message() -> Self {
        Self::new(
            400,
            "invalid_message",
            format!(
                "Le message doit contenir de 1 à {} caractères, sur une seule ligne.",
                crate::handlers::MAX_CHAT_CHARS
            ),
        )
    }

    /// Trop de requêtes depuis la même adresse (`src/rate_limit.rs`) ; `retry_after` : secondes avant de réessayer.
    pub fn rate_limited(retry_after: u64) -> Self {
        Self::new(
            429,
            "rate_limited",
            format!("Trop de requêtes. Réessayez dans {retry_after} s."),
        )
    }

    /// Erreur interne : le détail n'est jamais envoyé au navigateur.
    pub fn internal() -> Self {
        Self::new(500, "internal", "Erreur interne du serveur.")
    }
}

/// Conversion d'une erreur du jeu en erreur HTTP. `From` permet d'utiliser `?` directement
/// sur un `Result<_, GameError>` dans une fonction qui renvoie `Result<_, ApiError>`.
/// https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator
impl From<GameError> for ApiError {
    fn from(error: GameError) -> Self {
        let status = match error {
            GameError::InvalidName | GameError::InvalidRules => 400,
            GameError::NotHost => 403,
            // Ne devrait jamais arriver : l'API n'utilise que des joueurs identifiés par leur jeton.
            GameError::UnknownPlayer | GameError::NotEnoughCards => return ApiError::internal(),
            _ => 422,
        };
        ApiError::new(status, error.code(), error.message())
    }
}
