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
  grid: ColumnView[];
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
export interface JoinedResponse {
  code: string;
  token: string;
  player_id: number;
  version: number;
  view: GameView;
}

/** Réponse de `rules`, `start` et `action`. */
export interface WriteResponse {
  version: number;
  view: GameView;
}

/** Réponse de `state`. */
export type StateResponse =
  | { changed: false; version: number }
  | { changed: true; version: number; view: GameView };

export interface ErrorBody {
  error: { code: string; message: string };
}
