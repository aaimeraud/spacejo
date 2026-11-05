-- Réactions aux messages du chat (cœur, pouce levé…). Une seule par joueur et par message : en choisir une autre la
-- remplace (clé primaire `message_id, player_id`). Effacées avec leur message, donc avec leur partie.
-- `reaction` : nom d'une réaction de la liste fermée `CHAT_REACTIONS` (src/handlers.rs), vérifié par l'API.
CREATE TABLE chat_reactions (
    message_id BIGINT      NOT NULL REFERENCES chat_messages (id) ON DELETE CASCADE,
    player_id  INTEGER     NOT NULL CHECK (player_id >= 0),
    reaction   TEXT        NOT NULL CHECK (reaction <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (message_id, player_id)
);
