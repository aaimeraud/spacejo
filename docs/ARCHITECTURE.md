# Architecture de SpaceJo

Page d'entrée pour découvrir le projet. Elle décrit le code tel qu'il est ; les choix et leurs raisons
sont dans [decisions/](decisions/).

## Vue d'ensemble

SpaceJo est un jeu de cartes au tour par tour (2 à 4 joueurs) qui reprend les règles de Skyjo
([RULES.md](RULES.md)). Trois composants :

```
 Navigateur (web/, React)                Vercel                              PostgreSQL
┌──────────────────────┐   HTTPS/JSON   ┌───────────────────────────────┐   SQL   ┌──────────────┐
│ écrans, polling 2 s   │ ─────────────▶ │ api/*.rs : une fonction Rust  │ ──────▶ │ table games  │
│ envoie des Action     │ ◀───────────── │ par route, sans état          │ ◀────── │ (state JSONB,│
│ affiche la vue reçue  │   vue filtrée  │ src/ : HTTP, jetons, stockage │         │  version)    │
└──────────────────────┘                │ crates/spacejo-core : le jeu  │         └──────────────┘
                                        └───────────────────────────────┘
```

| Dossier | Rôle |
|---|---|
| `crates/spacejo-core/` | logique du jeu (Rust pur, sans réseau, sans base, sans horloge) |
| `src/` | bibliothèque `spacejo-api` : routes, jetons, format enregistré, stockage, adaptateur Vercel |
| `api/` | une fonction Vercel par fichier (quelques lignes : elles appellent `spacejo_api::vercel::serve`) |
| `src/bin/` | `dev_server` (toutes les routes dans un processus, pour le développement) et `migrate` |
| `migrations/` | migrations SQL versionnées, appliquées par `sqlx::migrate!` |
| `tests/` | tests de l'API (mémoire et PostgreSQL), exemple figé du format enregistré, tests des scripts |
| `web/` | interface React + TypeScript + Vite |
| `scripts/` | contrôle des commits, release, aperçu du changelog |

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

## L'API : `src/` et `api/`

Les routes et leurs formats JSON sont décrits dans [CONTRACT.md](CONTRACT.md).

| Fichier | Rôle |
|---|---|
| `src/handlers.rs` | les six opérations, indépendantes de Vercel (`handle(store, endpoint, request)`) |
| `src/auth.rs` | jetons (256 bits, hachage SHA-256, comparaison en temps constant), codes de partie, graines |
| `src/stored.rs` | `StoredGame` (ce qui est dans `games.state`) et `STATE_VERSION` |
| `src/store.rs` | `Store` : PostgreSQL (`sqlx`) ou mémoire, avec le verrou optimiste |
| `src/vercel.rs` | conversion entre les requêtes de `vercel_runtime` (hyper) et `handlers` |
| `src/error.rs` | `ApiError` (statut, code, message) et conversion depuis `GameError` |

### Parcours d'une action, du clic à la réponse

1. Le joueur clique sur une carte. L'interface envoie `POST /api/action` avec
   `{ code, expected_version, action }` et l'en-tête `X-Player-Token`. Elle envoie une intention
   (`Action`), jamais un état.
2. Vercel lance la fonction `api/action.rs`, qui appelle `vercel::serve(Endpoint::Action)`. Le corps est
   lu avec une taille maximale (16 Kio), puis `handlers::handle` prend le relais.
3. Le corps est validé (`serde` refuse champ inconnu, valeur inconnue, type incorrect → 400).
4. La partie est chargée par son code : `SELECT id, state, version FROM games WHERE code = $1`. Le JSON
   est relu par `StoredGame::decode`, qui vérifie `state_version`.
5. Le jeton est haché et comparé aux hachages stockés : on obtient l'identifiant du joueur (sinon 401).
6. Si `expected_version` diffère de `version` : 409, rien n'est écrit (écran périmé ou requête rejouée).
7. `spacejo_core::apply(&mut game, joueur, action)` applique la règle ; une erreur de jeu donne 422.
8. Sauvegarde avec verrou optimiste (ci-dessous). Si quelqu'un a écrit entre-temps : 409.
9. Réponse : `{ version, view }`, où `view = view_for(&game, joueur)`. L'interface affiche cette vue ; les
   autres joueurs la verront à leur prochain polling (`GET /api/state`, toutes les 2 secondes).

