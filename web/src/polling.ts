// Rythme du polling : rapide quand il se passe quelque chose à l'écran, lent sinon, pour ménager le serveur
// et la base (offres gratuites de Render et Supabase). Quand le flux d'avis du serveur est ouvert
// (api/events.ts), les coups arrivent par lui : le polling ralentit. Voir
// docs/decisions/2026-11-04-mises-a-jour-instantanees.md.
import type { GameView } from "./api/types";
import { POLL_IDLE_INTERVAL_MS, POLL_INTERVAL_MS, POLL_LIVE_INTERVAL_MS, POLL_LOBBY_INTERVAL_MS } from "./config";

/** Délai avant la prochaine lecture de l'état. `phase` est `null` tant que la partie n'est pas chargée ;
 * `live` : le flux d'avis du serveur est ouvert. */
export function pollInterval(hidden: boolean, phase: GameView["phase"] | null, live = false): number {
  if (hidden || phase === "GameOver") return POLL_IDLE_INTERVAL_MS;
  if (live) return POLL_LIVE_INTERVAL_MS;
  if (phase === "Lobby") return POLL_LOBBY_INTERVAL_MS;
  return POLL_INTERVAL_MS;
}
