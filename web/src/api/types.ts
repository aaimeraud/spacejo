// Types de l'API, écrits d'après docs/CONTRACT.md (la référence commune du front et du back).

export type PenaltyRule = "DoubleUnlessStrictlyLowest" | "DoubleOnlyIfBeaten" | "Disabled";
export type PenaltyScope = "RoundTotal" | "PositiveCardsOnly";
export type FirstPlayerTie = "JoinOrder" | "Random";
export type NextRoundStarter = "Finisher" | "RoundWinner" | "Rotate";
export type EmptyPile = "ReshuffleDiscard" | "EndRound";
export type EndTie = "SharedVictory" | "ExtraRound";

export interface Rules {
  penalty: PenaltyRule;
  penalty_scope: PenaltyScope;
  first_player_tie: FirstPlayerTie;
  next_round_starter: NextRoundStarter;
  empty_pile: EmptyPile;
  end_tie: EndTie;
  end_score: number;
}

export type Action =
  | { type: "RevealInitial"; column: number; row: number }
  | { type: "DrawFromPile" }
  | { type: "TakeDiscard" }
  | { type: "SwapHeld"; column: number; row: number }
  | { type: "DiscardHeldAndReveal"; column: number; row: number }
  | { type: "NextRound" };

export type PublicSlot = { kind: "Hidden" } | { kind: "Visible"; value: number };

export type ColumnView = { kind: "Present"; slots: PublicSlot[] } | { kind: "Removed" };

export interface PlayerView {
  id: number;
  name: string;
  total_score: number;
  initial_reveals: number;
  /** Tours passés d'affilée pour cause d'absence. */
  missed_turns: number;
  /** Éliminé après trop de tours manqués : ne joue plus. */
  eliminated: boolean;
  grid: ColumnView[];
}

/** Présence d'un joueur (aucune requête depuis 15 s = absent). */
export interface PresenceEntry {
  player: number;
  connected: boolean;
}

/** Joueur absent qui bloque la partie, et compte à rebours avant que son tour soit passé. */
export interface Waiting {
  player: number;
  seconds_left: number;
  missed_turns: number;
  max_missed_turns: number;
}

/** Champs communs à toutes les réponses qui donnent l'état. */
export interface PresenceInfo {
  presence: PresenceEntry[];
  waiting: Waiting | null;
}

export type Phase = "Lobby" | "InitialReveal" | "Playing" | "RoundOver" | "GameOver";

export interface HeldView {
  source: "Pile" | "Discard";
  value: number | null;
}

export interface PlayerRoundScore {
  player: number;
  raw: number;
  score: number;
  penalized: boolean;
}

export interface RoundResult {
  round: number;
  finisher: number | null;
  scores: PlayerRoundScore[];
}

export interface GameView {
  me: number;
  host: number;
  phase: Phase;
  round: number;
  rules: Rules;
  players: PlayerView[];
  current_player: number | null;
  finisher: number | null;
  held: HeldView | null;
  draw_pile_count: number;
  discard_top: number | null;
  discard_count: number;
  rounds: RoundResult[];
  winners: number[];
}

/** Réponse de `create` et `join`. */
export interface JoinedResponse extends PresenceInfo {
  code: string;
  token: string;
  player_id: number;
  version: number;
  view: GameView;
}

/** Réponse de `rules`, `start` et `action`. */
export interface WriteResponse extends PresenceInfo {
  version: number;
  view: GameView;
}

/** Réponse de `state`. */
export type StateResponse =
  | ({ changed: false; version: number } & PresenceInfo)
  | ({ changed: true; version: number; view: GameView } & PresenceInfo);

export interface ErrorBody {
  error: { code: string; message: string };
}
