//! Limitation de débit par adresse IP sur les routes ouvertes sans jeton ou presque (création, jonction et liste
//! des parties publiques), et sur l'envoi de messages du chat. Sans elle, un script pourrait créer des milliers de
//! parties, remplir des salons publics ou inonder un chat.
//!
//! Méthode la plus simple : une **fenêtre fixe** d'une minute. Chaque couple (adresse, route) a un compteur remis
//! à zéro au début de chaque fenêtre ; au-delà de la limite, la requête est refusée (`429`) avec le nombre de
//! secondes avant la fenêtre suivante. Les compteurs vivent en mémoire : ils sont perdus au redémarrage, et ne
//! seraient pas partagés entre plusieurs serveurs (voir « Limites connues » dans docs/ARCHITECTURE.md). Ce n'est
//! pas un état de partie : la règle « rien en mémoire entre deux requêtes » porte sur les parties.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::handlers::Endpoint;

/// Durée d'une fenêtre de comptage.
pub const RATE_WINDOW: Duration = Duration::from_secs(60);
/// Créations de partie par adresse et par minute.
pub const CREATE_PER_MINUTE: u32 = 10;
/// Jonctions par adresse et par minute (plusieurs joueurs peuvent partager une adresse, à la maison ou au lycée).
pub const JOIN_PER_MINUTE: u32 = 30;
/// Lectures de la liste des parties publiques par adresse et par minute : l'accueil la relit toutes les 5 s
/// (12 par minute), on laisse de la place pour plusieurs personnes derrière la même adresse.
pub const PUBLIC_GAMES_PER_MINUTE: u32 = 60;
/// Messages du chat par adresse et par minute : assez pour discuter, pas pour inonder les autres joueurs.
pub const CHAT_PER_MINUTE: u32 = 30;
/// Au-delà de ce nombre de compteurs, les compteurs des fenêtres terminées sont effacés (mémoire bornée).
const MAX_TRACKED: usize = 10_000;

/// Limite d'une route, ou `None` si la route n'est pas limitée (les autres routes exigent un jeton de joueur).
pub fn limit_for(endpoint: Endpoint) -> Option<u32> {
    match endpoint {
        Endpoint::Create => Some(CREATE_PER_MINUTE),
        Endpoint::Join => Some(JOIN_PER_MINUTE),
        Endpoint::PublicGames => Some(PUBLIC_GAMES_PER_MINUTE),
        Endpoint::Chat => Some(CHAT_PER_MINUTE),
        _ => None,
    }
}

/// Compteur d'une fenêtre.
#[derive(Clone, Copy, Debug)]
struct Window {
    started: Instant,
    count: u32,
}

/// Les compteurs, partagés par toutes les requêtes (`Arc`) et protégés par un verrou (`Mutex`).
/// https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html
#[derive(Clone, Debug, Default)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<(IpAddr, Endpoint), Window>>>,
}

impl RateLimiter {
    /// Compte une requête maintenant. `Err(secondes)` : refusée, nouvel essai possible dans ce délai.
    pub fn check(&self, ip: IpAddr, endpoint: Endpoint, limit: u32) -> Result<(), u64> {
        self.check_at(ip, endpoint, limit, Instant::now())
    }

    /// Comme `check`, avec l'instant donné (les tests font passer le temps sans attendre).
    pub fn check_at(
        &self,
        ip: IpAddr,
        endpoint: Endpoint,
        limit: u32,
        now: Instant,
    ) -> Result<(), u64> {
        // Verrou inutilisable (un autre fil a paniqué en le tenant) : on laisse passer plutôt que de bloquer le jeu.
        let Ok(mut windows) = self.windows.lock() else {
            return Ok(());
        };
        if windows.len() >= MAX_TRACKED {
            windows.retain(|_, window| now.duration_since(window.started) < RATE_WINDOW);
        }
        let window = windows.entry((ip, endpoint)).or_insert(Window {
            started: now,
            count: 0,
        });
        // Fenêtre terminée : on repart de zéro.
        if now.duration_since(window.started) >= RATE_WINDOW {
            *window = Window {
                started: now,
                count: 0,
            };
        }
        if window.count >= limit {
            let left = RATE_WINDOW.saturating_sub(now.duration_since(window.started));
            // Arrondi au-dessus, et au moins 1 s : « réessayez dans 0 s » n'aurait pas de sens.
            return Err(left.as_secs_f64().ceil().max(1.0) as u64);
        }
        window.count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    const ALICE: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1));
    const BOB: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2));

    #[test]
    fn requests_beyond_the_limit_are_refused_until_the_next_window() {
        let limiter = RateLimiter::default();
        let start = Instant::now();
        for _ in 0..3 {
            assert_eq!(limiter.check_at(ALICE, Endpoint::Create, 3, start), Ok(()));
        }
        let later = start + Duration::from_secs(20);
        assert_eq!(limiter.check_at(ALICE, Endpoint::Create, 3, later), Err(40));
        // Fenêtre suivante : de nouveau permis.
        let next = start + RATE_WINDOW;
        assert_eq!(limiter.check_at(ALICE, Endpoint::Create, 3, next), Ok(()));
    }

    #[test]
    fn each_address_and_route_has_its_own_counter() {
        let limiter = RateLimiter::default();
        let now = Instant::now();
        assert_eq!(limiter.check_at(ALICE, Endpoint::Create, 1, now), Ok(()));
        assert!(limiter.check_at(ALICE, Endpoint::Create, 1, now).is_err());
        assert_eq!(limiter.check_at(BOB, Endpoint::Create, 1, now), Ok(()));
        assert_eq!(limiter.check_at(ALICE, Endpoint::Join, 1, now), Ok(()));
    }

    #[test]
    fn only_open_routes_and_chat_messages_are_limited() {
        assert_eq!(limit_for(Endpoint::Create), Some(CREATE_PER_MINUTE));
        assert_eq!(limit_for(Endpoint::Join), Some(JOIN_PER_MINUTE));
        assert_eq!(
            limit_for(Endpoint::PublicGames),
            Some(PUBLIC_GAMES_PER_MINUTE)
        );
        assert_eq!(limit_for(Endpoint::Chat), Some(CHAT_PER_MINUTE));
        assert_eq!(limit_for(Endpoint::Action), None);
        assert_eq!(limit_for(Endpoint::State), None);
    }
}
