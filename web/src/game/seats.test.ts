import { describe, expect, it } from "vitest";
import type { GameView, PlayerView } from "../api/types";
import { OFFICIAL_RULES } from "../rulesInfo";
import { hiddenCount, seatPlayers } from "./seats";

const player = (id: number): PlayerView => ({ id, name: `P${id}`, total_score: 0, initial_reveals: 0,
  missed_turns: 0,
  eliminated: false,
  grid: [],
});

const view = (count: number, me: number): GameView => ({
  me,
  host: 0,
  phase: "Playing",
  round: 1,
  rules: OFFICIAL_RULES,
  players: Array.from({ length: count }, (_, i) => player(i)),
  current_player: 0,
  finisher: null,
  held: null,
  draw_pile_count: 0,
  discard_top: null,
  discard_count: 0,
  rounds: [],
  winners: [],
});

describe("seatPlayers", () => {
  it("place le joueur en bas, puis les autres dans l'ordre du tour", () => {
    const seats = seatPlayers(view(4, 2));
    expect(seats.map((s) => s.player.id)).toEqual([2, 3, 0, 1]);
    expect(seats[0]).toMatchObject({ index: 0, angle: 90, x: 50, y: 90 });
  });

  it("à 4 joueurs : en bas, à gauche, en haut, à droite (sens des aiguilles d'une montre)", () => {
    const seats = seatPlayers(view(4, 0));
    expect(seats.map((s) => s.angle)).toEqual([90, 180, 270, 0]);
    expect(seats[1]).toMatchObject({ x: 8, y: 50 });
    expect(seats[2]).toMatchObject({ x: 50, y: 10 });
    expect(seats[3]).toMatchObject({ x: 92, y: 50 });
  });

  it("tourne chaque grille vers son propriétaire", () => {
    expect(seatPlayers(view(4, 0)).map((s) => s.rotation)).toEqual([0, 90, 180, 270]);
    expect(seatPlayers(view(3, 0)).map((s) => s.rotation)).toEqual([0, 112.5, 247.5]);
  });

  it("à 2 joueurs, l'adversaire est en face", () => {
    const seats = seatPlayers(view(2, 1));
    expect(seats.map((s) => [s.player.id, s.angle])).toEqual([
      [1, 90],
      [0, 270],
    ]);
  });

  it("à 3 joueurs, les deux autres sont en haut à gauche et en haut à droite", () => {
    const [, left, right] = seatPlayers(view(3, 0));
    expect(left?.angle).toBe(202.5);
    expect(right?.angle).toBe(337.5);
    expect(left!.x).toBeLessThan(50);
    expect(right!.x).toBeGreaterThan(50);
    expect(left!.y).toBeLessThan(50);
  });

  it("à 10 joueurs, les adversaires restent hors du secteur de ma grille", () => {
    const seats = seatPlayers(view(10, 0));
    expect(seats).toHaveLength(10);
    expect(new Set(seats.map((s) => s.angle)).size).toBe(10);
    // Secteur 45°–135° (autour du bas, 90°) réservé à ma grille.
    for (const seat of seats.slice(1)) {
      const a = seat.angle < 90 ? seat.angle + 360 : seat.angle;
      expect(a).toBeGreaterThan(135);
      expect(a).toBeLessThan(405);
    }
    // Toujours dans l'ordre du tour, sens des aiguilles d'une montre.
    const unwrapped = seats.slice(1).map((s) => (s.angle < 90 ? s.angle + 360 : s.angle));
    expect([...unwrapped].sort((a, b) => a - b)).toEqual(unwrapped);
  });

  it("gère une partie sans joueur", () => {
    expect(seatPlayers(view(0, 0))).toEqual([]);
  });
});

describe("hiddenCount", () => {
  it("compte les cartes cachées des colonnes présentes", () => {
    expect(
      hiddenCount([
        { kind: "Present", slots: [{ kind: "Hidden" }, { kind: "Visible", value: 3 }, { kind: "Hidden" }] },
        { kind: "Removed" },
        { kind: "Present", slots: [{ kind: "Hidden" }, { kind: "Hidden" }, { kind: "Visible", value: 0 }] },
      ]),
    ).toBe(4);
  });
});
