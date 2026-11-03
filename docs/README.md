# Documentation de SpaceJo

Sommaire de la documentation. Chaque page décrit l'état ACTUEL du code ; l'historique est dans
`git log`, dans le changelog généré et dans `decisions/`.

- [ARCHITECTURE.md](ARCHITECTURE.md) : vue d'ensemble des composants, parcours d'une action, modèle de
  données, vue filtrée et verrou optimiste, limites connues.
- [RULES.md](RULES.md) : règles du jeu et réglages de partie (source de vérité), avec le statut et les sources.
- [CONTRACT.md](CONTRACT.md) : routes de l'API, formats JSON exacts des requêtes, réponses et erreurs.
- [CONCEPTS.md](CONCEPTS.md) : notions Rust rencontrées dans le code, avec le fichier où elles apparaissent.
- [decisions/](decisions/) : une page par décision importante (contexte, décision, alternatives, conséquences) :
  - [générateur aléatoire](decisions/2026-11-03-generateur-aleatoire.md) : SplitMix64 écrit dans le core ;
  - [structure Vercel](decisions/2026-11-03-structure-vercel.md) : un fichier par fonction dans `api/` ;
  - [PostgreSQL et sqlx](decisions/2026-11-03-postgresql-sqlx.md) : driver, verrou optimiste, migrations ;
  - [contrôle des commits](decisions/2026-11-03-controle-des-commits.md) : hook, CI et GitButler ;
  - [releases avec git-cliff](decisions/2026-11-03-releases-git-cliff.md) : changelog et règle de version en 0.x.
- `CHANGELOG.md` : journal des versions, GÉNÉRÉ par git-cliff à chaque release (ne pas l'éditer).
