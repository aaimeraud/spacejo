# Releases avec git-cliff, et règle de version en 0.x

## Contexte

Les releases doivent être automatisées : version calculée d'après les commits (Conventional Commits),
changelog généré, release GitHub publiée quand un tag est poussé. Le projet est en version 0.x.

## Décision

- **git-cliff** (https://git-cliff.org) génère `docs/CHANGELOG.md` et calcule la version suivante
  (`git cliff --bumped-version`). Raisons : il lit directement les Conventional Commits déjà imposés par
  `scripts/check-commit.sh`, c'est un binaire unique sans dépendance au projet (pas de Node ni de Python),
  et il fournit une action GitHub (`orhun/git-cliff-action`) pour les notes de release.
- `cliff.toml` part du modèle `git cliff --init detailed`, adapté : sections en français par type de
  commit, scope affiché en gras, changements cassants signalés, commits non conformes ignorés
  (`filter_unconventional = true`), commits `chore(release)` exclus.
- **Section `[bump]`** : `breaking_always_bump_major = false` et `features_always_bump_minor = true`.
  D'après la documentation de git-cliff, avec `breaking_always_bump_major = true` un changement cassant
  passe toujours à la version majeure, y compris de 0.x à 1.0.0. Vérifié par
  `tests/scripts/release.test.sh` : après `v0.1.0`, un commit `feat!:` donne `v0.2.0` avec notre réglage,
  et `v1.0.0` avec le réglage inverse.
- `scripts/release.sh` crée le commit et le tag localement mais ne pousse jamais ; le propriétaire du dépôt
  pousse le tag, ce qui déclenche `.github/workflows/release.yml` (permission `contents: write`
  seulement, actions épinglées sur des versions précises).

## Alternatives écartées

- **release-please** : passe par des pull requests de release et une configuration propre à GitHub ;
  plus lourd pour un projet de cours.
- **semantic-release** : écosystème Node, publie automatiquement à chaque fusion (on veut garder la main).
- **Changelog écrit à la main** : se désynchronise de l'historique.

## Conséquences

- Les messages de commit sont le journal du projet : un message mal écrit se retrouve tel quel dans le
  changelog.
- Le passage à 1.0.0 sera un choix explicite (`scripts/release.sh 1.0.0`), jamais un effet de bord.
