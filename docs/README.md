# Documentation de SpaceJo

Sommaire de la documentation. Chaque page décrit l'état ACTUEL du code ; l'historique est dans
`git log`, dans le changelog généré et dans `decisions/`.

- [ARCHITECTURE.md](ARCHITECTURE.md) : vue d'ensemble des composants, parcours d'une action, modèle de
  données, vue filtrée et verrou optimiste, limites connues.
- [CONCEPTS.md](CONCEPTS.md) : notions Rust rencontrées dans le code, avec le fichier où elles apparaissent.
- [decisions/](decisions/) : une page par décision importante (contexte, décision, alternatives, conséquences).
- `CHANGELOG.md` : journal des versions, GÉNÉRÉ par git-cliff à chaque release (ne pas l'éditer).
