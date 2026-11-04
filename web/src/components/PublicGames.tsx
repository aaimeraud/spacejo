// Liste des parties publiques au salon, sur l'accueil. Elle se rafraîchit seule tant qu'elle est affichée ;
// choisir une partie mène à l'étape du nom, code déjà rempli.
import { RefreshCw, Users } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import type { PublicGame } from "../api/types";
import { PUBLIC_GAMES_REFRESH_MS } from "../config";

interface Props {
  load: () => Promise<PublicGame[]>;
  onPick: (code: string) => void;
}

export function PublicGames({ load, onPick }: Props) {
  // `null` : première lecture en cours.
  const [games, setGames] = useState<PublicGame[] | null>(null);
  const [failed, setFailed] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setGames(await load());
      setFailed(false);
    } catch {
      setFailed(true);
    }
  }, [load]);

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), PUBLIC_GAMES_REFRESH_MS);
    return () => window.clearInterval(timer);
  }, [refresh]);

  return (
    <div className="public-games">
      <div className="public-games-head">
        <p className="note" role="status">
          {failed
            ? "Impossible de charger la liste. Nouvel essai dans quelques secondes."
            : games === null
              ? "Chargement des parties…"
              : games.length === 0
                ? "Aucune partie publique en attente pour le moment. Créez-en une !"
                : "Parties en attente de joueurs, les plus récentes d'abord."}
        </p>
        <button type="button" className="ghost-button" onClick={() => void refresh()}>
          <RefreshCw aria-hidden="true" /> Actualiser
        </button>
      </div>
      {games !== null && games.length > 0 && (
        <ul className="public-games-list">
          {games.map((game) => (
            <li key={game.code}>
              <button type="button" className="public-game" onClick={() => onPick(game.code)}>
                <span className="badge-avatar" aria-hidden="true">
                  {game.host.slice(0, 1).toUpperCase()}
                </span>
                <span className="public-game-host">Partie de {game.host}</span>
                <span className="public-game-count">
                  <Users aria-hidden="true" /> {game.players} / {game.max_players}
                  <span className="visually-hidden"> joueurs</span>
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