Aucune donnée de partie n'est gardée en mémoire entre deux requêtes : chaque requête recharge tout. Seul
le pool de connexions PostgreSQL survit d'une requête à l'autre dans une même instance.

### Verrou optimiste

« Optimiste » : on ne bloque personne pendant le calcul ; on vérifie seulement au moment d'écrire que
personne n'a écrit avant nous.

```sql
UPDATE games SET state = $1, version = version + 1, updated_at = now()
WHERE id = $2 AND version = $3      -- $3 = la version lue à l'étape 4
RETURNING version
```

Si deux requêtes ont lu la même version, PostgreSQL n'exécute la mise à jour que pour la première ; la
seconde ne trouve plus de ligne avec `version = $3`, ne modifie rien et reçoit un conflit (409). Le test
`tests/postgres.rs` lance deux écritures simultanées et vérifie qu'une seule réussit. La création et la
jonction, qui ne connaissent pas encore la version, relisent et réessaient elles-mêmes (5 fois au plus).

### Identité des joueurs

À la création ou à la jonction, le serveur tire un jeton de 32 octets aléatoires (`getrandom`, entropie du
système) et le renvoie UNE fois. Il n'en stocke que le hachage SHA-256, dans `StoredGame::token_hashes`
(l'indice est l'identifiant du joueur). Le navigateur garde le jeton dans `sessionStorage` (propre à chaque onglet, conservé au rechargement) et l'envoie dans
`X-Player-Token`. Un jeton n'est valable que pour sa partie, et ne permet d'agir qu'en son propre nom.

## L'interface : `web/`

React + TypeScript (mode strict) + Vite. L'interface n'a aucune règle du jeu : elle envoie des
intentions et affiche la vue reçue. Elle ne calcule aucun score (le tableau affiche `rounds` et
`total_score` tels que le serveur les donne).

| Fichier | Rôle |
|---|---|
| `src/api/types.ts` | types de l'API, recopiés de [CONTRACT.md](CONTRACT.md) |
| `src/api/client.ts` | appels HTTP, jeton dans l'en-tête `X-Player-Token`, erreurs → `ApiError` |
| `src/App.tsx` | session, polling, envoi des actions avec `expected_version`, choix de l'écran |
| `src/components/` | `Home` (créer avec réglages / rejoindre), `Lobby` (salon), `GameBoard` (table de jeu et fin de partie), `GridView`, `Card` (carte 3D à deux faces), `Scoreboard`, `RulesPanel` |
| `src/game/seats.ts` | place de chaque joueur autour de la table (moi en bas, puis l'ordre du tour, dans le sens des aiguilles d'une montre) |
| `src/rulesInfo.ts` | phrases d'explication et étiquettes « officiel » / « variante » des réglages |
| `src/game/logic.ts` | quelle action envoie un clic, consigne à afficher, classement (simple tri des totaux) |
| `src/session.ts` | session (code, jeton, identifiant) dans `sessionStorage` |
| `src/config.ts` | constantes : `POLL_INTERVAL_MS` (2000), bornes, longueurs |

**Table de jeu (CSS 3D)** : la partie se joue sur une table ovale inclinée en perspective (`rotateX`). Les
adversaires sont assis autour, à la place calculée par `seats.ts` (un angle sur une ellipse, pour n'importe
quel nombre de joueurs), avec un badge (nom, total, cartes cachées, hôte, « joue », « a terminé »). Pioche et
défausse sont au centre, la carte en main flotte au-dessus ; ma grille est devant moi, sur le bord de la table.
La table est inclinée (`rotateX(30deg)`) et ma grille est très légèrement couchée devant moi. Les grilles des
adversaires sont tournées vers leur propriétaire, comme des cartes posées devant lui (`Seat.rotation` : 180° en
face, 90° à gauche, 270° à droite) ; sur petit écran, elles restent droites pour être lisibles.
La grille de l'adversaire dont c'est le tour est agrandie, vers le centre de la table. Chaque carte a deux
faces dos à dos : la retourner, c'est la faire pivoter de 180° (`rotateY`). Une carte posée et la défausse ont
une petite animation d'apparition. Quand une colonne de trois cartes identiques est retirée, ses cartes
s'illuminent, s'envolent et disparaissent avec le message « Colonne identique ! » : `game/columns.ts` repère ces
colonnes en comparant la vue précédente et la nouvelle. Sur petit écran (moins de 760 px), la table est à
plat et les adversaires côte à côte. Avec la préférence système « réduire les animations », il n'y a aucune
animation. Le choix du CSS 3D plutôt que WebGL est expliqué dans
[decisions/2026-11-03-vue-table-css-3d.md](decisions/2026-11-03-vue-table-css-3d.md).

