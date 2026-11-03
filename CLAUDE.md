# CLAUDE.md — SpaceJo

## Contexte
Projet de cours (développement agentique) : SpaceJo, jeu de cartes en ligne basé sur les règles de Skyjo, multijoueur
au tour par tour (2 à 4 joueurs), déployée sur Vercel. Interface en français.
Le développeur est débutant en Rust et veut comprendre le code : préfère toujours le code
simple, lisible et commenté au code « malin ».

## Documents de référence
- `docs/RULES.md` : règles du jeu, SOURCE DE VÉRITÉ. N'invente jamais une règle. Cas non couvert
  → écris ton hypothèse dans `ASSUMPTIONS.md`, puis continue.
- `docs/CONTRACT.md` : formats exacts des requêtes et réponses de l'API. Front et back s'y conforment.
- `docs/ARCHITECTURE.md`, `docs/decisions/`, `docs/CHANGELOG.md` (généré), `docs/CONCEPTS.md` : voir la section
  « Documentation vivante » ci-dessous.
- `PLAN.md` : plan courant. `ASSUMPTIONS.md` : hypothèses prises. `BLOCKERS.md` : points bloquants.
  `IDEAS.md` : idées hors périmètre.

## Pile technique (imposée)
- **Logique du jeu** : crate Rust `spacejo-core`, sans AUCUNE dépendance au réseau, à la base de données
  ni à l'horloge. Fonction centrale : `apply(&mut Game, PlayerId, Action) -> Result<(), GameError>`.
- **API** : fonctions Rust sur Vercel (runtime Rust, en bêta), un fichier par fonction dans `api/`.
  Doc : https://vercel.com/docs/functions/runtimes/rust
- **Base de données** : PostgreSQL hébergé, dans la même région que les fonctions.
  Table `games(id, code, state JSONB, version INT, updated_at)`. Le schéma évolue uniquement par des
  migrations versionnées dans `migrations/`, jamais à la main. PostgreSQL local via `docker-compose.yml`.
  Le JSON `state` contient un champ `state_version` : toute évolution du format de `Game` fournit une
  migration des parties déjà stockées (ou un refus clair), sinon une mise à jour casserait les parties en cours.
- **Interface** : React + TypeScript (mode strict) + Vite, dans `web/`.
  Mise à jour de l'état par polling toutes les 2 secondes.
- Toute dépendance non prévue ici doit être justifiée dans `PLAN.md` avant d'être ajoutée.

## Architecture : règles non négociables
1. **Serveur autoritaire** : le navigateur envoie des intentions (`Action`), jamais un état.
2. **Information cachée** : la valeur d'une carte face cachée ne doit JAMAIS figurer dans une réponse
   d'API. Toute réponse passe par `view_for(game, player_id)`, qui produit une vue filtrée
   (`PublicSlot`, sans valeur pour les cartes cachées). Attention : une carte remplacée est posée face
   visible sur la défausse, sa valeur devient alors publique.
3. **Pas d'état en mémoire entre deux requêtes** (serverless) : chaque action charge la partie en base,
   applique `apply`, puis sauvegarde.
4. **Verrou optimiste** : la sauvegarde ne réussit que si `version` n'a pas changé depuis la lecture
   (`UPDATE ... WHERE id = $1 AND version = $2`). Sinon : erreur de conflit, rien n'est écrit.
5. **Identité** : jeton secret aléatoire (au moins 128 bits) remis une seule fois à la création ou à la
   jonction, puis envoyé dans l'en-tête `X-Player-Token`. Côté serveur, seul son hachage est stocké.
   Toute action sans jeton valide, ou hors du tour du joueur, est refusée.
6. **Règles maison** : les réglages vivent dans `Rules`, stocké dans l'état de la partie. Seul l'hôte
   peut les modifier, et UNIQUEMENT avant le lancement. Le calcul des scores lit `Rules` ; aucune règle
   de score en dur ailleurs. Valeurs par défaut = règles officielles. Toute règle que la notice officielle
   ne précise pas (ou sur laquelle les sources divergent) est un réglage de `Rules`, jamais une valeur en
   dur. Les réglages sont choisis par l'hôte à la création de la partie ; `docs/RULES.md` indique le statut
   de chaque règle (officiel ou option) et ses sources.
7. **Validation** : toute donnée venant du navigateur (JSON, code de partie, réglages) est validée
   côté serveur ; valeur inconnue = refus.
8. Le mélange du paquet est fait par le serveur, avec un générateur initialisable par une graine (seed),
   pour que les tests soient déterministes. En production, la graine vient de l'entropie du système et
   n'est jamais exposée.
9. **Version attendue** : toute action et toute modification de réglages envoie l'`expected_version` que
   le client connaît ; si elle ne correspond pas à la version en base, la requête est refusée (409) et
   n'écrit rien. Cela protège aussi contre une requête rejouée ou envoyée depuis un écran périmé.
