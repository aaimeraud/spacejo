import { describe, expect, it } from "vitest";
import type { GameView } from "../api/types";
import { pileReshuffled } from "./reshuffle";

function view(round: number, phase: GameView["phase"], drawPile: number): GameView {
  return { round, phase, draw_pile_count: drawPile } as GameView;
}

describe("remélange de la défausse", () => {
  it("est repéré quand la pioche grossit pendant une manche", () => {
    expect(pileReshuffled(view(1, "Playing", 0), view(1, "Playing", 9))).toBe(true);
    expect(pileReshuffled(view(1, "Playing", 1), view(1, "Playing", 9))).toBe(true);
  });

  it("n'est pas confondu avec une pioche qui diminue ou une nouvelle manche", () => {
    expect(pileReshuffled(view(1, "Playing", 10), view(1, "Playing", 9))).toBe(false);
    expect(pileReshuffled(view(1, "Playing", 9), view(1, "Playing", 9))).toBe(false);
    expect(pileReshuffled(view(1, "RoundOver", 3), view(2, "InitialReveal", 15))).toBe(false);
    expect(pileReshuffled(view(0, "Lobby", 0), view(1, "InitialReveal", 15))).toBe(false);
    expect(pileReshuffled(null, view(1, "Playing", 9))).toBe(false);
  });
});
