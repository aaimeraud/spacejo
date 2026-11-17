# Row Level Security activée sur les tables de Supabase

## Contexte

Le projet Supabase « spacejo » expose le schéma `public` à son API automatique (PostgREST), avec les rôles
`anon` (clé publique) et `authenticated`. Sans RLS, ces rôles pouvaient lire et écrire les tables `games`
(dont `state`, qui contient les cartes cachées), `presence`, `chat_messages` et `chat_reactions`, et
`_sqlx_migrations` (table de suivi des migrations). Constat fait le 2026-11-17 : RLS désactivée sur les 5 tables.

Documentation : https://supabase.com/docs/guides/database/postgres/row-level-security

## Décision

Migration `migrations/0006_enable_rls.sql` : `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` sur les 4 tables
de jeu et sur `_sqlx_migrations`, **sans aucune politique**. Avec zéro politique, `anon` et `authenticated`
ne voient plus rien. Le serveur Rust, lui, se connecte avec le rôle propriétaire des tables
(`postgres.<ref>` dans `DATABASE_URL`), qui contourne la RLS tant qu'elle n'est pas forcée. Il continue donc
de fonctionner sans changement.

## Alternatives écartées

- **Créer des politiques pour `anon`** : inutile, le navigateur ne parle jamais directement à la base ; toute
  lecture passe par `view_for`, côté serveur. Des politiques ouvriraient une voie de contournement de la vue filtrée.
- **Retirer les droits `anon`/`authenticated` (`REVOKE`)** : efficace, mais la RLS reste la protection attendue
  par l'advisor Supabase et couvre aussi les droits accordés plus tard par erreur.
- **`FORCE ROW LEVEL SECURITY`** : appliquerait la RLS au propriétaire, donc au serveur Rust, qui serait bloqué.

## Conséquences

- Toute nouvelle table dans `public` doit aussi recevoir `ENABLE ROW LEVEL SECURITY` dans sa migration.
- Le serveur doit rester connecté avec le rôle propriétaire. Si `DATABASE_URL` passait à un rôle qui n'est pas
  propriétaire et n'a pas `BYPASSRLS`, l'API ne lirait plus rien.
- L'advisor Supabase signale désormais `rls_enabled_no_policy` (niveau INFO) : c'est voulu, pas une erreur.
