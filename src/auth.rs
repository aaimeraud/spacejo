//! Identité des joueurs et codes de partie.
//!
//! - Le jeton est un secret aléatoire de 256 bits, remis une seule fois au joueur. Le serveur ne
//!   stocke que son hachage SHA-256 : une fuite de la base ne donne pas les jetons.
//!   Un hachage rapide suffit, car le jeton est long et aléatoire (on ne peut pas le deviner par
//!   essais successifs, contrairement à un mot de passe).
//! - Le code de partie est tiré avec le générateur cryptographique du système, pour ne pas être
//!   devinable.

use sha2::{Digest, Sha256};

use crate::error::ApiError;

/// Nombre d'octets aléatoires d'un jeton (32 octets = 256 bits, au-delà des 128 bits exigés).
pub const TOKEN_BYTES: usize = 32;
/// Longueur d'un code de partie.
pub const GAME_CODE_LENGTH: usize = 6;
/// Alphabet des codes : sans 0/O ni 1/I, qu'on confond à l'écrit. 32 symboles exactement, ce qui
/// permet un tirage sans biais (voir `generate_code`).
pub const GAME_CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Remplit `buffer` avec l'entropie du système. Une erreur ici est exceptionnelle
/// (système sans source d'aléa) : on la transforme en erreur interne, sans paniquer.
fn fill_random(buffer: &mut [u8]) -> Result<(), ApiError> {
    getrandom::fill(buffer).map_err(|_| ApiError::internal())
}

/// Écrit des octets en hexadécimal (2 caractères par octet).
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Nouveau jeton secret, en hexadécimal (64 caractères).
pub fn generate_token() -> Result<String, ApiError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    fill_random(&mut bytes)?;
    Ok(to_hex(&bytes))
}

/// Hachage SHA-256 d'un jeton, en hexadécimal. C'est la seule forme stockée.
pub fn hash_token(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

/// Comparaison en temps constant : le temps de réponse ne dépend pas du nombre de caractères
/// identiques, ce qui empêche de deviner un hachage caractère par caractère.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // `fold` parcourt tous les octets et accumule les différences, sans s'arrêter au premier écart.
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Retrouve le joueur dont le hachage de jeton correspond à `token`.
pub fn find_player(token_hashes: &[String], token: &str) -> Option<usize> {
    let hash = hash_token(token);
    // On compare avec TOUS les hachages (pas de sortie anticipée) pour un temps constant.
    let mut found = None;
    for (player, stored) in token_hashes.iter().enumerate() {
        if constant_time_eq(stored, &hash) {
            found = Some(player);
        }
    }
    found
}

/// Nouveau code de partie, par exemple `K7MPQX`.
pub fn generate_code() -> Result<String, ApiError> {
    let mut bytes = [0u8; GAME_CODE_LENGTH];
    fill_random(&mut bytes)?;
    // L'alphabet a 32 symboles : garder les 5 bits de poids faible (`& 31`) donne un symbole
    // uniforme, sans le biais qu'aurait un modulo avec une autre taille d'alphabet.
    Ok(bytes
        .iter()
        .map(|b| GAME_CODE_ALPHABET[(b & 31) as usize] as char)
        .collect())
}

/// Nouvelle graine pour le mélange, tirée de l'entropie du système. Elle n'est jamais exposée.
pub fn generate_seed() -> Result<u64, ApiError> {
    let mut bytes = [0u8; 8];
    fill_random(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Valide un code saisi par un joueur : espaces retirés, majuscules, longueur et alphabet.
pub fn normalize_code(input: &str) -> Result<String, ApiError> {
    let code = input.trim().to_ascii_uppercase();
    let valid =
        code.len() == GAME_CODE_LENGTH && code.bytes().all(|b| GAME_CODE_ALPHABET.contains(&b));
    if valid {
        Ok(code)
    } else {
        Err(ApiError::invalid_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_random_and_hashed() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        let hash = hash_token(&a);
        assert_eq!(hash.len(), 64);
        assert_ne!(hash, a);
        assert_eq!(hash, hash_token(&a), "hachage déterministe");
        assert_eq!(find_player(&[hash_token(&b), hash], &a), Some(1));
        assert_eq!(find_player(&[hash_token(&b)], &a), None);
    }

    #[test]
    fn codes_use_the_unambiguous_alphabet() {
        for _ in 0..200 {
            let code = generate_code().unwrap();
            assert_eq!(code.len(), GAME_CODE_LENGTH);
            assert!(!code.contains(['0', 'O', '1', 'I']));
            assert_eq!(normalize_code(&code), Ok(code.clone()));
        }
    }

    #[test]
    fn code_input_is_normalized_and_validated() {
        assert_eq!(normalize_code("  k7mpqx "), Ok("K7MPQX".to_string()));
        assert!(normalize_code("K7MPQ").is_err());
        assert!(normalize_code("K7MPQXY").is_err());
        assert!(
            normalize_code("K7MPQ0").is_err(),
            "0 n'est pas dans l'alphabet"
        );
        assert!(
            normalize_code("K7MPQI").is_err(),
            "I n'est pas dans l'alphabet"
        );
        assert!(normalize_code("K7MPQé").is_err());
    }

    #[test]
    fn constant_time_eq_compares_whole_strings() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "abcd"));
    }
}
