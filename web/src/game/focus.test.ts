import { describe, expect, it } from "vitest";
import { focusedOpponent } from "./focus";

const opponents = [
  { id: 1, eliminated: false },
  { id: 2, eliminated: false },
  { id: 3, eliminated: false },
];

describe("adversaire montré en grand", () => {
  it("montre l'adversaire qui joue", () => {
    expect(focusedOpponent(opponents, 2, null, null)).toBe(2);
  });

  it("pendant mon tour, garde celui qui vient de jouer", () => {
    expect(focusedOpponent(opponents, 0, null, 3)).toBe(3);
  });

  it("un choix fait à la main l'emporte", () => {
    expect(focusedOpponent(opponents, 2, 1, 3)).toBe(1);
  });

  it("ignore un choix qui n'est pas un adversaire", () => {
    expect(focusedOpponent(opponents, 2, 0, null)).toBe(2);
    expect(focusedOpponent(opponents, 0, 9, null)).toBe(1);
  });

  it("sans indice, prend le premier adversaire encore en jeu", () => {
    const withEliminated = [{ id: 1, eliminated: true }, ...opponents.slice(1)];
    expect(focusedOpponent(withEliminated, null, null, null)).toBe(2);
    expect(focusedOpponent([], null, null, null)).toBeNull();
  });
});
