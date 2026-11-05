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
  // Pendant la partie, les places ne bougent plus : on compare joueur par joueur. Un joueur qui n'a pas voté pour la
  // manche suivante la quitte au lancement (le serveur le marque `abandoned`) : on le dit autrement.
  const message = newRound(previous, next) ? "ne joue pas la manche suivante." : "a abandonné la partie.";
  return next.players
    .filter((player) => player.abandoned && player.id !== next.me && previous.players[player.id]?.abandoned === false)
    .map((player) => `${player.name} ${message}`);
}

/** La manche suivante vient d'être lancée (on passe de la fin de manche à la mise en place). */
function newRound(previous: GameView, next: GameView): boolean {
  return previous.phase === "RoundOver" && next.phase !== "RoundOver";
}

/** La manche suivante a été lancée sans moi (je n'avais pas voté) : l'interface me ramène à l'accueil. */
export function leftBehind(previous: GameView | null, next: GameView): boolean {
  if (previous === null || !newRound(previous, next)) return false;
  return previous.players[previous.me]?.eliminated === false && next.players[next.me]?.eliminated === true;
}
