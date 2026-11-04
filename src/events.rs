//! Avis de changement de partie, poussés aux navigateurs (server-sent events, voir `server.rs`).
//!
//! Quand une partie change (un coup, une jonction, un tour passé…), le serveur publie sa nouvelle version ;
//! chaque navigateur abonné à cette partie relit aussitôt l'état, au lieu d'attendre son prochain polling.
//! Seul le numéro de version circule : aucune carte, aucun nom, rien de caché.
//!
//! Ces avis vivent dans la mémoire du serveur. C'est une simple accélération : l'état, lui, reste en base,
//! et le polling continue (plus lentement) au cas où un avis se perdrait. Avec plusieurs serveurs, un avis
//! ne toucherait que les navigateurs connectés au même serveur (les autres verraient le coup au polling).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::broadcast;

/// Nombre d'avis gardés pour un abonné lent ; au-delà, il saute les plus anciens (seul le dernier compte).
const CHANNEL_CAPACITY: usize = 16;

/// Les canaux d'avis, un par partie (clé : code de partie).
///
/// `Arc<Mutex<…>>` : une même table partagée par toutes les requêtes, protégée contre les accès simultanés.
/// https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html
#[derive(Clone, Debug, Default)]
pub struct Events {
    channels: Arc<Mutex<HashMap<String, Channel>>>,
}

#[derive(Debug)]
struct Channel {
    /// Émetteur `broadcast` : chaque abonné reçoit sa copie de chaque avis.
    sender: broadcast::Sender<i32>,
    /// Dernière version publiée : on ne republie pas la même (chaque polling renvoie la version courante).
    last: i32,
}

impl Channel {
    fn new() -> Self {
        Self {
            sender: broadcast::channel(CHANNEL_CAPACITY).0,
            last: 0,
        }
    }
}

impl Events {
    /// Table verrouillée. Un verrou « empoisonné » (une requête a paniqué en le tenant) est repris tel quel :
    /// la table ne contient que des canaux, toujours utilisables.
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Channel>> {
        self.channels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Annonce la version `version` de la partie `code`, si elle est plus récente que la dernière annoncée.
    pub fn publish(&self, code: &str, version: i32) {
        let mut channels = self.lock();
        let channel = channels
            .entry(code.to_string())
            .or_insert_with(Channel::new);
        if version > channel.last {
            channel.last = version;
            // Personne à l'écoute : l'envoi échoue, sans importance.
            let _ = channel.sender.send(version);
        }
    }

    /// S'abonne aux avis de la partie `code`. Au passage, oublie les parties que plus personne n'écoute.
    pub fn subscribe(&self, code: &str) -> broadcast::Receiver<i32> {
        let mut channels = self.lock();
        channels.retain(|other, channel| other == code || channel.sender.receiver_count() > 0);
        channels
            .entry(code.to_string())
            .or_insert_with(Channel::new)
            .sender
            .subscribe()
    }

    /// Ferme tous les flux (arrêt du serveur) : les connexions ouvertes se terminent, sans quoi l'arrêt
    /// attendrait indéfiniment la fin de ces réponses sans fin.
    pub fn close_all(&self) {
        self.lock().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscribers_get_each_newer_version_once() {
        let events = Events::default();
        let mut receiver = events.subscribe("K7MPQX");
        events.publish("K7MPQX", 2);
        events.publish("K7MPQX", 2); // même version (un polling) : rien de nouveau
        events.publish("K7MPQX", 1); // plus ancienne : ignorée
        events.publish("K7MPQX", 3);
        events.publish("ABCDEF", 9); // autre partie
        assert_eq!(receiver.recv().await.ok(), Some(2));
        assert_eq!(receiver.recv().await.ok(), Some(3));
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn closing_ends_the_streams() {
        let events = Events::default();
        let mut receiver = events.subscribe("K7MPQX");
        events.close_all();
        assert!(matches!(
            receiver.recv().await,
            Err(broadcast::error::RecvError::Closed)
        ));
    }

    #[test]
    fn games_nobody_listens_to_are_forgotten() {
        let events = Events::default();
        drop(events.subscribe("K7MPQX"));
        events.publish("ABCDEF", 1);
        let _kept = events.subscribe("ZZZZZZ");
        let channels = events.lock();
        assert!(!channels.contains_key("K7MPQX"));
        assert!(!channels.contains_key("ABCDEF"));
        assert!(channels.contains_key("ZZZZZZ"));
    }
}
