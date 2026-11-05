import { describe, expect, it } from "vitest";
import { CARD_MAX_VALUE, CARD_MIN_VALUE, HOME_CARD_COUNT } from "../config";
import { randomHomeCards } from "./homeCards";

/** Générateur pseudo-aléatoire déterministe (pour des tests reproductibles). */
function seeded(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1103515245 + 12345) % 2147483648;
    return state / 2147483648;
  };
}

describe("cartes de l'accueil", () => {
  it("tire des valeurs différentes, valides, de la plus petite à la plus grande", () => {
    for (let seed = 1; seed <= 50; seed += 1) {
      const cards = randomHomeCards(seeded(seed));
      expect(cards).toHaveLength(HOME_CARD_COUNT);
      expect(new Set(cards).size).toBe(HOME_CARD_COUNT);
      expect([...cards].sort((a, b) => a - b)).toEqual(cards);
      for (const card of cards) {
        expect(Number.isInteger(card)).toBe(true);
        expect(card).toBeGreaterThanOrEqual(CARD_MIN_VALUE);
        expect(card).toBeLessThanOrEqual(CARD_MAX_VALUE);
      }
    }
  });

  it("change d'une visite à l'autre", () => {
    const draws = new Set(Array.from({ length: 20 }, (_, i) => randomHomeCards(seeded(i + 1)).join(",")));
    expect(draws.size).toBeGreaterThan(10);
  });

  it("reste valide aux bornes du générateur", () => {
    // `random` toujours à 0 : aucun échange, ce sont les premières valeurs.
    expect(randomHomeCards(() => 0)).toEqual([-2, -1, 0, 1, 2]);
    // Presque 1 : on tire toujours la dernière place restante, sans sortir de la liste.
    const high = randomHomeCards(() => 0.999999);
    expect(new Set(high).size).toBe(HOME_CARD_COUNT);
    expect(high.every((card) => card >= CARD_MIN_VALUE && card <= CARD_MAX_VALUE)).toBe(true);
  });
});
