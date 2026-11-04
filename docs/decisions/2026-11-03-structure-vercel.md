# Structure du projet pour le runtime Rust de Vercel

> **Remplacée** le 2026-11-04 par [un seul serveur Rust, hébergé sur Render](2026-11-04-serveur-unique-render.md).

## Contexte

La documentation du runtime Rust de Vercel (bêta, lue le 2026-11-03 :
https://vercel.com/docs/functions/runtimes/rust) demande un `Cargo.toml` à la racine, un fichier Rust
par fonction dans `api/`, et une entrée `[[bin]]` par fonction. Elle ne parle ni d'espace de travail Cargo
ni de routes dynamiques (`api/games/[code].rs`). La logique du jeu doit rester isolée dans `spacejo-core`.

## Décision

- Le `Cargo.toml` racine est à la fois un espace de travail (membre : `crates/spacejo-core`) et le paquet
  `spacejo-api`, qui déclare un `[[bin]]` par fichier de `api/`.
- Les fichiers de `api/` ne font que trois lignes utiles : `vercel::serve(Endpoint::X)`. Toute la logique
  HTTP est dans la bibliothèque `src/`, testable sans Vercel (`handlers::handle`).
- Six routes statiques (`/api/create`, `join`, `rules`, `start`, `action`, `state`) ; le code de partie
  passe dans le corps JSON ou dans `?code=` pour `state`. Pas de route dynamique.
- `vercel_runtime::run` démarre un serveur HTTP local (port `VERCEL_DEV_PORT`) : `src/bin/dev_server.rs`
  s'en sert pour servir toutes les routes dans un seul processus en développement, sans la CLI Vercel.

## Alternatives écartées

- **Une seule fonction avec un routeur** (`api/index.rs` + réécritures `vercel.json`) : moins de fonctions,
  mais s'écarte de l'exemple officiel, et dépend du comportement des réécritures en bêta.
- **Routes dynamiques** : non documentées pour Rust.
- **`vercel dev` pour le développement** : exige la CLI et un compte Vercel ; le serveur local suffit.

## Conséquences

- Six binaires sont compilés au déploiement (temps de build plus long, chacun embarque le core).
- Risque : la prise en charge de l'espace de travail par Vercel n'est pas vérifiée (aucun déploiement
  n'a été fait). Si elle pose problème, déplacer le core dans un module de `src/` reste possible sans
  toucher à sa logique.