10. **Hygiène des journaux** : ne jamais journaliser un jeton, un hachage de jeton, une valeur de carte
    cachée, la graine ni l'état complet d'une partie.
11. **Pas de limite en dur** : nombre minimum et maximum de joueurs, bornes de `end_score`, taille du code
    de partie, durée du polling vivent dans des constantes nommées, définies une seule fois. `spacejo-core`
    ne suppose pas 4 joueurs (la v1 plafonne à 4 par une constante, pas par la logique).
12. **Code de partie** : au moins 6 caractères d'un alphabet sans ambiguïté (ni 0/O ni 1/I), tirés avec un
    générateur cryptographique, pour qu'il ne soit pas devinable.

## Conventions de code
- Identifiants et messages de commit en anglais (Conventional Commits : `feat:`, `fix:`, `test:`, `docs:`…).
- Commentaires en français, qui expliquent le POURQUOI (surtout là où une règle du jeu est codée).
- Rust : `cargo fmt` et `cargo clippy -- -D warnings`. Pas de `unwrap()` ni `expect()` hors tests,
  pas de `unsafe`, aucun panic possible sur une entrée utilisateur : utilise `Result` et `?`.
- TypeScript : pas de `any`. Types de l'API écrits d'après `docs/CONTRACT.md`.
- Accessibilité de base : éléments sémantiques (`button`, pas un `div` cliquable), navigation au clavier,
  information jamais portée par la seule couleur.
- Pédagogie : quand tu utilises une notion Rust non triviale (propriété et emprunt, trait, enum avec
  données, `Result` et `?`, itérateurs), ajoute un commentaire de 1 à 3 lignes avec le lien du chapitre
  du livre https://doc.rust-lang.org/stable/book/ et une ligne dans `docs/CONCEPTS.md`
  (notion, fichier où elle apparaît).

## Documentation vivante (`docs/`)
La documentation vit dans `docs/` et se met à jour AVEC le code, dans le même commit.
- `docs/README.md` : sommaire de la documentation, avec une phrase par page.
- `docs/ARCHITECTURE.md` : vue d'ensemble pour quelqu'un qui découvre le projet. Schéma en texte des
  composants, parcours d'une action de bout en bout (clic → API → base → réponse), modèle de données,
  rôle de chaque dossier, flux de la vue filtrée (cartes cachées) et du verrou optimiste.
- `docs/CONTRACT.md` (API), `docs/RULES.md` (règles du jeu), `docs/CONCEPTS.md` (notions Rust rencontrées).
- `docs/decisions/` : un fichier par décision importante, nommé `AAAA-MM-JJ-titre.md` (contexte, décision,
  alternatives écartées, conséquences). Un fichier par décision évite les conflits quand plusieurs personnes
  travaillent en parallèle.
- `docs/CHANGELOG.md` : GÉNÉRÉ par git-cliff (`cliff.toml`) à partir des messages de commit, à chaque
  release (`scripts/release.sh`, puis workflow `release.yml`).
  Ne l'édite jamais à la main.

Règles :
1. Avant chaque commit, relis `git diff --staged` et mets à jour toutes les pages de `docs/` que ce
   changement rend inexactes ou incomplètes.
