-- Une ligne par partie. L'état complet (cartes cachées comprises) est dans `state` (JSONB) ;
-- il n'est jamais renvoyé tel quel au navigateur (voir docs/ARCHITECTURE.md, « Vue filtrée »).
-- `version` sert au verrou optimiste : chaque écriture fait `version = version + 1`
-- à condition que `version` n'ait pas changé depuis la lecture.
CREATE TABLE games (
    id         BIGSERIAL   PRIMARY KEY,
    code       TEXT        NOT NULL UNIQUE,
    state      JSONB       NOT NULL,
    version    INTEGER     NOT NULL DEFAULT 1 CHECK (version >= 1),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
