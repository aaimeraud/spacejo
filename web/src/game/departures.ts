// Messages de départ montrés à tous : comparer la vue précédente et la nouvelle suffit (le serveur dit qui a
// abandonné ; au salon, un joueur qui part disparaît simplement de la liste).
import type { GameView } from "../api/types";

/** Messages à afficher entre deux vues : « X a abandonné la partie » pendant une partie, « X a quitté le salon »
 * au salon (qu'il soit parti ou que l'hôte l'ait retiré). Rien pour soi-même, ni à la première vue. */
export function departureNotices(previous: GameView | null, next: GameView): string[] {
  if (previous === null) return [];
  if (previous.phase === "Lobby" && next.phase === "Lobby") {
    // Au salon, les places se décalent : on compare les noms (deux joueurs peuvent porter le même nom, d'où le
    // décompte plutôt qu'un simple ensemble).
    const remaining = new Map<string, number>();
    for (const player of next.players) remaining.set(player.name, (remaining.get(player.name) ?? 0) + 1);
    const notices: string[] = [];
    for (const player of previous.players) {
      const count = remaining.get(player.name) ?? 0;
      if (count > 0) remaining.set(player.name, count - 1);
      else if (player.id !== previous.me) notices.push(`${player.name} a quitté le salon.`);
    }
    return notices;
  }
  // Pendant la partie, les places ne bougent plus : on compare joueur par joueur.
  return next.players
    .filter((player) => player.abandoned && player.id !== next.me && previous.players[player.id]?.abandoned === false)
    .map((player) => `${player.name} a abandonné la partie.`);
}
