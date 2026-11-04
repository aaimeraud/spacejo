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
/// Pioche réduite (`DeckSize::Reduced`) : cartes gardées par joueur en jeu, 12 pour sa grille, le reste pour la pioche.
pub const REDUCED_DECK_CARDS_PER_PLAYER: usize = 40;

/// Tous les réglages d'une partie. Ils sont enregistrés en entier dans l'état de la partie.
///
/// `deny_unknown_fields` : un champ inconnu dans le JSON est refusé (validation stricte des
/// données du navigateur). Un champ manquant est refusé aussi, sauf `deck`, ajouté après la v1.3 :
/// absent, c'est le paquet complet (règle officielle), pour qu'une interface encore ancienne, qui
/// envoie les 7 réglages d'origine, continue de fonctionner pendant une mise à jour.
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
    #[serde(default)]
    pub deck: DeckSize,
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

/// Combien de cartes compte le paquet de chaque manche ?
/// `Default` (avec `#[default]`) donne la valeur officielle, utilisée quand `deck` est absent du JSON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckSize {
    /// Officiel : les 150 cartes.
    #[default]
    Full,
    /// Variante : `REDUCED_DECK_CARDS_PER_PLAYER` cartes par joueur en jeu (au plus 150), tirées au hasard dans le
    /// paquet mélangé. La pioche est plus courte : la défausse est remélangée plus souvent.
    Reduced,
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
            deck: DeckSize::Full,
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
