# Architecture de SpaceJo

Page d'entrée pour découvrir le projet. Elle décrit le code tel qu'il est ; les choix et leurs raisons
sont dans [decisions/](decisions/).

## Vue d'ensemble

SpaceJo est un jeu de cartes au tour par tour (2 à 10 joueurs) qui reprend les règles de Skyjo
([RULES.md](RULES.md)). Trois composants :

```
 Navigateur (web/, React)            Serveur Rust (Render, Docker)               PostgreSQL (Supabase)
┌──────────────────────┐   HTTPS    ┌────────────────────────────────────┐   SQL   ┌──────────────┐
│ écrans, polling 2 s   │ ─────────▶ │ src/server.rs (axum) :             │ ──────▶ │ table games  │
│ envoie des Action     │ ◀───────── │  /api/* → src/handlers.rs          │ ◀────── │ (state JSONB,│
│ affiche la vue reçue  │ vue filtrée│  le reste → web/dist (l'interface) │         │  version)    │
└──────────────────────┘            │ crates/spacejo-core : le jeu       │         └──────────────┘
                                    └────────────────────────────────────┘
```

| Dossier | Rôle |
|---|---|
| `crates/spacejo-core/` | logique du jeu (Rust pur, sans réseau, sans base, sans horloge) |
| `src/` | bibliothèque `spacejo-api` : routes, jetons, format enregistré, stockage, serveur HTTP |
| `src/bin/` | `server` (l'API et l'interface, en local comme en production) et `migrate` |
| `migrations/` | migrations SQL versionnées, appliquées par `sqlx::migrate!` |
| `tests/` | tests de l'API (mémoire et PostgreSQL), exemple figé du format enregistré, tests des scripts |
| `web/` | interface React + TypeScript + Vite |
| `scripts/` | contrôle des commits, release, aperçu du changelog |
| `Dockerfile`, `render.yaml` | image de production (interface puis serveur) et déploiement sur Render |

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

## L'API : `src/`

Les routes et leurs formats JSON sont décrits dans [CONTRACT.md](CONTRACT.md).

| Fichier | Rôle |
|---|---|
| `src/handlers.rs` | les neuf opérations (dont la liste des parties publiques et la visibilité), indépendantes du serveur HTTP (`handle(store, endpoint, request)`) |
| `src/auth.rs` | jetons (256 bits, hachage SHA-256, comparaison en temps constant), codes de partie, graines |
| `src/stored.rs` | `StoredGame` (ce qui est dans `games.state`) et `STATE_VERSION` |
| `src/store.rs` | `Store` : PostgreSQL (`sqlx`) ou mémoire, avec le verrou optimiste ; liste des parties publiques au salon (`list_public`) et visibilité (`set_public`) |
| `src/events.rs` | avis de changement de version, un canal `broadcast` par partie, poussés aux navigateurs par `GET /api/events` |
| `src/server.rs` | serveur `axum` : `/api/health`, `/api/*` vers `handlers` (réponses compressées), fichiers de l'interface par `tower-http` (version gzip, règles de cache, retour à `index.html` pour une page de l'application) |
| `src/error.rs` | `ApiError` (statut, code, message) et conversion depuis `GameError` |

### Parcours d'une action, du clic à la réponse

1. Le joueur clique sur une carte. L'interface envoie `POST /api/action` avec
   `{ code, expected_version, action }` et l'en-tête `X-Player-Token`. Elle envoie une intention
   (`Action`), jamais un état.
