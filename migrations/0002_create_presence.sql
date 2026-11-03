-- Dernière requête reçue de chaque joueur, pour savoir qui est absent (ASSUMPTIONS.md, n° 18).
-- Table séparée de `games` : mettre à jour la présence ne change ni l'état ni la `version` de la partie,
-- donc ne crée aucun conflit avec le verrou optimiste.
CREATE TABLE presence (
    game_id   BIGINT      NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    player_id INTEGER     NOT NULL CHECK (player_id >= 0),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (game_id, player_id)
);
