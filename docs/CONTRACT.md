# Contrat de l'API

Référence commune du front (`web/src/api/types.ts`) et du back (`src/`, `crates/spacejo-core`). Toute
évolution du format passe d'abord par cette page.

## Généralités

- Toutes les requêtes et réponses sont en JSON (`Content-Type: application/json`), encodé en UTF-8.
- Corps de requête limité à 16 Kio (`MAX_BODY_BYTES` dans `src/handlers.rs`) : au-delà, réponse `413`.
- Un champ inconnu, une valeur d'énumération inconnue ou un type incorrect est refusé (`400`).
- **Jeton** : `create` et `join` renvoient un `token` (64 caractères hexadécimaux, 256 bits aléatoires).
  Il n'est remis qu'une fois. Les autres routes l'attendent dans l'en-tête `X-Player-Token`. Le serveur
  ne stocke que son hachage SHA-256.
- **Version** : chaque partie a un numéro `version` (entier, 1 à la création, +1 à chaque écriture :
  jonction, départ, retrait, réglages, visibilité, lancement, action, passage d'un tour). `rules`, `visibility`, `kick`, `start`, `action` et `skip` envoient `expected_version` ; si
  elle diffère de la version en base, réponse `409 version_conflict` et rien n'est écrit. Une requête
  rejouée échoue donc aussi.
- **Vue filtrée** : chaque réponse qui contient l'état le donne sous la forme `view` (type `GameView`
  ci-dessous), calculée pour le joueur qui demande. Une carte face cachée n'a jamais de valeur.
- **Code de partie** : 6 caractères de l'alphabet `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (ni `0`/`O` ni
  `1`/`I`). Le serveur accepte les minuscules et les espaces autour, et les convertit.
- **Nom de joueur** : 1 à 20 caractères après suppression des espaces autour.
- **Limitation de débit** : `create`, `join`, `public-games` et `chat` sont limitées par adresse IP et par minute (10,
  30, 60 et 30 requêtes ; constantes de `src/rate_limit.rs`). Au-delà : `429 rate_limited`, avec l'en-tête `Retry-After`
  (secondes avant de réessayer). L'adresse est lue dans l'en-tête `CF-Connecting-IP` (écrit par Cloudflare devant
  Render), sinon c'est celle de la connexion.

## Routes

### `POST /api/create`

Crée une partie ; le créateur en est l'hôte (joueur `0`).

