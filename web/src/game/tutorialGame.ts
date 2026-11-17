// Moteur du tutoriel : une petite manche fixe entre vous et Aatish (un ami automatisé). Il applique les coups un par
// un et refuse ceux qui ne sont pas permis par les règles de docs/RULES.md (échange, défausse puis retournement,
// colonne identique). Il ne choisit rien à la place du joueur et ne fait aucun accès réseau : il est pur, donc testable.
import type { PublicSlot } from "../api/types";

export type Who = "me" | "bot";

/** Une case : sa valeur réelle et si elle est face visible. La valeur d'une carte cachée ne doit jamais être
 * affichée : l'écran passe par `slotOf`, qui la masque (comme `view_for` côté serveur). */
export interface Cell {
  value: number;
  visible: boolean;
}

/** Colonne de 3 cases, ou `[]` une fois retirée (colonne identique). */
export type Column = Cell[];
export type Grid = Column[];

/** Un geste. Les coordonnées sont (colonne, ligne) dans la grille de celui qui joue, à partir de 0. */
export type TutorialAction =
  | { type: "reveal"; column: number; row: number }
  | { type: "draw" }
  | { type: "takeDiscard" }
  | { type: "swap"; column: number; row: number }
  | { type: "discardAndReveal"; column: number; row: number };

export interface Held {
  value: number;
  /** D'où vient la carte : celle de la pioche peut être défaussée, celle de la défausse doit être échangée. */
  from: "pile" | "discard";
}

export interface TutorialState {
  grids: Record<Who, Grid>;
  /** Retournements restants de la mise en place (2 par joueur, docs/RULES.md). */
  setupLeft: Record<Who, number>;
  /** Pioche : le premier élément est la carte du dessus. */
  pile: number[];
  /** Défausse : le dernier élément est la carte du dessus. Toutes ses cartes sont face visible. */
  discard: number[];
  held: Held | null;
}

export type ApplyResult = { ok: true; state: TutorialState } | { ok: false; error: string };

/** Nombre de cartes que chaque joueur retourne au début de la manche (docs/RULES.md, « Mise en place »). */
const SETUP_REVEALS = 2;

function toGrid(columns: number[][]): Grid {
  return columns.map((column) => column.map((value) => ({ value, visible: false })));
}

/** La donne de la démonstration. Elle est fixe pour que chaque essai se déroule de la même façon.
 * Les coups de Aatish sont écrits dans `tutorial.ts` ; ici, seule la donne compte. */
export function initialState(): TutorialState {
  return {
    grids: {
      // Colonnes de 3 cartes, de haut en bas. Toutes cachées au départ.
      me: toGrid([
        [10, 4, 2],
        [9, 9, 5],
        [-1, 7, 8],
        [3, 11, 6],
      ]),
      bot: toGrid([
        [2, 5, 8],
        [1, 4, 12],
        [0, 6, 9],
        [10, 3, 7],
      ]),
    },
    setupLeft: { me: SETUP_REVEALS, bot: SETUP_REVEALS },
    pile: [6, 3, 7, 1, 2, 9, 4, 8, 0, 12, -2, 10, 5, 11],
    discard: [12],
    held: null,
  };
}

/** Ce que voit le joueur d'une case : la valeur seulement si elle est face visible. */
export function slotOf(cell: Cell): PublicSlot {
  return cell.visible ? { kind: "Visible", value: cell.value } : { kind: "Hidden" };
}

/** Applique un geste de `who` sans modifier `state` (copie). Renvoie une erreur lisible si le geste est refusé. */
export function applyAction(state: TutorialState, who: Who, action: TutorialAction): ApplyResult {
  const next = structuredClone(state);
  const error = mutate(next, who, action);
  return error === null ? { ok: true, state: next } : { ok: false, error };
}

/** Modifie `s` en place. Renvoie `null` si le geste est permis, sinon le message d'erreur. */
function mutate(s: TutorialState, who: Who, action: TutorialAction): string | null {
  switch (action.type) {
    case "reveal":
      return reveal(s, who, action.column, action.row);
    case "draw":
      return draw(s);
    case "takeDiscard":
      return takeDiscard(s);
    case "swap":
      return swap(s, who, action.column, action.row);
    case "discardAndReveal":
      return discardAndReveal(s, who, action.column, action.row);
  }
}

function reveal(s: TutorialState, who: Who, column: number, row: number): string | null {
  if (s.held !== null) return "Une carte est en main : échangez-la ou défaussez-la d'abord.";
  // Après la mise en place, on ne retourne plus une carte librement : seulement après une défausse.
  if (s.setupLeft[who] <= 0) return "La mise en place est terminée pour ce joueur.";
  const cell = s.grids[who][column]?.[row];
  if (cell === undefined) return "Cette case n'existe pas, ou sa colonne est retirée.";
  if (cell.visible) return "Cette carte est déjà face visible.";
  cell.visible = true;
  s.setupLeft[who] -= 1;
  return null;
}

function draw(s: TutorialState): string | null {
  if (s.held !== null) return "Une carte est déjà en main.";
  const value = s.pile.shift();
  if (value === undefined) return "La pioche est vide.";
  s.held = { value, from: "pile" };
  return null;
}

function takeDiscard(s: TutorialState): string | null {
  if (s.held !== null) return "Une carte est déjà en main.";
  const value = s.discard.pop();
  if (value === undefined) return "La défausse est vide.";
  s.held = { value, from: "discard" };
  return null;
}

function swap(s: TutorialState, who: Who, column: number, row: number): string | null {
  const held = s.held;
  if (held === null) return "Aucune carte en main : piochez ou prenez la défausse d'abord.";
  const cell = s.grids[who][column]?.[row];
  if (cell === undefined) return "Cette case n'existe pas, ou sa colonne est retirée.";
  // La carte remplacée part sur la défausse, face visible : tout le monde la voit.
  s.discard.push(cell.value);
  cell.value = held.value;
  cell.visible = true;
  s.held = null;
  removeIfIdentical(s, who, column);
  return null;
}

function discardAndReveal(s: TutorialState, who: Who, column: number, row: number): string | null {
  const held = s.held;
  if (held === null) return "Aucune carte en main : piochez d'abord.";
  // Règle : on ne peut défausser que la carte piochée. Celle prise sur la défausse doit être échangée.
  if (held.from !== "pile") return "Une carte prise sur la défausse doit être échangée, pas défaussée.";
  const cell = s.grids[who][column]?.[row];
  if (cell === undefined) return "Cette case n'existe pas, ou sa colonne est retirée.";
  if (cell.visible) return "Pour défausser, retournez une carte cachée.";
  s.discard.push(held.value);
  s.held = null;
  cell.visible = true;
  removeIfIdentical(s, who, column);
  return null;
}

/** Colonne identique : 3 cartes visibles de même valeur sont retirées, et vont sur la défausse. */
function removeIfIdentical(s: TutorialState, who: Who, column: number): void {
  const cells = s.grids[who][column];
  if (cells === undefined || cells.length !== 3) return;
  const first = cells[0];
  if (first === undefined) return;
  if (!cells.every((cell) => cell.visible && cell.value === first.value)) return;
  s.discard.push(...cells.map((cell) => cell.value));
  s.grids[who][column] = [];
}
