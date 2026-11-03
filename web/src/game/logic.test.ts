import { describe, expect, it } from "vitest";
import type { ColumnView, GameView, PlayerView } from "../api/types";
import { OFFICIAL_RULES } from "../rulesInfo";
import { hint, ranking, slotAction, slotLabel } from "./logic";

const column = (...values: (number | null)[]): ColumnView => ({
  kind: "Present",
  slots: values.map((v) => (v === null ? { kind: "Hidden" } : { kind: "Visible", value: v })),
});

const player = (id: number, overrides: Partial<PlayerView> = {}): PlayerView => ({
  id,
  name: `P${id}`,
  total_score: 0,
  initial_reveals: 2,
  missed_turns: 0,
  eliminated: false,
  grid: [column(5, null, null), column(null, null, null), { kind: "Removed" }, column(1, 2, 3)],
  ...overrides,
});

const baseView = (overrides: Partial<GameView> = {}): GameView => ({
  me: 0,
  host: 0,
  phase: "Playing",
  round: 1,
  rules: OFFICIAL_RULES,
  players: [player(0), player(1)],
  current_player: 0,
  finisher: null,
  held: null,
  draw_pile_count: 100,
  discard_top: 4,
  discard_count: 1,
  rounds: [],
  winners: [],
  ...overrides,
});

describe("slotAction", () => {
  it("retourne une carte cachée pendant la mise en place", () => {
    const view = baseView({ phase: "InitialReveal", current_player: null, players: [player(0, { initial_reveals: 1 }), player(1)] });
    expect(slotAction(view, "swap", 1, 0)).toEqual({ type: "RevealInitial", column: 1, row: 0 });
    expect(slotAction(view, "swap", 0, 0)).toBeNull(); // déjà visible
  });

  it("ne fait rien après les 2 retournements", () => {
    const view = baseView({ phase: "InitialReveal", current_player: null });
    expect(slotAction(view, "swap", 1, 0)).toBeNull();
  });

  it("ne fait rien sans carte en main ou hors de son tour", () => {
    expect(slotAction(baseView(), "swap", 0, 0)).toBeNull();
    const notMine = baseView({ current_player: 1, held: { source: "Pile", value: null } });
    expect(slotAction(notMine, "swap", 0, 0)).toBeNull();
  });

  it("échange ou retourne selon le mode avec une carte piochée", () => {
    const view = baseView({ held: { source: "Pile", value: 7 } });
    expect(slotAction(view, "swap", 0, 0)).toEqual({ type: "SwapHeld", column: 0, row: 0 });
    expect(slotAction(view, "reveal", 0, 1)).toEqual({ type: "DiscardHeldAndReveal", column: 0, row: 1 });
    expect(slotAction(view, "reveal", 0, 0)).toBeNull(); // carte visible : rien à retourner
  });

  it("une carte prise sur la défausse s'échange toujours", () => {
    const view = baseView({ held: { source: "Discard", value: 4 } });
    expect(slotAction(view, "reveal", 0, 1)).toEqual({ type: "SwapHeld", column: 0, row: 1 });
  });

  it("une colonne retirée n'est pas jouable", () => {
    const view = baseView({ held: { source: "Pile", value: 7 } });
    expect(slotAction(view, "swap", 2, 0)).toBeNull();
    expect(slotAction(view, "swap", 9, 0)).toBeNull();
  });
});

describe("hint", () => {
  it("indique à qui est le tour", () => {
    expect(hint(baseView({ current_player: 1 }), "swap")).toContain("P1");
    expect(hint(baseView(), "swap")).toContain("piochez");
    expect(hint(baseView({ finisher: 1 }), "swap")).toContain("dernier tour");
  });
});

describe("ranking", () => {
  it("trie par total reçu du serveur, puis par ordre d'arrivée", () => {
    const view = baseView({
      players: [player(0, { total_score: 40 }), player(1, { total_score: 12 }), player(2, { total_score: 12 })],
    });
    expect(ranking(view).map((p) => p.id)).toEqual([1, 2, 0]);
  });
});

describe("slotLabel", () => {
  it("décrit la carte sans jamais deviner une valeur cachée", () => {
    expect(slotLabel({ kind: "Hidden" }, 0, 2)).toBe("Carte cachée, colonne 1, ligne 3");
    expect(slotLabel({ kind: "Visible", value: -2 }, 3, 0)).toBe("Carte -2, colonne 4, ligne 1");
  });
});
