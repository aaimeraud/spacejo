-- Messages du chat d'une partie. Ils ne font pas partie de l'état du jeu (`games.state`) : en envoyer un ne change
-- pas `games.version`, donc ne crée aucun conflit avec les coups des joueurs. Effacés avec leur partie.
-- `name` : nom du joueur au moment de l'envoi (les places peuvent bouger au salon).
-- La longueur maximale du texte est vérifiée par l'API (`MAX_CHAT_CHARS`, dans src/handlers.rs).
CREATE TABLE chat_messages (
    id         BIGSERIAL   PRIMARY KEY,
    game_id    BIGINT      NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    player_id  INTEGER     NOT NULL CHECK (player_id >= 0),
    name       TEXT        NOT NULL,
    body       TEXT        NOT NULL CHECK (body <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Lecture des derniers messages d'une partie, dans l'ordre.
CREATE INDEX chat_messages_game ON chat_messages (game_id, id);
