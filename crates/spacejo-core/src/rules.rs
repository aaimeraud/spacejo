//! Réglages de partie (« règles maison »), choisis par l'hôte avant le lancement.
//!
//! Chaque règle que la notice officielle ne précise pas est un réglage ici, jamais une valeur
//! en dur ailleurs. Les valeurs par défaut sont les règles officielles (docs/RULES.md).

use serde::{Deserialize, Serialize};

use crate::error::GameError;

/// Borne basse de `end_score` (incluse).
pub const END_SCORE_MIN: i32 = 10;
/// Borne haute de `end_score` (incluse).
pub const END_SCORE_MAX: i32 = 500;
/// Score de fin de partie officiel.
pub const DEFAULT_END_SCORE: i32 = 100;

/// Tous les réglages d'une partie. Ils sont enregistrés en entier dans l'état de la partie.
///
/// `deny_unknown_fields` : un champ inconnu dans le JSON est refusé (validation stricte des
/// données du navigateur). Comme il n'y a pas de `#[serde(default)]`, un champ manquant est
/// refusé aussi : les réglages reçus sont toujours complets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub penalty: PenaltyRule,
    pub penalty_scope: PenaltyScope,
    pub first_player_tie: FirstPlayerTie,
    pub next_round_starter: NextRoundStarter,
    pub empty_pile: EmptyPile,
    pub end_tie: EndTie,
    pub end_score: i32,
}

// Les enums ci-dessous n'ont que des variantes sans données : serde les écrit en JSON sous
// forme de chaîne ("DoubleUnlessStrictlyLowest"…), et refuse toute chaîne inconnue.
// Enums : https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html

/// Quand le finisseur est-il doublé ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenaltyRule {
    /// Officiel : doublé s'il n'a pas le score strictement le plus bas (une égalité le double).
    DoubleUnlessStrictlyLowest,
    /// Variante : doublé seulement si un autre joueur a strictement moins (une égalité l'épargne).
    DoubleOnlyIfBeaten,
    /// Variante : aucune pénalité.
    Disabled,
}

/// Sur quoi porte le doublement ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenaltyScope {
    /// Le total de la manche est doublé (transcription du livret).
    RoundTotal,
    /// Seules les cartes positives comptent deux fois.
    PositiveCardsOnly,
}

/// Égalité sur la somme des 2 premières cartes retournées.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirstPlayerTie {
    /// Le premier arrivé dans la partie commence (l'hôte d'abord).
    JoinOrder,
    /// Tirage au sort par le serveur, avec la graine de la partie.
    Random,
}

/// Qui commence la manche suivante ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextRoundStarter {
    /// Officiel : celui qui a terminé la manche précédente.
    Finisher,
    /// Celui qui a le score de manche le plus bas (après pénalité ; égalité : ordre d'arrivée).
    RoundWinner,
    /// Le joueur suivant celui qui a commencé la manche précédente.
    Rotate,
}

/// Que faire quand la pioche est vide ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmptyPile {
    /// On remélange la défausse, sauf sa carte du dessus, pour former la pioche.
    ReshuffleDiscard,
    /// La manche se termine immédiatement, sans dernier tour.
    EndRound,
}

/// Égalité sur le total le plus bas en fin de partie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndTie {
    /// Tous les ex æquo gagnent.
    SharedVictory,
    /// On joue des manches supplémentaires jusqu'à un total le plus bas unique.
    ExtraRound,
}

/// Valeurs par défaut = règles officielles : la première valeur de chaque enum.
impl Default for Rules {
    fn default() -> Self {
        Self {
            penalty: PenaltyRule::DoubleUnlessStrictlyLowest,
            penalty_scope: PenaltyScope::RoundTotal,
            first_player_tie: FirstPlayerTie::JoinOrder,
            next_round_starter: NextRoundStarter::Finisher,
            empty_pile: EmptyPile::ReshuffleDiscard,
            end_tie: EndTie::SharedVictory,
            end_score: DEFAULT_END_SCORE,
        }
    }
}

impl Rules {
    /// Vérifie ce que le typage ne garantit pas : les bornes de `end_score`.
    /// Les valeurs d'enum inconnues, elles, sont déjà refusées à la lecture du JSON.
    pub fn validate(&self) -> Result<(), GameError> {
        if (END_SCORE_MIN..=END_SCORE_MAX).contains(&self.end_score) {
            Ok(())
        } else {
            Err(GameError::InvalidRules)
        }
    }
}