2. Messages de commit au format Conventional Commits : `type(scope): résumé`, en anglais, au présent.
   Types : `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `ci`, `chore`, `style`.
   Scopes : `core`, `api`, `web`, `db`, `docs`, `ci`, `release`. Changement cassant : `!` après le type et un pied
   `BREAKING CHANGE: ...`. Le message est le journal du projet : il doit se lire seul.
3. La documentation décrit l'état ACTUEL du code, pas son historique (l'historique, c'est `git log`, le
   changelog généré et `docs/decisions/`). Une page ne doit jamais contredire le code.
4. Écris pour un lecteur débutant en Rust : définis chaque terme technique à sa première apparition
   et renvoie vers le chapitre utile de la documentation officielle.
5. Un commit de type `feat`, `fix`, `refactor` ou `perf` qui modifie du code source (chemins listés dans
   `scripts/check-commit.sh`) doit aussi modifier un fichier de `docs/`, ou porter en pied de message
   `Docs: n/a — <raison courte>`. Les types `docs`, `test`, `ci`, `chore` et `style` sont exemptés.
   Cette règle est vérifiée deux fois : par le hook local `.githooks/commit-msg` (retour immédiat) et par la
   CI sur chaque commit d'une pull request (contrôle de référence, car un hook local n'est ni versionné
   ni obligatoire). Si les pull requests sont fusionnées par squash, le titre de la PR devient le
   message de commit : la CI le valide avec le même format.
6. Ne contourne jamais ces contrôles (`--no-verify`, affaiblissement des scripts ou de `core.hooksPath`).
   `Docs: n/a` est réservé aux cas où la documentation n'a vraiment rien à dire, et la raison doit être vraie.
   Installation du hook, une fois par clone : `git config core.hooksPath .githooks`.

## Qualité et maintenance
- **CI** : `.github/workflows/ci.yml` lance, à chaque push et pull request, `cargo fmt --check`,
  `cargo clippy -- -D warnings`, les tests Rust, puis le typecheck et les tests du front, plus la vérification des messages de commit et de la règle
  de documentation (`scripts/check-commit.sh`, sur chaque commit de la pull request). Les mêmes
  commandes doivent passer en local avant chaque commit. Aucune fusion sur la branche principale sans CI verte.
- **Dépendances** : fichiers de verrouillage commités (`Cargo.lock`, lockfile npm), versions justifiées,
  `cargo audit` et `npm audit` dans la CI (non bloquants au début).
- **Releases** : versionnage sémantique. Une release = `scripts/release.sh` (version calculée par
  git-cliff d'après les commits, changelog régénéré, commit `chore(release)`, tag annoté), puis le
  tag poussé par le propriétaire du dépôt déclenche `.github/workflows/release.yml`, qui publie la
  release GitHub avec ses notes. Tu ne pousses jamais de tag ni de commit.
- **Dette technique** : jamais silencieuse. Un raccourci assumé va dans `docs/decisions/` avec la
  raison et ce qu'il faudrait faire pour le corriger ; une idée d'évolution va dans `IDEAS.md`.
- **Limites connues** : `docs/ARCHITECTURE.md` a une section « Limites connues et évolutions prévues »
  (polling remplacé par SSE ou service temps réel, expiration des anciennes parties, limitation de débit
  sur création et jonction, comptes utilisateurs, nombre de joueurs). Elle reste à jour.
- **Droits** : Le projet s'appelle SpaceJo. « Skyjo » est une marque de Magilano : ce nom n'apparaît que pour
  désigner les règles d'origine (`docs/RULES.md`, README), jamais dans les noms de crates, dossiers,
  dépôt, domaine ou interface. Le README indique que le projet n'est pas affilié à Magilano,
  et les visuels sont originaux (aucune reproduction des cartes officielles). N'ajoute pas de fichier
  `LICENSE` de ton propre chef : propose les options dans `PLAN.md`, le propriétaire du dépôt choisit.

## Méthode de travail
- Toute tâche touchant plus d'un fichier : écris ou mets à jour `PLAN.md` d'abord.
- `spacejo-core` : écris les tests AVANT le code, puis implémente jusqu'à ce qu'ils passent.
- Avant de dire qu'une tâche est terminée, lance les vérifications (voir Commandes). Si quelque chose
  échoue, dis-le clairement. Ne désactive, ne supprime et n'assouplis jamais un test pour le faire passer.
- Pour toute API récente ou en bêta (runtime Rust de Vercel, etc.) : lis la documentation officielle
  avec WebFetch plutôt que de te fier à ta mémoire.
- Reste dans le périmètre de la demande. Une bonne idée hors périmètre va dans `IDEAS.md`.
- Bloqué après 3 tentatives sur le même problème : note-le dans `BLOCKERS.md` (symptôme, essais,
  hypothèse), puis passe à autre chose.
- Petits commits, fréquents, sur la branche courante, chacun accompagné de la mise à jour de `docs/`.
  Ne fais jamais de push ni de déploiement.
  Ne lis et n'écris aucun secret réel : seulement un fichier `.env.example`.

## Commandes
(À garder à jour.)
- Tests Rust : `cargo test --workspace` (tests PostgreSQL : définir `DATABASE_URL` vers un utilisateur qui
  peut créer des bases, par exemple `postgres://spacejo:spacejo@localhost:5433/spacejo` après
  `docker compose up -d`)
- Lint Rust : `cargo fmt --all --check` et `cargo clippy --workspace --all-targets -- -D warnings`
- Lancer l'API en local : `cargo run --bin dev_server` (port 3000 ; en mémoire sans `DATABASE_URL`) ;
  migrations seules : `cargo run --bin migrate`
- Interface (installer / lancer / typecheck / tests) : `npm --prefix web install` /
  `npm --prefix web run dev` / `npm --prefix web run typecheck` / `npm --prefix web test`
- Scripts : `tests/scripts/check-commit.test.sh`, `tests/scripts/release.test.sh`,
  `scripts/changelog.sh`, `scripts/release.sh --dry-run`
- Commit avec GitButler (`but commit` ne lance pas le hook) : lancer d'abord
  `scripts/check-commit.sh <fichier-message>` avec la liste des fichiers du commit sur l'entrée standard.

## Définition de « terminé »
Tests verts, lint propre, `docs/` complète et cohérente avec le code (en particulier `ARCHITECTURE.md` et
`CONTRACT.md`), README (lancer en local, déployer), `ASSUMPTIONS.md` rempli, et une liste honnête de ce
qui ne marche pas encore.