2. Le serveur (`server.rs`) reconnaît la route `Endpoint::Action` d'après le chemin. Le corps est lu avec une
   taille maximale (16 Kio), puis `handlers::handle` prend le relais.
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
le pool de connexions PostgreSQL survit d'une requête à l'autre. On pourrait donc lancer plusieurs serveurs
côte à côte : le verrou optimiste suffit à les départager.

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
| `src/invite.ts` | lien d'invitation (`/?code=K7MPQX`) : le fabriquer, et relire le code d'une adresse |
| `src/api/events.ts` | flux d'avis du serveur (`EventSource`) : nouvelle version → relecture immédiate |
| `src/App.tsx` | session, polling, envoi des actions avec `expected_version`, choix de l'écran |
| `src/components/` | `Home` (accueil et parcours pas à pas), `PublicGames` (liste des parties publiques de l'accueil), `Lobby` (salon, lien d'invitation), `GameBoard` (table de jeu et fin de partie), `GridView`, `Card` (carte 3D à deux faces), `Scoreboard`, `Modal` (fenêtres), `FitStage` (écran figé mis à l'échelle), `ThemeToggle` (bascule clair / sombre), `icons.tsx` (icônes Lucide des réglages) |
| `src/components/wizard/RuleSteps.tsx` | étapes du parcours de réglages (type de partie, un réglage par écran, score de fin, récapitulatif), partagées par l'accueil et le salon |
| `src/game/seats.ts` | place de chaque joueur autour de la table (moi en bas, puis l'ordre du tour, dans le sens des aiguilles d'une montre) |
| `src/rulesInfo.ts` | explications, exemples, étiquettes « officiel » / « variante », préréglages (raccourcis de l'interface) et résumé des réglages |
| `src/game/focus.ts` | en portrait, quel adversaire montrer en grand (choisi à la main, sinon celui qui joue, sinon celui qui vient de jouer) |
| `src/game/logic.ts` | quelle action envoie un clic, consigne à afficher, classement (simple tri des totaux) |
| `public/` | fichiers servis tels quels : icônes (`favicon.ico`, `favicon.svg`, `apple-touch-icon.png`, `icons/`), image d'aperçu `og-image.png`, manifeste `manifest.webmanifest` et service worker `sw.js` de l'application installable (voir la décision « application installable ») |
| `src/theme.ts` | thème clair ou sombre : celui du système par défaut, ou le choix gardé dans `localStorage` ; appliqué par `data-theme` sur `<html>` |
| `src/session.ts` | session de l'onglet (code, jeton, identifiant) dans `sessionStorage`, et parties mémorisées dans `localStorage` pour la reprise |
| `src/config.ts` | constantes : `POLL_INTERVAL_MS` (2000), `POLL_LOBBY_INTERVAL_MS` (3000), `POLL_IDLE_INTERVAL_MS` (10000), bornes, longueurs |
| `src/polling.ts` | rythme du polling selon la phase, la visibilité de l'onglet et le flux d'avis |

**Table de jeu (CSS 3D)** : la partie se joue sur une table ovale inclinée en perspective (`rotateX`). Les
adversaires sont assis autour, à la place calculée par `seats.ts` (un angle sur une ellipse, pour n'importe
quel nombre de joueurs), avec un badge (nom, total, cartes cachées, hôte, « joue », « a terminé »). Pioche et
défausse sont au centre, la carte en main flotte au-dessus ; ma grille est devant moi, sur le bord de la table.
Avec une carte piochée en main, deux tuiles à bascule (« Échanger », « Défausser et retourner », boutons
`aria-pressed`) choisissent ce que fera le clic sur une carte de ma grille.
La table est inclinée (`rotateX(40deg)`, variable CSS `--table-tilt`) et ma grille est très légèrement couchée
devant moi. Les grilles des adversaires sont posées à plat sur la table, tournées vers leur propriétaire comme de
vraies cartes posées devant lui (`Seat.rotation` : 180° en face, donc à l'envers pour moi, 90° à gauche, 270° à
droite) : la perspective de la table les déforme. Le badge de chaque adversaire est placé juste au-dessus de ses
cartes puis redressé (rotation inverse de la table, `rotateX(-40deg)`) : il se tient debout, face à moi et
lisible, quelle que soit l'orientation des cartes. En portrait, les grilles restent droites pour être lisibles.
La grille de l'adversaire dont c'est le tour est agrandie, vers le centre de la table. Chaque carte a deux
faces dos à dos : la retourner, c'est la faire pivoter de 180° (`rotateY`). Une carte posée et la défausse ont
une petite animation d'apparition. Quand une colonne de trois cartes identiques est retirée, ses cartes
s'illuminent, s'envolent et disparaissent avec le message « Colonne identique ! » : `game/columns.ts` repère ces
colonnes en comparant la vue précédente et la nouvelle. En portrait (téléphone), on ne
montre qu'un adversaire à la fois, en grand : celui qui joue, ou, pendant mon tour, celui qui vient de jouer
(`game/focus.ts`). Une bande en haut de la table donne un bouton par adversaire (initiale, nom, total ; pastille
« joue », « absent » ou « éliminé ») pour regarder le jeu d'un autre, jusqu'au changement de tour. À 2 joueurs,
la bande disparaît. Avec la préférence système « réduire les animations », il n'y a aucune
animation. Le choix du CSS 3D plutôt que WebGL est expliqué dans
[decisions/2026-11-03-vue-table-css-3d.md](decisions/2026-11-03-vue-table-css-3d.md).

**Écran figé** (`FitStage.tsx`) : l'application occupe exactement la fenêtre ; rien ne défile, comme dans un jeu.
Chaque écran (accueil, salon, partie, fin de partie) est dessiné à une taille de référence, en paysage
(par exemple 1280 × 900 pour la partie) ou en portrait (480 × 1000) selon la forme de la fenêtre, puis mis à
l'échelle pour tenir dans la place disponible. Les classes `.is-landscape` / `.is-portrait` choisissent la
disposition ; `crowd-few`, `crowd-some`, `crowd-many` réduisent les sièges adverses quand il y a beaucoup de
joueurs (jusqu'à 10). Seul le contenu des fenêtres (`Modal` : scores, règles) peut défiler.

**Accueil et réglages, pas à pas** : l'accueil propose « Créer une partie », « Rejoindre une partie » et les
parties à reprendre. Créer : nom, type de partie (« Règles officielles », « Détente », « Partie rapide »,
« Personnalisée »), puis, pour « Personnalisée », un écran par réglage (grandes tuiles avec explication, exemple
chiffré et étiquette « officiel » ou « variante » ; curseur et raccourcis pour le score de fin), enfin un
récapitulatif où chaque ligne se modifie d'un clic. On peut revenir en arrière à chaque étape. L'hôte modifie
les règles au salon avec le même parcours, dans une fenêtre. Un préréglage n'est qu'un raccourci : le serveur
reçoit toujours les 7 réglages.

**Icônes** : `lucide-react` (licence ISC). Elles sont décoratives (`aria-hidden`) : un texte, visible ou réservé
aux lecteurs d'écran, porte toujours l'information.

**Avis et polling** : le navigateur ouvre un flux d'avis (`GET /api/events?code=…`, server-sent events,
`api/events.ts`) ; à chaque nouvelle version annoncée, il relit l'état aussitôt (`GET /api/state`, avec son
jeton). Le polling continue en secours : toutes les 5 secondes quand le flux est ouvert (présence, avis perdu),
sinon 2 en partie, 3 au salon, 10 quand l'onglet est caché ou la partie finie (`polling.ts`) ; revenir sur
l'onglet relit aussitôt. Si la version n'a pas changé,
la réponse est minimale (`changed: false`) et rien n'est redessiné.

**Conflits** : chaque écriture envoie la version connue. En cas de 409 (un autre joueur a joué entre-temps,
fréquent pendant la mise en place où tous retournent leurs cartes en même temps), l'interface affiche le
message, relit l'état complet, et le joueur refait son geste.

**Session et reprise** : la session de l'onglet vit dans `sessionStorage` (propre à chaque onglet, conservée au
rechargement : on peut jouer à deux dans deux onglets du même navigateur). Chaque partie rejointe est aussi
mémorisée dans `localStorage` (5 au plus) : après avoir fermé l'onglet, l'accueil propose « Reprendre une partie ».
Une partie quittée avec « Quitter », terminée, ou refusée par le serveur (jeton invalide, partie disparue) est oubliée.

