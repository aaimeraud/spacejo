# Architecture de SpaceJo

Page d'entrée pour découvrir le projet. Elle décrit le code tel qu'il est ; les choix et leurs raisons
sont dans [decisions/](decisions/).

## Vue d'ensemble

SpaceJo est un jeu de cartes au tour par tour (2 à 4 joueurs) qui reprend les règles de Skyjo
([RULES.md](RULES.md)).

```
crates/spacejo-core   logique du jeu (Rust pur, sans réseau, sans base, sans horloge)
```

> Les sections sur l'API, la base de données et l'interface seront ajoutées avec le code correspondant.

## Le core : `crates/spacejo-core`

Une *caisse* (crate) est un paquet Rust ([chapitre 7](https://doc.rust-lang.org/stable/book/ch07-01-packages-and-crates.html)).
Celle-ci ne contient que les règles du jeu : elle ne fait aucun accès réseau, aucune requête SQL, et
ne lit pas l'heure. Elle est donc entièrement testable avec `cargo test`, et déterministe : même graine
et mêmes actions donnent la même partie.

| Fichier | Rôle |
|---|---|
| `cards.rs` | composition du paquet (150 cartes) |
| `rng.rs` | générateur SplitMix64 initialisé par une graine, mélange de Fisher-Yates |
| `rules.rs` | `Rules` (réglages de partie), ses enums, valeurs par défaut officielles, bornes de `end_score` |
| `scoring.rs` | `finisher_round_score` : pénalité du finisseur selon `Rules` |
| `grid.rs` | grille 3 × 4, colonnes retirées, retrait des colonnes identiques |
| `game.rs` | `Game` (état complet), `Phase`, `Action`, `apply`, salon (`new`, `join`, `update_rules`, `start`) |
| `view.rs` | `view_for` : vue filtrée d'un joueur (aucune valeur de carte cachée) |
| `error.rs` | `GameError` : code stable + message en français |

### Phases d'une partie

```
Lobby ──start (hôte, ≥ 2 joueurs)──▶ InitialReveal ──tous ont retourné 2 cartes──▶ Playing
                                          ▲                                           │
                                          │ NextRound (n'importe quel joueur)         │ fin de manche
                                          │                                           ▼
                                      RoundOver ◀───── pas de fin de partie ───── comptage
                                                                                      │
                                                         un total ≥ end_score ────────┴──▶ GameOver
                                                         (et total le plus bas unique,
                                                          ou end_tie = SharedVictory)
```

- `Lobby` : les joueurs arrivent (`join`), l'hôte (joueur 0) peut changer les réglages.
- `InitialReveal` : chaque joueur retourne 2 cartes, dans n'importe quel ordre. Ensuite, le premier joueur
  est celui désigné à la fin de la manche précédente (`next_round_starter`), ou, à la première manche, celui
  dont les cartes visibles ont la plus grande somme (égalité : `first_player_tie`).
- `Playing(Turn)` : `Turn` contient le joueur courant, la carte en main éventuelle (`Held`, avec sa
  provenance : pioche ou défausse) et le finisseur éventuel.
- `RoundOver` : la manche est comptée (`Game::rounds`), on attend l'action `NextRound`.
- `GameOver` : `Game::winners` contient le ou les gagnants.

### Un tour de jeu

1. `DrawFromPile` ou `TakeDiscard` : la carte passe « en main » (`Turn::held`).
2. `SwapHeld` (toujours possible) ou `DiscardHeldAndReveal` (seulement pour une carte de la pioche, et sur
   une carte cachée). La carte remplacée ou défaussée va, face visible, sur la défausse.
3. Fin du coup (`Game::end_move`) : retrait des colonnes identiques, détection du finisseur, pioche épuisée
   (`empty_pile`), puis joueur suivant ou fin de manche quand le tour revient au finisseur.

Chaque fonction vérifie tout AVANT de modifier l'état : une action refusée renvoie une `GameError` et ne
change rien. Les dessus de pioche et de défausse sont les derniers éléments de leurs `Vec`.

### Vue filtrée (information cachée)

L'état complet (`Game`) contient la valeur de toutes les cartes et l'état du générateur : il ne sort jamais
du serveur. `view_for(&game, joueur)` construit un `GameView` où chaque carte face cachée devient
`PublicSlot::Hidden`, un type qui n'a pas de champ valeur : une fuite est impossible par construction. La
pioche n'est donnée que par son nombre de cartes ; la carte en main n'est montrée qu'à celui qui l'a piochée
(une carte prise sur la défausse était déjà publique). Le test `crates/spacejo-core/tests/view.rs` vérifie que
deux parties qui ne diffèrent que par UNE carte cachée donnent exactement la même vue.

## Limites connues et évolutions prévues

- **Polling** : l'interface interroge le serveur toutes les 2 secondes. Évolution prévue : SSE ou service
  temps réel.
- **Expiration des parties** : les anciennes parties ne sont jamais supprimées.
- **Limitation de débit** : aucune limite sur la création et la jonction de parties.
- **Comptes utilisateurs** : aucun ; l'identité est un jeton secret par partie.
- **Nombre de joueurs** : plafonné à 4 en v1 (`MAX_PLAYERS`) ; le core n'en dépend pas ailleurs.
