# Parties publiques, parties privées et lien d'invitation

## Contexte

Après la release v1.0.0, le propriétaire du dépôt a demandé trois choses : partager un lien d'invitation à la
création d'une partie, pouvoir rejoindre une partie publique, et distinguer parties publiques et privées.
Jusque-là, on ne rejoignait une partie qu'en tapant son code de 6 caractères.

## Décision

- **Visibilité dans une colonne `games.public`** (migration `0003_public_games.sql`), pas dans l'état `state`.
  Ce n'est pas une règle du jeu (elle ne change rien au déroulement d'une partie), donc elle n'a rien à faire dans
  `Game` ni dans `Rules` : le core ne change pas et le format enregistré garde sa version (`STATE_VERSION` = 2),
  sans migration de l'état. Une colonne se filtre et s'indexe en SQL (index partiel `games_public_recent`).
- **Choisie à la création** (`public` dans `POST /api/create`), privée par défaut : rendre visible une partie à
  tout le monde doit être un choix explicite de l'hôte.
- **Liste `GET /api/public-games`, sans jeton** : parties publiques au salon, pas pleines, dont l'hôte est
  présent, les plus récentes d'abord, au plus 20. La présence de l'hôte (table `presence`, déjà tenue à jour par
  le polling) fait disparaître d'elle-même un salon abandonné, sans tâche de fond ni date d'expiration. La phase
  et le nombre de joueurs sont lus dans le JSONB (`state -> 'game' ->> 'phase'`) : pas de colonne en double à
  garder synchronisée avec l'état.
- **Lien d'invitation `/?code=K7MPQX`**, fabriqué par l'interface seule : le serveur sert déjà `index.html` à
  toute adresse, il n'y a rien à ajouter côté API. Le lien ne contient que le code, information déjà partagée.

## Alternatives écartées

- **`public` dans `StoredGame`** : il aurait fallu un `state_version` 3 et une migration de l'état, et filtrer la
  liste sur un champ JSON sans index.
- **Lien `/rejoindre/K7MPQX`** (chemin plutôt que paramètre) : aussi simple côté serveur, mais il faut ensuite
  lire et nettoyer le chemin ; le paramètre se retire de l'adresse avec `history.replaceState` sans changer de page.
- **Bascule publique/privée au salon** : utile, mais elle demande une route d'écriture de plus (hôte, avant le
  lancement, `expected_version`). Notée dans `IDEAS.md`.
- **Expiration par date** (salons modifiés il y a moins de N minutes) : un hôte qui attend longtemps ses amis
  verrait sa partie disparaître ; la présence de l'hôte reflète mieux un salon vivant.

## Conséquences

- Les noms des hôtes de parties publiques sont visibles de tous ; pas de modération (voir « Limites connues »
  dans `ARCHITECTURE.md`).
- La liste n'a pas de limitation de débit, comme la création et la jonction.
- Les parties existantes deviennent privées (valeur par défaut de la colonne) : aucun changement pour elles.
