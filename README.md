# SpaceJo

Jeu de cartes en ligne, au tour par tour, de 2 à 10 joueurs. Il reprend les règles du jeu de cartes
**Skyjo** (voir [docs/RULES.md](docs/RULES.md)), avec des réglages de partie choisis par l'hôte.

> SpaceJo est un projet de cours indépendant. Il n'est **pas affilié à Magilano**, éditeur de Skyjo, ni
> approuvé par lui. « Skyjo » est une marque de Magilano ; le nom ne sert ici qu'à désigner les règles
> d'origine. Les visuels sont originaux : aucune carte officielle n'est reproduite.

- Logique du jeu en Rust (`crates/spacejo-core`), sans réseau ni base de données.
- API en fonctions Rust sur Vercel (`api/`, `src/`), état dans PostgreSQL.
- Interface React + TypeScript + Vite (`web/`), mise à jour par polling toutes les 2 secondes.

Documentation : [docs/README.md](docs/README.md) (architecture, contrat d'API, règles, notions Rust,
décisions). Plan et hypothèses : [PLAN.md](PLAN.md), [ASSUMPTIONS.md](ASSUMPTIONS.md).

## Prérequis

- Rust stable (1.88 ou plus récent) : https://rustup.rs
- Node.js 24 ou plus récent, avec npm
- PostgreSQL : Docker (`docker compose`) ou une installation locale. Facultatif pour jouer en local
  (le serveur sait garder les parties en mémoire), nécessaire pour les tests PostgreSQL.
- git-cliff, pour les releases seulement : https://git-cliff.org/docs/installation

Une fois par clone, activer le contrôle des messages de commit :

```bash
git config core.hooksPath .githooks
```

## Lancer en local

1. Base de données (facultatif) :

   ```bash
   docker compose up -d
   ```

   ```bash
   export DATABASE_URL=postgres://spacejo:spacejo@localhost:5433/spacejo
   ```

   Sans `DATABASE_URL`, le serveur local garde les parties en mémoire (perdues à l'arrêt).

2. API (port 3000 ; applique les migrations au démarrage si `DATABASE_URL` est définie) :

   ```bash
   cargo run --bin dev_server
   ```

3. Interface, dans un autre terminal (http://localhost:5173, qui renvoie `/api` vers le port 3000) :

   ```bash
   npm --prefix web install
   ```

   ```bash
   npm --prefix web run dev
   ```

## Vérifications (les mêmes que la CI)

```bash
cargo fmt --all --check
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
cargo test --workspace
```

Les tests PostgreSQL (`tests/postgres.rs`) ne s'exécutent que si `DATABASE_URL` désigne un utilisateur qui
peut créer des bases (chaque test crée puis supprime sa base `spacejo_test_…`). Sinon ils affichent un
message et s'arrêtent ; la CI, elle, les rend obligatoires avec `SPACEJO_REQUIRE_DB=1`.

```bash
npm --prefix web run typecheck
```

```bash
npm --prefix web test
```

```bash
tests/scripts/check-commit.test.sh
```

```bash
tests/scripts/release.test.sh
```

## Vérification manuelle : une partie à deux dans deux onglets

1. Lancer l'API et l'interface (ci-dessus), puis ouvrir http://localhost:5173 dans **deux onglets**.
   Chaque onglet garde sa propre session (`sessionStorage`).
2. Onglet 1 : « Créer une partie », saisir un nom, choisir « Règles officielles », puis « Créer la partie »
   sur le récapitulatif. Noter le code affiché dans le salon.
3. Onglet 2 : « Rejoindre une partie », saisir le code puis un autre nom. Les deux salons affichent les deux
   joueurs et les règles actives (« Explications » les détaille avec l'étiquette « officiel » ou « variante »).
4. Onglet 1 (hôte) : « Lancer la partie ». Chaque joueur retourne 2 cartes, puis joue à son tour (piocher
   ou prendre la défausse, puis échanger ou défausser et retourner). Si un message « La partie a changé
   entre-temps » s'affiche, l'autre joueur a joué au même moment : refaire le geste.
5. Jouer jusqu'à « Fin de partie » : le classement s'affiche (« Tableau des scores » pour le détail). La page
   ne défile jamais : tout tient à l'écran, en paysage comme en portrait.
6. Recommencer (« Quitter », puis créer une partie) avec au moins une variante, par exemple le préréglage
   « Détente », ou « Personnalisée » avec un score de fin de 30 points au curseur pour une partie courte. Vérifier
   que l'onglet 2 voit le changement de réglage dans le salon, et qu'il ne peut pas les modifier.
7. Absence : pendant une partie, fermer l'onglet 2. Dans l'onglet 1, son badge passe à « absent » après 15 s ; à
   son tour, un bandeau compte 45 s, puis son tour est passé ; au deuxième tour manqué, il est éliminé. Rouvrir le
   site : l'accueil propose « Reprendre une partie ».

## Déployer sur Vercel

Le déploiement n'a pas encore été essayé (le runtime Rust de Vercel est en bêta : voir
[docs/decisions/2026-11-03-structure-vercel.md](docs/decisions/2026-11-03-structure-vercel.md)).

1. Créer une base PostgreSQL hébergée (Neon, Supabase…) **dans la même région** que les fonctions :
   `vercel.json` fixe la région `cdg1` (Paris), donc une base à Paris (par exemple AWS `eu-west-3`).
2. Appliquer les migrations sur cette base, depuis sa machine :

   ```bash
   DATABASE_URL="<url de la base>" cargo run --bin migrate
   ```

3. Importer le dépôt dans Vercel, puis définir dans Settings → Environment Variables : `DATABASE_URL` (l'URL
   « poolée » du fournisseur, avec `sslmode=require`) et `VITE_SITE_URL` (l'adresse publique du site, sans
   « / » final, pour les aperçus des liens partagés).
4. Déployer. Vercel compile une fonction par fichier de `api/` (déclarés en `[[bin]]` dans `Cargo.toml`)
   et construit l'interface avec la commande de `vercel.json` (`web/dist`).
5. À chaque nouvelle migration, relancer l'étape 2 avant de déployer.

## Releases

Versionnage sémantique, changelog généré par git-cliff ([cliff.toml](cliff.toml)) dans
`docs/CHANGELOG.md`, à ne jamais éditer à la main.

```bash
scripts/changelog.sh
```

affiche les changements non publiés.

```bash
scripts/release.sh --dry-run
```

affiche la version suivante et l'aperçu du changelog sans rien modifier. Sans `--dry-run` (depuis `main`,
arbre propre), le script met à jour les versions, régénère le changelog, crée le commit
`chore(release): vX.Y.Z` et le tag annoté, puis affiche les commandes `git push` à lancer. La première
release se donne en argument :

```bash
scripts/release.sh 0.1.0
```

Le tag poussé déclenche [.github/workflows/release.yml](.github/workflows/release.yml), qui publie la
release GitHub avec les notes de cette version.

## Ce qui ne marche pas encore

Voir aussi « Limites connues » dans [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

- Jamais déployé sur Vercel : la structure suit la documentation, sans vérification réelle.
- Pendant la mise en place, les retournements simultanés provoquent souvent un conflit de version (409) :
  le joueur doit refaire son geste.
- Pas d'expiration des anciennes parties, ni de limitation de débit sur la création et la jonction.

## Licence

Aucune licence n'a encore été choisie (tous droits réservés par défaut). Les options sont listées dans
[PLAN.md](PLAN.md) ; le choix revient au propriétaire du dépôt.
