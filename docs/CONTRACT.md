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
  jonction, réglages, lancement, action). `rules`, `start` et `action` envoient `expected_version` ; si
  elle diffère de la version en base, réponse `409 version_conflict` et rien n'est écrit. Une requête
  rejouée échoue donc aussi.
- **Vue filtrée** : chaque réponse qui contient l'état le donne sous la forme `view` (type `GameView`
  ci-dessous), calculée pour le joueur qui demande. Une carte face cachée n'a jamais de valeur.
- **Code de partie** : 6 caractères de l'alphabet `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (ni `0`/`O` ni
  `1`/`I`). Le serveur accepte les minuscules et les espaces autour, et les convertit.
- **Nom de joueur** : 1 à 20 caractères après suppression des espaces autour.

## Routes

### `POST /api/create`

Crée une partie ; le créateur en est l'hôte (joueur `0`).

Requête :
```json
{ "name": "Alice", "rules": { ...Rules } }
```
`rules` est facultatif : absent, ce sont les règles officielles (`Rules` par défaut). S'il est présent,
il doit être complet (les 7 champs).

Réponse `201` :
```json
{ "code": "K7MPQX", "token": "<64 hex>", "player_id": 0, "version": 1, "view": { ...GameView } }
```

### `POST /api/join`

Rejoint une partie au salon (pas encore lancée, moins de 4 joueurs).

Requête : `{ "code": "K7MPQX", "name": "Bob" }`

Réponse `200` : même forme que `create` (`player_id` = ordre d'arrivée, à partir de 0).

### `POST /api/rules` (hôte, avant le lancement)

En-tête `X-Player-Token`. Requête :
```json
{ "code": "K7MPQX", "expected_version": 3, "rules": { ...Rules } }
```
`rules` est complet : il remplace tous les réglages. Réponse `200` : `{ "version": 4, "view": { ... } }`.

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

### `GET /api/state?code=K7MPQX&known_version=6`

En-tête `X-Player-Token`. `known_version` est facultatif.
- Si `known_version` est égal à la version en base : `200 { "changed": false, "version": 6 }`.
- Sinon : `200 { "changed": true, "version": 7, "view": { ... } }`.

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
  "end_score": 100
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
| 401 | `missing_token` | en-tête `X-Player-Token` absent |
| 401 | `invalid_token` | jeton qui ne correspond à aucun joueur de cette partie |
| 403 | `not_host` | réglages ou lancement demandés par un joueur qui n'est pas l'hôte |
| 404 | `game_not_found` | aucune partie avec ce code |
| 404 | `not_found` | route inconnue (serveur local) |
| 405 | `method_not_allowed` | mauvaise méthode HTTP |
| 409 | `version_conflict` | `expected_version` périmée, ou écriture concurrente |
| 413 | `payload_too_large` | corps de requête trop gros |
| 422 | `wrong_phase` | action impossible dans la phase actuelle (dont réglages après lancement) |
| 422 | `not_your_turn` | action hors de son tour |
| 422 | `game_full` | déjà 4 joueurs |
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