Requête :
```json
{ "name": "Alice", "rules": { ...Rules }, "public": true }
```
`rules` est facultatif : absent, ce sont les règles officielles (`Rules` par défaut). S'il est présent,
il doit être complet : les 7 réglages d'origine sont obligatoires ; `deck` est facultatif (absent : `Full`,
les règles officielles), pour qu'une interface encore ancienne continue de fonctionner pendant une mise à jour.
`public` est facultatif (booléen) : absent ou `false`, la partie est privée (on ne la rejoint qu'avec son code) ;
`true`, elle est listée par `GET /api/public-games` tant qu'elle est au salon. L'hôte peut changer ce choix au salon
(`POST /api/visibility`).

Réponse `201` :
```json
{ "code": "K7MPQX", "token": "<64 hex>", "player_id": 0, "version": 1, "view": { ...GameView } }
```

### `POST /api/join`

Rejoint une partie au salon (pas encore lancée, moins de 10 joueurs).

Requête : `{ "code": "K7MPQX", "name": "Bob" }`

Réponse `200` : même forme que `create` (`player_id` = ordre d'arrivée, à partir de 0).

### `POST /api/rules` (hôte, avant le lancement)

En-tête `X-Player-Token`. Requête :
```json
{ "code": "K7MPQX", "expected_version": 3, "rules": { ...Rules } }
```
`rules` est complet : il remplace tous les réglages. Réponse `200` : `{ "version": 4, "view": { ... } }`.

### `POST /api/visibility` (hôte, avant le lancement)

En-tête `X-Player-Token`. Requête :
```json
{ "code": "K7MPQX", "expected_version": 4, "public": true }
```
Rend la partie publique (`true`) ou privée (`false`). La version augmente comme pour toute écriture (les autres
joueurs sont prévenus et relisent l'état), même si la valeur ne change pas. Refus : `not_host` si l'appelant n'est
pas l'hôte, `wrong_phase` si la partie est lancée. Réponse `200` : `{ "version": 5, "public": true, "view": { ... } }`.

### `POST /api/leave`

En-tête `X-Player-Token`. Requête : `{ "code": "K7MPQX" }` (pas d'`expected_version` : un départ n'est jamais
refusé pour conflit ; le serveur relit et réessaie, comme pour `join`).
- **Au salon** : le joueur est retiré. Les joueurs arrivés après lui avancent d'une place (leur `player_id` baisse
  de 1 ; leur jeton reste valable) ; si c'était l'hôte, le joueur suivant devient l'hôte. Le jeton du joueur
  parti n'est plus reconnu (`401 invalid_token`). Seul au salon, rien n'est écrit.
- **Partie lancée** : le joueur abandonne. Il est éliminé et marqué `abandoned` dans la vue de tous ; s'il tenait
  une carte, elle va sur la défausse et le tour passe. S'il ne reste qu'un joueur, il gagne.
- **Partie finie** : rien n'est écrit.

Réponse `200` : `{ "version": 8 }` (la version après le départ, ou la version inchangée si rien n'est écrit).

### `POST /api/kick` (hôte, avant le lancement)

En-tête `X-Player-Token`. Requête : `{ "code": "K7MPQX", "expected_version": 4, "player": 2 }`.
Retire `player` du salon, avec les mêmes effets qu'un départ (places décalées, jeton du joueur retiré refusé).
Refus : `not_host`, `cannot_kick_host` (l'hôte se viserait lui-même), `wrong_phase` après le lancement,
`unknown_player`. Réponse `200` : `{ "version": 5, "view": { ... } }`.

### `POST /api/start` (hôte)

En-tête `X-Player-Token`. Requête : `{ "code": "K7MPQX", "expected_version": 4 }`.
Il faut au moins 2 joueurs. La partie passe à la phase `InitialReveal` (chaque joueur retourne 2 cartes).
Réponse `200` : `{ "version": 5, "view": { ... } }`.

### `POST /api/action`

En-tête `X-Player-Token`. Requête :
```json
{ "code": "K7MPQX", "expected_version": 5, "action": { ...Action } }
```
Réponse `200` : `{ "version": 6, "view": { ... } }`.

### `POST /api/skip` (hôte)

En-tête `X-Player-Token`. Requête :
```json
{ "code": "K7MPQX", "expected_version": 6, "player": 2 }
```
Passe tout de suite le tour du joueur absent qui bloque la partie (celui de `waiting.player`), sans attendre la
fin de `waiting.seconds_left`. Les conséquences sont celles d'un passage automatique : tour manqué compté,
élimination au `max_missed_turns`-ième. Refus : `not_host` si l'appelant n'est pas l'hôte, `player_not_absent` si
`player` est présent ou n'est pas celui que l'on attend. Réponse `200` : `{ "version": 7, "view": { ... } }`.

### `GET /api/public-games`

Parties publiques que l'on peut rejoindre. Pas de jeton, aucun paramètre (un paramètre donne `400 invalid_request`).
Sont listées les parties publiques encore au salon (phase `Lobby`), avec moins de `max_players` joueurs, dont
l'hôte a fait une requête depuis moins de 15 s (`PRESENCE_TIMEOUT_SECS`) ; les plus récemment modifiées d'abord,
au plus 20 (`PUBLIC_GAMES_LIMIT`, dans `src/handlers.rs`).

Réponse `200` :
```json
{ "games": [ { "code": "K7MPQX", "host": "Alice", "players": 2, "max_players": 10 } ] }
```
Aucune carte, aucun jeton : seulement le code, le nom de l'hôte et le nombre de joueurs.

### `GET /api/health`

Contrôle de santé, pour l'hébergeur (et pour réveiller le serveur) : ne lit ni la base ni de jeton.
Réponse `200` : `{ "status": "ok" }`.

### `POST /api/chat`

En-tête `X-Player-Token`. Requête : `{ "code": "K7MPQX", "text": "Bien joué !" }`.
Envoie un message dans le chat de la partie (à n'importe quelle phase). Le texte, sans les espaces autour, fait de
1 à 200 caractères (`MAX_CHAT_CHARS`, dans `src/handlers.rs`), sur une seule ligne et sans caractère de contrôle ;
sinon `400 invalid_message`. Les messages ne font pas partie de l'état du jeu : la `version` ne change pas.
Réponse `200` : `{ "message": { "id": 12, "player": 1, "name": "Bob", "text": "Bien joué !" } }`.

### `GET /api/messages?code=K7MPQX&after=11`

En-tête `X-Player-Token` (un joueur de cette partie). `after` est facultatif (0 par défaut).
Réponse `200` : `{ "messages": [ { "id": 12, "player": 1, "name": "Bob", "text": "Bien joué !" } ] }` : les
messages dont `id` est plus grand que `after`, du plus ancien au plus récent, au plus les 50 derniers
(`MAX_CHAT_HISTORY`). `name` est le nom du joueur au moment de l'envoi. Paramètre inconnu ou `after` non entier :
`400 invalid_request`.

### `GET /api/events?code=K7MPQX`

Flux d'avis (server-sent events, `text/event-stream`) : la connexion reste ouverte. Chaque nouvelle version de la
partie envoie un évènement `version` dont la donnée est le numéro ; chaque nouveau message du chat, un évènement
`chat` dont la donnée est le numéro du message :

```
event: version
data: 7

event: chat
data: 12
```

Pas de jeton : le flux ne révèle que des numéros (jamais le texte d'un message). Le client relit alors l'état avec
`GET /api/state`, ou les messages avec `GET /api/messages`.
Des commentaires (`:`) maintiennent la connexion toutes les 15 s. Code mal formé : `400 invalid_code`.

### `GET /api/state?code=K7MPQX&known_version=6`

En-tête `X-Player-Token`. `known_version` est facultatif.
- Si `known_version` est égal à la version en base : `200 { "changed": false, "version": 6, "presence": [...], "waiting": ... }`
  (sans `view`).
- Sinon : `200 { "changed": true, "version": 7, "view": { ... }, "presence": [...], "waiting": ... }`.

Lire l'état note aussi que le joueur est présent (sans modifier la partie ni sa version). Si un joueur absent
bloque la partie depuis plus de 45 s, cette lecture passe son tour (voir « Présence » ci-dessous) : la version
augmente alors de 1.

## Types

### `Rules`

```json
{
  "penalty": "DoubleUnlessStrictlyLowest",
  "penalty_scope": "RoundTotal",
  "first_player_tie": "JoinOrder",
  "next_round_starter": "Finisher",
  "empty_pile": "ReshuffleDiscard",
  "end_tie": "SharedVictory",
  "end_score": 100,
  "deck": "Full"
}
```
Valeurs permises (défaut en premier) :
- `penalty` : `DoubleUnlessStrictlyLowest` | `DoubleOnlyIfBeaten` | `Disabled`
- `penalty_scope` : `RoundTotal` | `PositiveCardsOnly`
- `first_player_tie` : `JoinOrder` | `Random`
- `next_round_starter` : `Finisher` | `RoundWinner` | `Rotate`
- `empty_pile` : `ReshuffleDiscard` | `EndRound`
- `end_tie` : `SharedVictory` | `ExtraRound`
- `end_score` : entier de 10 à 500
- `deck` : `Full` | `Reduced` (pioche réduite : 40 cartes par joueur en jeu, au plus 150 ; voir docs/RULES.md)

### `Action`

Objet avec un champ `type`. Les positions sont `column` (0 à 3, de gauche à droite) et `row` (0 à 2, de
haut en bas), dans la grille d'origine (une colonne retirée garde son numéro, mais n'est plus jouable).

| `type` | Champs | Phase | Effet |
|---|---|---|---|
| `RevealInitial` | `column`, `row` | `InitialReveal` | retourne une de ses cartes (2 par joueur, sans ordre de tour) |
| `DrawFromPile` | — | `Playing`, rien en main | prend la carte du dessus de la pioche |
| `TakeDiscard` | — | `Playing`, rien en main | prend la carte du dessus de la défausse ; il faudra l'échanger |
| `SwapHeld` | `column`, `row` | `Playing`, carte en main | échange la carte en main avec la carte visée ; l'ancienne va, face visible, sur la défausse |
| `DiscardHeldAndReveal` | `column`, `row` | `Playing`, carte de la pioche en main | défausse la carte en main et retourne la carte visée (qui doit être cachée) |
| `NextRound` | — | `RoundOver` | distribue la manche suivante (n'importe quel joueur) |

Exemples : `{ "type": "DrawFromPile" }`, `{ "type": "SwapHeld", "column": 2, "row": 0 }`.

### `GameView`

```json
{
  "me": 1,
  "host": 0,
  "phase": "Playing",
  "round": 1,
  "rules": { ...Rules },
  "players": [
    {
      "id": 0,
      "name": "Alice",
      "total_score": 0,
      "initial_reveals": 2,
      "missed_turns": 0,
      "eliminated": false,
      "grid": [
        { "kind": "Present", "slots": [ { "kind": "Hidden" }, { "kind": "Visible", "value": 7 }, { "kind": "Hidden" } ] },
        { "kind": "Removed" }
      ]
    }
  ],
  "current_player": 0,
  "finisher": null,
  "held": { "source": "Pile", "value": null },
  "draw_pile_count": 120,
  "discard_top": 4,
  "discard_count": 3,
  "rounds": [
    { "round": 1, "finisher": 0, "scores": [ { "player": 0, "raw": 12, "score": 24, "penalized": true } ] }
  ],
  "winners": []
}
```

- `phase` : `Lobby` | `InitialReveal` | `Playing` | `RoundOver` | `GameOver`.
- `players` : dans l'ordre d'arrivée (`id` = indice). `grid` : liste des colonnes (vide au salon), chaque
  colonne est `Present` (3 cases, de haut en bas) ou `Removed`. Une case est `Hidden` (sans valeur) ou
  `Visible` avec `value`.
- `initial_reveals` : nombre de cartes déjà retournées pendant la phase `InitialReveal`.
- `current_player` : joueur dont c'est le tour (`null` hors phase `Playing`).
- `finisher` : joueur qui a retourné toutes ses cartes dans la manche en cours (`null` sinon). Les autres
  jouent alors leur dernier tour.
- `held` : carte en main du joueur courant (`null` s'il n'en a pas). `value` n'est donné qu'au joueur
  courant quand la carte vient de la pioche ; une carte prise sur la défausse était déjà publique, sa
  valeur est donc donnée à tous.
- `discard_top` : valeur de la carte du dessus de la défausse (`null` si elle est vide).
- `rounds` : scores de chaque manche terminée. `raw` = somme des cartes, `score` = score retenu après
  pénalité, `penalized` = la pénalité a été appliquée. C'est le serveur qui calcule : l'interface n'additionne rien.
- `winners` : identifiants des gagnants (rempli seulement en phase `GameOver`).

### Visibilité et présence (`public`, `presence`, `waiting`)

Toutes les réponses qui donnent l'état (`create`, `join`, `rules`, `visibility`, `start`, `action`, `skip`, `state`) contiennent aussi :

```json
{
  "public": false,
  "presence": [ { "player": 0, "connected": true }, { "player": 1, "connected": false } ],
  "waiting": { "player": 1, "seconds_left": 30, "missed_turns": 0, "max_missed_turns": 2 }
}
```

- `public` : la partie est publique (listée par `GET /api/public-games` tant qu'elle est au salon) ou privée.
- `presence` : pour chaque joueur, `connected` vaut `false` s'il n'a fait aucune requête depuis 15 s
  (`PRESENCE_TIMEOUT_SECS`, dans `src/handlers.rs`).
- `waiting` : `null`, ou le joueur absent qui bloque la partie (c'est son tour, ou il n'a pas retourné ses cartes
  de mise en place). `seconds_left` : secondes avant que son tour soit passé (45 s après la dernière modification
  de la partie, `ABSENT_TURN_WAIT_SECS`), sauf si l'hôte le passe avant (`POST /api/skip`). `missed_turns` : tours déjà manqués d'affilée ; au `max_missed_turns`-ième,
  il est éliminé.
- Dans `GameView.players`, `missed_turns`, `eliminated` et `abandoned` (le joueur a quitté la partie de lui-même,
  il est alors aussi éliminé) décrivent la situation de chaque joueur. Un joueur
  éliminé ne joue plus, ne reçoit plus de cartes, et n'apparaît plus dans les scores des manches suivantes ni dans
  le classement.

## Erreurs

Toute erreur a la forme :
```json
{ "error": { "code": "not_your_turn", "message": "Ce n'est pas votre tour." } }
```

| Statut | `code` | Cas |
|---|---|---|
| 400 | `invalid_request` | JSON invalide, champ inconnu ou manquant, valeur inconnue, paramètre absent |
| 400 | `invalid_name` | nom vide ou trop long |
| 400 | `invalid_code` | code de partie mal formé |
| 400 | `invalid_rules` | réglage hors bornes (`end_score` hors de 10 à 500) |
| 400 | `invalid_message` | message du chat vide, trop long, sur plusieurs lignes ou avec un caractère de contrôle |
| 401 | `missing_token` | en-tête `X-Player-Token` absent |
| 401 | `invalid_token` | jeton qui ne correspond à aucun joueur de cette partie |
| 403 | `not_host` | réglages, visibilité, lancement ou passage d'un tour demandés par un joueur qui n'est pas l'hôte |
| 404 | `game_not_found` | aucune partie avec ce code |
| 404 | `not_found` | route inconnue (serveur local) |
| 405 | `method_not_allowed` | mauvaise méthode HTTP |
| 409 | `version_conflict` | `expected_version` périmée, ou écriture concurrente |
| 413 | `payload_too_large` | corps de requête trop gros |
| 429 | `rate_limited` | trop de `create`, `join`, `public-games` ou `chat` depuis la même adresse (en-tête `Retry-After`) |
| 422 | `wrong_phase` | action impossible dans la phase actuelle (dont réglages ou visibilité après lancement) |
| 422 | `not_your_turn` | action hors de son tour |
| 422 | `game_full` | déjà 10 joueurs |
| 422 | `player_not_absent` | passage du tour d'un joueur présent, ou qui ne bloque pas la partie |
| 422 | `eliminated` | le joueur ne joue plus (éliminé pour absence, ou il a abandonné) |
| 422 | `cannot_kick_host` | l'hôte demande à se retirer lui-même du salon |
| 422 | `unknown_player` | joueur visé inexistant dans cette partie (`kick`) |
| 422 | `not_enough_players` | lancement à moins de 2 joueurs |
| 422 | `invalid_position` | colonne ou ligne hors de la grille, ou colonne retirée |
| 422 | `already_revealed` | carte déjà visible (retournement) |
| 422 | `initial_reveals_done` | le joueur a déjà retourné ses 2 cartes |
| 422 | `already_holding` | le joueur a déjà une carte en main |
| 422 | `nothing_held` | échange ou défausse sans carte en main |
| 422 | `must_swap_discard` | défausser une carte prise sur la défausse (interdit : il faut l'échanger) |
| 422 | `empty_discard` | défausse vide |
| 422 | `empty_pile` | pioche vide (ne devrait pas arriver : elle est reconstituée à la fin du tour) |
| 500 | `internal` | erreur de stockage ou état illisible (le détail n'est pas renvoyé) |
| 500 | `unsupported_state_version` | partie stockée dans un format que ce serveur ne sait pas lire |