**Absents et éliminés** : les badges affichent « absent » (d'après `presence`) et « éliminé » ; la grille d'un
joueur éliminé est grisée. Quand on attend un absent, un bandeau donne le compte à rebours reçu du serveur
(`waiting`) et dit si ce sera son dernier tour manqué.

**Accessibilité** : cartes et commandes sont des `button` (clavier, focus visible) avec un libellé
(« Carte cachée, colonne 2, ligne 1 », « Carte 7, … ») ; la valeur d'une carte est toujours écrite (la
couleur n'est qu'un repère) ; les étiquettes de réglage sont textuelles ; la consigne et les erreurs sont
annoncées (`role="status"`, `role="alert"`).

En développement, Vite (port 5173) renvoie `/api` vers `cargo run` (le serveur, port 3000). En production,
le même serveur sert l'API et l'interface construite (`web/dist`) sur une seule adresse : pas de CORS.

## Parties publiques, parties privées et lien d'invitation

- **Visibilité** : à la création, l'hôte choisit une partie **privée** (par défaut) ou **publique** ; il peut
  changer d'avis au salon (« Rendre publique » / « Rendre privée », `POST /api/visibility`), jamais après le
  lancement. Ce n'est pas une règle du jeu : elle est rangée dans la colonne `games.public`, hors de l'état `state`
  (le core n'en sait rien), et les contrôles d'hôte et de phase sont faits par la route. Changer la visibilité
  incrémente `version` avec le même verrou optimiste qu'une action (`Store::set_public`), ce qui prévient les
  autres joueurs par le flux d'avis. Toutes les réponses qui donnent l'état contiennent `public`, que le salon affiche.
- **Liste des parties publiques** (`GET /api/public-games`, sans jeton) : parties publiques encore au salon, pas
  pleines, dont l'hôte est présent (requête depuis moins de `PRESENCE_TIMEOUT_SECS`) ; les plus récentes d'abord,
  au plus `PUBLIC_GAMES_LIMIT` (20). On n'y donne que le code, le nom de l'hôte et le nombre de joueurs. Un salon
  abandonné par son hôte, une partie lancée ou pleine disparaît donc de lui-même, sans tâche de fond.
  L'accueil la relit toutes les 5 s (`PUBLIC_GAMES_REFRESH_MS`) tant qu'elle est affichée.
- **Lien d'invitation** : `https://<site>/?code=K7MPQX` (`web/src/invite.ts`). Le salon le partage par la feuille
  de partage du système quand elle existe (téléphone), sinon le copie. Ouvert, le lien mène directement à l'étape
  « Comment vous appelez-vous ? », code rempli ; si cet appareil a déjà rejoint la partie (l'hôte qui ouvre son
  propre lien), elle est reprise au lieu d'être rejointe une seconde fois. Le paramètre est retiré de l'adresse
  aussitôt lu. Le lien ne contient que le code, jamais de jeton.

Choix et alternatives : [decisions/2026-11-04-parties-publiques.md](decisions/2026-11-04-parties-publiques.md).

## Présence, absence et élimination

Le serveur ne garde rien en mémoire et ne lance aucune tâche de fond : tout se décide pendant les requêtes.

1. Chaque requête d'un joueur identifié met à jour `presence.last_seen` (`Store::touch`). Cette table est séparée
   de `games` : la présence ne change ni l'état ni la version, donc aucun conflit de verrou optimiste.
2. Un joueur est **absent** s'il n'a rien demandé depuis `PRESENCE_TIMEOUT_SECS` (15 s ; l'interface interroge
   le serveur toutes les 2 s, et un onglet fermé n'interroge plus).
3. Si un absent bloque la partie (c'est son tour, ou il doit retourner ses cartes de mise en place) et que la partie
   n'a pas été modifiée depuis `ABSENT_TURN_WAIT_SECS` (45 s, `games.updated_at`), la prochaine lecture de l'état
   par un joueur présent appelle `Game::skip_absent` puis sauvegarde avec le verrou optimiste.
4. `skip_absent` (dans le core, sans horloge) défausse la carte en main ou retourne les cartes de mise en place,
   compte un tour manqué, et élimine le joueur au deuxième tour manqué d'affilée (`MAX_MISSED_TURNS`). Jouer une
   action remet le compteur à zéro. S'il ne reste qu'un joueur, il gagne.
5. L'hôte n'est pas obligé d'attendre : `POST /api/skip` passe tout de suite le tour de l'absent qui bloque
   (`Game::skip_absent`, mêmes conséquences). Le serveur vérifie que l'appelant est l'hôte et que le joueur visé
   est bien absent et bloquant : on ne peut jamais passer le tour d'un joueur présent.
6. Les réponses donnent `presence` (qui est connecté) et `waiting` (le compte à rebours), que l'interface affiche
   dans le bandeau « X est parti·e », avec, pour l'hôte, un bouton « Passer son tour » (« Éliminer » au dernier
   tour manqué). En portrait, ce bandeau tient sur une ligne et remplace l'indication du tour.

Le core ne connaît que les tours manqués ; l'heure est mesurée par PostgreSQL (`now()`), ou par le stockage en
mémoire, dont l'horloge peut être avancée dans les tests (`MemoryStore::advance`).

## Modèle de données

Table `games`, créée par `migrations/0001_create_games.sql` (colonne `public` ajoutée par
`migrations/0003_public_games.sql`) :

| Colonne | Type | Rôle |
|---|---|---|
| `id` | `BIGSERIAL` | clé primaire |
| `code` | `TEXT UNIQUE` | code de partie à partager (6 caractères sans 0/O ni 1/I) |
| `state` | `JSONB` | `StoredGame` : `state_version`, `game` (état complet du core), `token_hashes` |
| `version` | `INTEGER` | numéro de version pour le verrou optimiste (1 à la création, +1 par écriture) |
| `updated_at` | `TIMESTAMPTZ` | date de la dernière écriture |
| `public` | `BOOLEAN` | partie publique (listée sur l'accueil tant qu'elle est au salon) ; faux par défaut |

Index partiel `games_public_recent` (`updated_at DESC`, seulement les lignes où `public` est vrai) : la liste des
parties publiques ne parcourt pas les parties privées.

Table `presence`, créée par `migrations/0002_create_presence.sql` : `game_id` (référence `games.id`, supprimée
avec la partie), `player_id`, `last_seen` ; clé primaire (`game_id`, `player_id`).

Le schéma n'évolue que par de nouveaux fichiers dans `migrations/` (jamais à la main) ; `cargo run --bin migrate`
les applique, et `sqlx` note celles déjà appliquées dans `_sqlx_migrations`.

Le JSON de `state` a son propre numéro de format, `state_version` (actuellement 2 ; la v1 est migrée à la lecture). Si le format de `Game`
change, il faut incrémenter `STATE_VERSION` et écrire la migration dans `StoredGame::decode` ; un état
illisible est refusé avec l'erreur `unsupported_state_version`. Le test `tests/state_version.rs` relit un
exemple figé (`tests/fixtures/state_v1.json`) pour détecter tout changement de format oublié.

## Journaux

Le serveur n'écrit que la méthode, le chemin et le statut de chaque requête, et le type des erreurs de
stockage. Jamais de jeton, de hachage, de valeur de carte, de graine ni d'état de partie.

## Limites connues et évolutions prévues

- **Polling** : l'interface interroge le serveur toutes les 2 secondes pendant une partie (moins souvent
  sinon) ; chaque requête est un appel de fonction facturé. Évolution prévue : SSE ou service temps réel
  (Supabase Realtime, par exemple).
- **Expiration des parties** : les anciennes parties ne sont jamais supprimées (`updated_at` permettra un
  nettoyage périodique).
- **Connexions PostgreSQL** : le serveur garde au plus 5 connexions, par le pooler de Supabase en mode session
  (le cache de requêtes préparées de sqlx ne marche pas en mode transaction).
- **Avis en mémoire** : avec plusieurs serveurs, les avis d'un serveur ne toucheraient que ses navigateurs (les
  autres verraient les coups au polling, en 5 s au plus).
- **Hébergement gratuit** : sur l'offre gratuite de Render, le serveur s'endort après 15 minutes sans requête et
  met environ une minute à se réveiller ; l'interface le réveille dès l'ouverture de l'accueil.
- **Limitation de débit** : aucune limite sur la création et la jonction de parties, ni sur la liste des parties
  publiques.
- **Modération** : les noms des hôtes de parties publiques sont visibles de tous, sans filtre.
- **Comptes utilisateurs** : aucun ; l'identité est un jeton secret par partie.
- **Nombre de joueurs** : de 2 à 10 (`MAX_PLAYERS`) ; le core n'en dépend pas ailleurs.
