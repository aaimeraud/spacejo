# Déploiement : Supabase pour PostgreSQL

> L'hébergement de l'API est passé de Vercel à Render le même jour : voir
> [un seul serveur Rust, hébergé sur Render](2026-11-04-serveur-unique-render.md). Ce qui suit sur le pooler,
> le polling et la version de Rust reste valable ; le quota d'appels de Vercel ne s'applique plus.

## Contexte

Le propriétaire du dépôt déploie sur Vercel (offre gratuite Hobby au départ) avec une base PostgreSQL chez
Supabase. Documentation lue le 2026-11-04 : https://supabase.com/docs/guides/database/connecting-to-postgres,
https://vercel.com/docs/functions/runtimes/rust, https://vercel.com/docs/plans/hobby.

Deux contraintes en ressortent :

- **Connexions** : sur l'offre gratuite, la connexion directe à Supabase n'existe qu'en IPv6, que Vercel ne
  sait pas joindre. Il faut passer par le « pooler » partagé (IPv4), qui met les connexions en commun. Supabase
  recommande son mode transaction (port 6543) pour les fonctions serverless, mais « The shared pooler doesn't
  support prepared statements in transaction mode ».
- **Coût du polling** : l'offre Hobby inclut 1 000 000 d'appels de fonctions par mois ; un joueur qui interroge
  le serveur toutes les 2 s en fait 1 800 par heure de jeu. Au-delà du quota, Vercel bloque la fonctionnalité
  jusqu'à 30 jours.

## Décision

- *Remplacé depuis par le mode session et le cache réactivé (voir
  [mises à jour instantanées](2026-11-04-mises-a-jour-instantanees.md)) :* **sqlx sans cache de requêtes
  préparées** (`statement_cache_capacity(0)` dans `PgStore::connect_lazy`) :
  sqlx n'utilise alors que des requêtes préparées sans nom, valables le temps d'une requête, ce que le mode
  transaction accepte. `DATABASE_URL` en production = adresse du pooler en mode transaction (port 6543).
  Les migrations, elles, se lancent depuis une machine avec l'adresse en mode session (port 5432).
- **Polling adaptatif** (`web/src/polling.ts`) : 2 s pendant une partie, 3 s au salon, 10 s quand l'onglet est
  caché ou la partie terminée, et une lecture immédiate quand on revient sur l'onglet. 10 s reste sous le délai
  d'absence du serveur (15 s) : changer d'onglet ne rend pas absent.
- **Région** : l'offre gratuite de Supabase ne propose que « Europe », c'est-à-dire l'Irlande (AWS
  `eu-west-1`). Le serveur doit en être proche : chaque requête du jeu fait environ trois allers-retours vers la
  base (sur Render : Francfort, la région la plus proche).

- **Version de Rust épinglée** (`rust-toolchain.toml`, 1.99.1 depuis la mise à jour du 2026-11-04) : la machine de build de Vercel fournissait Rust
  1.92, alors que sqlx 0.9 en demande 1.94 (`rust-version` du `Cargo.toml` corrigé en conséquence). rustup lit
  ce fichier et installe la bonne version, en local, dans la CI et sur Vercel si son build passe par rustup. À
  défaut, il faudrait revenir à sqlx 0.8.

- **Profil « release » allégé** (`Cargo.toml`) : sans `lto = "fat"` ni `codegen-units = 1`, la compilation
  complète passe de 1 min 32 à 53 s sur un ordinateur portable, et bien davantage sur la machine de build de
  Vercel (2 cœurs), où le premier build dépassait 4 minutes. Les fonctions attendent surtout la base de données :
  le gain de vitesse de ces réglages ne se verrait pas.

## Alternatives écartées

- **Pooler en mode session** (port 5432) en production : marche sans rien changer, mais garde une connexion
  réservée par client ; avec beaucoup d'instances de fonctions, on atteint vite la limite du pooler.
- **Supabase Realtime à la place du polling** : le serveur préviendrait les joueurs à chaque changement, au lieu
  d'être interrogé ; bien plus économe, mais c'est un chantier à part (voir `IDEAS.md`).

## Conséquences

- Un peu plus de travail par requête côté PostgreSQL (la requête est analysée à chaque fois) : négligeable pour
  nos requêtes, toutes très courtes.
- À l'échelle de quelques dizaines de joueurs, l'offre Hobby suffit si les parties restent occasionnelles ;
  au-delà, ou pour un usage commercial (interdit sur Hobby), il faut l'offre Pro.
