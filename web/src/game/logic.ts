// Aide à l'interface : quelle action envoie un clic, et quelle consigne afficher.
// Ces fonctions ne calculent AUCUN score et ne décident d'aucune règle : le serveur valide tout
// et renvoie une erreur claire si l'action est refusée.
import type { Action, GameView, PlayerView, PublicSlot } from "../api/types";

/** Ce que fait un clic sur une carte de sa grille, quand on a une carte de la pioche en main. */
export type HeldMode = "swap" | "reveal";

export function slotAt(
  player: PlayerView,
  column: number,
  row: number,
): PublicSlot | undefined {
  const col = player.grid[column];
  if (col === undefined || col.kind === "Removed") return undefined;
  return col.slots[row];
}

export function isMyTurn(view: GameView): boolean {
  return view.phase === "Playing" && view.current_player === view.me;
}

export function playerName(view: GameView, id: number | null): string {
  if (id === null) return "";
  return view.players[id]?.name ?? `Joueur ${id + 1}`;
}

/** Action envoyée par un clic sur la case (colonne, ligne) de SA grille, ou `null` si ce clic ne
 * correspond à rien dans la situation actuelle. */
export function slotAction(
  view: GameView,
  mode: HeldMode,
  column: number,
  row: number,
): Action | null {
  const me = view.players[view.me];
  if (me === undefined) return null;
  const slot = slotAt(me, column, row);
  if (slot === undefined) return null;
  if (view.phase === "InitialReveal") {
    if (me.initial_reveals >= 2 || slot.kind !== "Hidden") return null;
    return { type: "RevealInitial", column, row };
  }
  if (!isMyTurn(view) || view.held === null) return null;
  if (view.held.source === "Pile" && mode === "reveal") {
    return slot.kind === "Hidden"
      ? { type: "DiscardHeldAndReveal", column, row }
      : null;
  }
  return { type: "SwapHeld", column, row };
}

/** Consigne affichée au joueur. */
export function hint(view: GameView, mode: HeldMode): string {
  const me = view.players[view.me];
  switch (view.phase) {
    case "Lobby":
      return view.me === view.host
        ? "Attendez les autres joueurs, puis lancez la partie."
        : "En attente du lancement par l'hôte.";
    case "InitialReveal": {
      const left = 2 - (me?.initial_reveals ?? 0);
      return left > 0
        ? `Retournez ${left} carte${left > 1 ? "s" : ""} de votre grille.`
        : "En attente des autres joueurs.";
    }
    case "Playing": {
      const last = view.finisher !== null ? " (dernier tour)" : "";
      if (!isMyTurn(view))
        return `C'est au tour de ${playerName(view, view.current_player)}${last}.`;
      if (view.held === null) {
        return `À vous de jouer${last} : piochez une carte ou prenez la carte de la défausse.`;
      }
      if (view.held.source === "Discard") {
        return "Choisissez la carte de votre grille à échanger avec la carte prise sur la défausse.";
      }
      return mode === "swap"
        ? "Choisissez la carte de votre grille à remplacer par la carte piochée."
        : "Choisissez une carte cachée à retourner (la carte piochée va sur la défausse).";
    }
    case "RoundOver":
      return roundOverHint(view);
    case "GameOver":
      return "Partie terminée.";
  }
}

/** Ma place dans le vote de fin de manche :
 * - `decider` : je lance la manche suivante (l'hôte, ou le premier joueur encore en jeu s'il est parti) ;
 * - `voted` : j'ai voté, j'attends que la manche soit lancée ;
 * - `can-vote` : je n'ai pas encore voté (sans vote, je ne serai pas de la manche suivante) ;
 * - `out` : je ne joue plus (éliminé·e ou parti·e). */
export type NextRoundRole = "decider" | "voted" | "can-vote" | "out";

export interface NextRoundStatus {
  role: NextRoundRole;
  /** Nombre de joueurs qui ont voté (sans celui qui décide). */
  votes: number;
  /** Nombre de joueurs qui peuvent voter : joueurs encore en jeu, sans celui qui décide. */
  voters: number;
}

/** Où en est le vote pour la manche suivante (lecture de la vue : c'est le serveur qui compte les votes). */
export function nextRoundStatus(view: GameView): NextRoundStatus {
  const active = view.players.filter((player) => !player.eliminated);
  const voters = active.filter((player) => player.id !== view.next_round_decider).length;
  const votes = view.next_round_votes.length;
  const me = view.players[view.me];
  let role: NextRoundRole;
  if (me === undefined || me.eliminated) role = "out";
  else if (view.next_round_decider === view.me) role = "decider";
  else role = view.next_round_votes.includes(view.me) ? "voted" : "can-vote";
  return { role, votes, voters };
}

function roundOverHint(view: GameView): string {
  switch (nextRoundStatus(view).role) {
    case "decider":
      return "Manche terminée. Lancez la manche suivante quand les joueurs ont voté.";
    case "voted":
      return `Vote enregistré : en attente de ${playerName(view, view.next_round_decider)}.`;
    case "can-vote":
      return "Manche terminée. Votez pour jouer la manche suivante.";
    case "out":
      return "Manche terminée.";
  }
}

/** Classement final, du plus petit total au plus grand (simple tri des totaux reçus du serveur). */
export function ranking(view: GameView): PlayerView[] {
  return [...view.players].sort(
    (a, b) => a.total_score - b.total_score || a.id - b.id,
  );
}

/** Texte accessible d'une carte. */
export function slotLabel(
  slot: PublicSlot,
  column: number,
  row: number,
): string {
  const where = `colonne ${column + 1}, ligne ${row + 1}`;
  return slot.kind === "Hidden"
    ? `Carte cachée, ${where}`
    : `Carte ${slot.value}, ${where}`;
}