**Polling** : toutes les 2 secondes, `GET /api/state?code=…&known_version=…`. Si la version n'a pas changé,
la réponse est minimale (`changed: false`) et rien n'est redessiné.

**Conflits** : chaque écriture envoie la version connue. En cas de 409 (un autre joueur a joué entre-temps,
fréquent pendant la mise en place où tous retournent leurs cartes en même temps), l'interface affiche le
message, relit l'état complet, et le joueur refait son geste.

**Session** : `sessionStorage` est propre à chaque onglet et survit au rechargement de la page ; on peut
donc jouer à deux dans deux onglets du même navigateur. Fermer l'onglet fait perdre la session.

**Accessibilité** : cartes et commandes sont des `button` (clavier, focus visible) avec un libellé
(« Carte cachée, colonne 2, ligne 1 », « Carte 7, … ») ; la valeur d'une carte est toujours écrite (la
couleur n'est qu'un repère) ; les étiquettes de réglage sont textuelles ; la consigne et les erreurs sont
annoncées (`role="status"`, `role="alert"`).

En développement, Vite (port 5173) renvoie `/api` vers `cargo run --bin dev_server` (port 3000).

## Modèle de données

Une seule table, créée par `migrations/0001_create_games.sql` :

| Colonne | Type | Rôle |
|---|---|---|
| `id` | `BIGSERIAL` | clé primaire |
| `code` | `TEXT UNIQUE` | code de partie à partager (6 caractères sans 0/O ni 1/I) |
| `state` | `JSONB` | `StoredGame` : `state_version`, `game` (état complet du core), `token_hashes` |
| `version` | `INTEGER` | numéro de version pour le verrou optimiste (1 à la création, +1 par écriture) |
| `updated_at` | `TIMESTAMPTZ` | date de la dernière écriture |

Le schéma n'évolue que par de nouveaux fichiers dans `migrations/` (jamais à la main) ; `cargo run --bin migrate`
les applique, et `sqlx` note celles déjà appliquées dans `_sqlx_migrations`.

Le JSON de `state` a son propre numéro de format, `state_version` (actuellement 1). Si le format de `Game`
change, il faut incrémenter `STATE_VERSION` et écrire la migration dans `StoredGame::decode` ; un état
illisible est refusé avec l'erreur `unsupported_state_version`. Le test `tests/state_version.rs` relit un
exemple figé (`tests/fixtures/state_v1.json`) pour détecter tout changement de format oublié.

## Journaux

Le serveur n'écrit que la méthode, le chemin et le statut de chaque requête, et le type des erreurs de
stockage. Jamais de jeton, de hachage, de valeur de carte, de graine ni d'état de partie.

## Limites connues et évolutions prévues

- **Polling** : l'interface interroge le serveur toutes les 2 secondes. Évolution prévue : SSE ou service
  temps réel.
- **Expiration des parties** : les anciennes parties ne sont jamais supprimées (`updated_at` permettra un
  nettoyage périodique).
- **Connexions PostgreSQL** : chaque instance de fonction ouvre ses propres connexions (5 au plus) ; en
  production, utiliser l'URL « poolée » du fournisseur.
- **Déploiement non vérifié** : le runtime Rust de Vercel est en bêta ; l'espace de travail Cargo (core à
  part) n'a pas été testé par un vrai déploiement.
- **Limitation de débit** : aucune limite sur la création et la jonction de parties.
- **Comptes utilisateurs** : aucun ; l'identité est un jeton secret par partie.
- **Nombre de joueurs** : plafonné à 4 en v1 (`MAX_PLAYERS`) ; le core n'en dépend pas ailleurs.
