-- Visibilité d'une partie : une partie publique est proposée à tous sur l'accueil tant qu'elle est au salon
-- (GET /api/public-games) ; une partie privée ne se rejoint qu'avec son code ou son lien d'invitation.
-- Faux par défaut : les parties déjà enregistrées restent privées.
-- La visibilité n'est pas une règle du jeu : elle vit dans une colonne, pas dans l'état `state`.
ALTER TABLE games ADD COLUMN public BOOLEAN NOT NULL DEFAULT false;

-- Index partiel : la liste ne parcourt que les parties publiques, des plus récentes aux plus anciennes.
CREATE INDEX games_public_recent ON games (updated_at DESC) WHERE public;
