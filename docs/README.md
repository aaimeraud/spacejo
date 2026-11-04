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
  - [structure Vercel](decisions/2026-11-03-structure-vercel.md) : un fichier par fonction dans `api/` (remplacée) ;
  - [PostgreSQL et sqlx](decisions/2026-11-03-postgresql-sqlx.md) : driver, verrou optimiste, migrations ;
  - [contrôle des commits](decisions/2026-11-03-controle-des-commits.md) : hook, CI et GitButler ;
  - [releases avec git-cliff](decisions/2026-11-03-releases-git-cliff.md) : changelog et règle de version en 0.x ;
  - [vue table en CSS 3D](decisions/2026-11-03-vue-table-css-3d.md) : table inclinée et cartes qui se retournent, sans WebGL ;
  - [écran figé](decisions/2026-11-03-ecran-fige.md) : aucune page ne défile, chaque écran est mis à l'échelle ;
  - [thème « nuit »](decisions/2026-11-04-theme-nuit.md) : fond noir, une couleur d'accent, police Inter, et son pendant clair ;
  - [application installable](decisions/2026-11-04-application-installable.md) : PWA, icônes, aperçus des liens ;
  - [déploiement Supabase](decisions/2026-11-04-deploiement-supabase.md) : pooler, polling adaptatif, version de Rust ;
  - [serveur unique sur Render](decisions/2026-11-04-serveur-unique-render.md) : axum, Docker, l'API et l'interface sur une même adresse ;
  - [mises à jour instantanées](decisions/2026-11-04-mises-a-jour-instantanees.md) : avis poussés par le serveur (SSE), moins d'allers-retours vers la base ;
  - [parties publiques](decisions/2026-11-04-parties-publiques.md) : visibilité dans une colonne, liste des salons ouverts, lien d'invitation ;
  - [code QR](decisions/2026-11-04-code-qr.md) : bibliothèque `qrcode-generator`, dessin en SVG par React ;
  - [limitation de débit](decisions/2026-11-04-limitation-de-debit.md) : fenêtre fixe par adresse IP, en-tête de Cloudflare ;
  - [chat](decisions/2026-11-04-chat.md) : table à part, sans changer la version de la partie, livré par le flux d'avis ;
  - [version sur l'accueil](decisions/2026-11-04-version-sur-accueil.md) : API publique de GitHub appelée par le navigateur, cache de 10 minutes.
- `CHANGELOG.md` : journal des versions, GÉNÉRÉ par git-cliff à chaque release (ne pas l'éditer).
