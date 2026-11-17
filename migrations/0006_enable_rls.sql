-- Active la Row Level Security (RLS) sur les tables de la base.
-- Pourquoi : Supabase expose le schéma `public` à l'API automatique (rôles `anon` et `authenticated`).
-- Sans RLS, n'importe qui disposant de la clé publique pourrait lire `games.state`, qui contient les cartes cachées.
-- Documentation : https://supabase.com/docs/guides/database/postgres/row-level-security
--
-- Aucune politique n'est créée : avec la RLS activée et zéro politique, les rôles `anon` et `authenticated`
-- ne voient et ne modifient rien. Le serveur Rust n'est pas concerné : il se connecte avec le rôle propriétaire
-- des tables (`postgres.<ref>`), qui contourne la RLS tant qu'elle n'est pas forcée (`FORCE ROW LEVEL SECURITY`).
-- Les accès passent donc uniquement par l'API Rust, qui applique les règles du jeu.
--
-- `_sqlx_migrations` (table de suivi des migrations) est aussi verrouillée : elle n'a pas à être exposée.

ALTER TABLE games ENABLE ROW LEVEL SECURITY;
ALTER TABLE presence ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_reactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE _sqlx_migrations ENABLE ROW LEVEL SECURITY;
