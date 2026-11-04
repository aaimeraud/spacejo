// Rythme du polling : rapide quand il se passe quelque chose à l'écran, lent sinon, pour ménager le serveur
// et la base (offres gratuites de Render et Supabase). Voir docs/decisions/2026-11-04-deploiement-supabase.md.
import type { GameView } from "./api/types";
import { POLL_IDLE_INTERVAL_MS, POLL_INTERVAL_MS, POLL_LOBBY_INTERVAL_MS } from "./config";

/** Délai avant la prochaine lecture de l'état. `phase` est `null` tant que la partie n'est pas chargée. */
export function pollInterval(hidden: boolean, phase: GameView["phase"] | null): number {
  if (hidden || phase === "GameOver") return POLL_IDLE_INTERVAL_MS;
  if (phase === "Lobby") return POLL_LOBBY_INTERVAL_MS;
  return POLL_INTERVAL_MS;
}
