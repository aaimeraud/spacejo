import { describe, expect, it } from "vitest";
import { OFFICIAL_RULES } from "./rulesInfo";
import { TUTORIAL_MAX_CARD, TUTORIAL_MIN_CARD, TUTORIAL_STEPS, type TutorialCard } from "./tutorial";

const validCard = (card: TutorialCard) =>
  card === null || (Number.isInteger(card) && card >= TUTORIAL_MIN_CARD && card <= TUTORIAL_MAX_CARD);

describe("tutoriel", () => {
  it("reste court et chaque étape a un titre et du texte", () => {
    expect(TUTORIAL_STEPS.length).toBeGreaterThanOrEqual(4);
    expect(TUTORIAL_STEPS.length).toBeLessThanOrEqual(8);
    for (const step of TUTORIAL_STEPS) {
      expect(step.title.trim()).not.toBe("");
      expect(step.paragraphs.length).toBeGreaterThan(0);
      for (const paragraph of step.paragraphs) expect(paragraph.trim()).not.toBe("");
    }
  });

  it("n'utilise que des cartes qui existent", () => {
    for (const step of TUTORIAL_STEPS) {
      const visual = step.visual;
      if (visual === undefined) continue;
      const cards = visual.kind === "cards" ? visual.cards : visual.columns.flat();
      expect(cards.every(validCard)).toBe(true);
    }
  });

  it("dessine des grilles de 4 colonnes de 3 cartes", () => {
    for (const step of TUTORIAL_STEPS) {
      if (step.visual?.kind !== "grid") continue;
      expect(step.visual.columns).toHaveLength(4);
      for (const column of step.visual.columns) expect(column).toHaveLength(3);
    }
  });

  it("montre une colonne identique : trois cartes visibles de même valeur", () => {
    const grids = TUTORIAL_STEPS.flatMap((step) => (step.visual?.kind === "grid" ? [step.visual] : []));
    const withColumn = grids.filter((grid) => grid.highlightColumn !== undefined);
    expect(withColumn.length).toBeGreaterThan(0);
    for (const grid of withColumn) {
      const column = grid.columns[grid.highlightColumn ?? 0] ?? [];
      expect(column[0]).not.toBeNull();
      expect(new Set(column).size).toBe(1);
    }
  });

  it("annonce le score de fin des règles officielles", () => {
    const text = TUTORIAL_STEPS.flatMap((step) => step.paragraphs).join(" ");
    expect(text).toContain(`${OFFICIAL_RULES.end_score} points`);
  });
});
