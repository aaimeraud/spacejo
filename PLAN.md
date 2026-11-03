# PLAN — SpaceJo v1

Plan courant de la version 1 jouable. Les hypothèses de règles sont dans `ASSUMPTIONS.md`, les décisions
importantes dans `docs/decisions/`.

## Étapes

1. [x] Hook Git, `scripts/check-commit.sh` et ses tests, squelette de `docs/`.
2. [x] `docs/RULES.md`, ce plan, `docs/CONTRACT.md`.
3. [x] `spacejo-core` : tests d'abord (critères 1 à 8, 13 à 15), puis implémentation.
4. [x] Vue filtrée `view_for` (critère 10) et jetons (critère 11).
5. [x] API, persistance, verrou optimiste, migrations, version du format d'état (critères 9, 12, 16, 17, 18).
6. [x] Interface React (accueil, salon, partie, fin de partie).
7. [x] CI, release (git-cliff), `vercel.json`, `.env.example`, `docker-compose.yml`, README.
8. [x] Relecture critère par critère ; `docs/` décrit le code réel.

## Évolution en cours : vue « table de jeu » en CSS 3D (prototype)

Demandée après la v1 par le propriétaire du dépôt, qui pourra préférer ensuite React Three Fiber
(voir `docs/decisions/2026-11-03-vue-table-css-3d.md`). Touche uniquement `web/` (l'API ne change pas).

1. [x] `web/src/game/seats.ts` : place des joueurs autour de la table (moi en bas, les autres dans l'ordre
   du tour, sens des aiguilles d'une montre), sans nombre de joueurs en dur ; tests.
2. [x] Table inclinée en perspective, sièges avec badge (nom, total, cartes cachées, hôte, tour, a terminé),
   pioche et défausse au centre, carte en main au-dessus.
3. [x] Cartes à deux faces qui se retournent (`rotateY`), apparition d'une carte posée, défausse animée ;
   aucune animation si `prefers-reduced-motion`.
4. [x] Petit écran : table à plat, adversaires côte à côte, cartes et badges compacts.
5. [x] Vérification dans le navigateur à 2 et 4 joueurs ; documentation à jour.

Aucune nouvelle dépendance.

## Structure des dossiers

La documentation Vercel du runtime Rust (lue le 2026-11-03,
https://vercel.com/docs/functions/runtimes/rust) demande un `Cargo.toml` à la racine du projet, un fichier
Rust par fonction dans `api/`, et une entrée `[[bin]]` par fonction. On en déduit :

```
Cargo.toml            espace de travail Cargo + paquet `spacejo-api` (les [[bin]] des fonctions)
api/*.rs              une fonction Vercel par fichier, quelques lignes chacune : elles appellent la bibliothèque
src/                  bibliothèque `spacejo-api` : HTTP, jetons, stockage, migrations du format d'état
src/bin/dev_server.rs serveur local qui regroupe toutes les routes (développement uniquement)
src/bin/migrate.rs    applique les migrations SQL sur la base désignée par DATABASE_URL
crates/spacejo-core/  logique du jeu, sans réseau, sans base, sans horloge
migrations/           migrations SQL versionnées (seul moyen de faire évoluer le schéma)
tests/                tests d'intégration de l'API (+ tests/scripts/ pour les scripts shell)
web/                  interface React + TypeScript + Vite
docs/                 documentation vivante
scripts/              check-commit.sh, release.sh, changelog.sh
.github/workflows/    ci.yml, release.yml
```

Routes (une par fichier de `api/`, donc sans route dynamique, que la documentation ne décrit pas) :
`/api/create`, `/api/join`, `/api/rules`, `/api/start`, `/api/action`, `/api/state`.
Six fonctions, sous la limite de 12 fonctions par déploiement du plan gratuit.

## Base de données et driver

- **PostgreSQL** hébergé (par exemple Neon ou Supabase), dans la même région que les fonctions
  (`iad1` par défaut chez Vercel). En local : PostgreSQL via `docker-compose.yml`.
- **Driver : `sqlx` 0.9** (fonctions `sqlx::query`, sans les macros vérifiées à la compilation, qui
  exigeraient une base pendant la compilation). Raisons : asynchrone sur `tokio` (déjà imposé par
  `vercel_runtime`), TLS intégré (`rustls`, sans OpenSSL à installer, obligatoire pour une base hébergée),
  et un système de migrations versionnées (`sqlx::migrate!`) qui embarque le dossier `migrations/` dans le
  binaire et note les migrations appliquées dans la table `_sqlx_migrations`.
- Alternative écartée : `tokio-postgres` + `refinery` + une caisse TLS : trois dépendances au lieu
  d'une, et TLS à câbler à la main.
- Une connexion par requête (pas de pool en mémoire : fonctions sans état). En production, utiliser
  l'URL « poolée » du fournisseur (PgBouncer) pour ne pas épuiser les connexions.

## Dépendances (justification)

| Caisse | Où | Pourquoi |
|---|---|---|
| `serde` (+ `derive`) | core, api | sérialiser l'état en JSON (JSONB) et les vues |
| `serde_json` | core (tests), api | lire et écrire le JSON |
| `vercel_runtime` 2 | api | imposé par le runtime Rust de Vercel |
| `tokio` | api | moteur asynchrone exigé par `vercel_runtime` et `sqlx` |
| `http-body-util` | api | lire le corps d'une requête avec une taille maximale (déjà tiré par `vercel_runtime`) |
| `sqlx` 0.9 | api | driver PostgreSQL + migrations (voir ci-dessus) |
| `getrandom` | api | entropie du système : jetons, codes de partie, graines |
| `sha2` | api | hachage SHA-256 des jetons (seul le hachage est stocké) |
| `react`, `react-dom` | web | imposé |
| `typescript`, `vite`, `@vitejs/plugin-react` | web | imposé (Vite + TypeScript strict) |
| `vitest` | web | tests du front, même configuration que Vite |

Pas de caisse `rand` dans le core : le mélange utilise un petit générateur SplitMix64 écrit dans le
core (voir `docs/decisions/2026-11-03-generateur-aleatoire.md`), pour un résultat identique quelle que
soit la version des dépendances.

Outils (hors dépendances du code) : `git-cliff` pour le changelog et les versions (justifié dans
`docs/decisions/2026-11-03-releases-git-cliff.md`).

## Risques

- **Runtime Rust de Vercel en bêta** : la documentation ne parle ni d'espace de travail Cargo ni de
  routes dynamiques. Le paquet racine contient les `[[bin]]` comme dans l'exemple officiel ; le core est
  un membre de l'espace de travail. Non vérifié par un déploiement (interdit ici) : à confirmer au premier
  déploiement.
- **Connexions PostgreSQL** : une connexion par requête coûte une poignée de main TLS ; acceptable
  avec un polling toutes les 2 s et 4 joueurs, à revoir au-delà.
- **Polling** : charge proportionnelle au nombre de joueurs connectés ; la réponse est minimale quand
  rien n'a changé.
- **GitButler** ne lance pas le hook `commit-msg` : le contrôle est lancé à la main avant chaque commit
  et la CI le refait (voir `docs/decisions/2026-11-03-controle-des-commits.md`).

## Licence (au choix du propriétaire du dépôt)

Aucun fichier `LICENSE` n'est ajouté. Options possibles : MIT (permissive, courte), Apache-2.0
(permissive, avec clause de brevets), AGPL-3.0 (oblige à publier les modifications d'un service en ligne),
ou rien (tous droits réservés, ce qui est le cas actuel).
