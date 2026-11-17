import { describe, expect, it } from "vitest";
import { OFFICIAL_RULES } from "./rulesInfo";
import {
  TUTORIAL_STEPS,
  applyStep,
  stateBeforeStep,
  sameTarget,
  targetOf,
  type TutorialStep,
} from "./tutorial";
import { initialState, slotOf, type TutorialState } from "./game/tutorialGame";

/** Tous les textes affichés par le tutoriel, pour vérifier les points qu'il doit annoncer. */
function allText(steps: TutorialStep[]): string {
  return steps
    .flatMap((step) => {
      if (step.kind === "info") return step.paragraphs;
      if (step.kind === "play") return [step.prompt, step.success];
      return step.narration;
    })
    .join(" ");
}

const finalState = (): TutorialState => stateBeforeStep(TUTORIAL_STEPS, TUTORIAL_STEPS.length);

describe("tutoriel : contenu", () => {
  it("a un titre et du texte à chaque étape", () => {
    for (const step of TUTORIAL_STEPS) {
      expect(step.title.trim()).not.toBe("");
      if (step.kind === "info") {
        expect(step.paragraphs.length).toBeGreaterThan(0);
        for (const paragraph of step.paragraphs) expect(paragraph.trim()).not.toBe("");
      } else if (step.kind === "play") {
        expect(step.prompt.trim()).not.toBe("");
        expect(step.success.trim()).not.toBe("");
      } else {
        expect(step.narration.length).toBeGreaterThan(0);
      }
    }
  });

  it("contient des gestes à faire et des coups de Aatish", () => {
    expect(TUTORIAL_STEPS.some((step) => step.kind === "play")).toBe(true);
    expect(TUTORIAL_STEPS.some((step) => step.kind === "bot")).toBe(true);
  });

  it("annonce le score de fin des règles officielles", () => {
    expect(allText(TUTORIAL_STEPS)).toContain(`${OFFICIAL_RULES.end_score} points`);
  });

  it("n'écrit pas de pronom masculin ou féminin pour Aatish", () => {
    // Aatish est un personnage fictif : les textes le désignent par son nom pour éviter tout genre imposé.
    const text = allText(TUTORIAL_STEPS);
    expect(text).not.toMatch(/Aatish[^.]*\b(il|elle)\b/);
  });
});

describe("tutoriel : scénario joué", () => {
  it("se rejoue sans erreur du début à la fin", () => {
    expect(() => stateBeforeStep(TUTORIAL_STEPS, TUTORIAL_STEPS.length)).not.toThrow();
  });

  it("donne la même partie à chaque lancement", () => {
    expect(JSON.stringify(finalState())).toBe(JSON.stringify(finalState()));
  });

  it("fait commencer le joueur avec le plus gros total de mise en place", () => {
    // Juste après le coup de Aatish, les deux totaux visibles doivent confirmer ce que le texte annonce : 18 contre 3.
    const index = TUTORIAL_STEPS.findIndex((step) => step.kind === "bot");
    const state = stateBeforeStep(TUTORIAL_STEPS, index + 1);
    const total = (who: "me" | "bot") =>
      state.grids[who].flat().reduce((sum, cell) => sum + (cell.visible ? cell.value : 0), 0);
    expect(total("me")).toBe(18);
    expect(total("bot")).toBe(3);
  });

  it("retire la colonne 2 à la fin : trois 9 identiques", () => {
    const state = finalState();
    expect(state.grids.me[1]).toEqual([]);
    expect(state.discard.slice(-3)).toEqual([9, 9, 9]);
    expect(state.held).toBeNull();
  });

  it("ne montre jamais la valeur d'une carte cachée, celle de Aatish comme la sienne", () => {
    const state = finalState();
    const cells = [...state.grids.me.flat(), ...state.grids.bot.flat()];
    const hidden = cells.filter((cell) => !cell.visible);
    expect(hidden.length).toBeGreaterThan(0);
    for (const cell of hidden) expect(slotOf(cell)).toEqual({ kind: "Hidden" });
  });

  it("applique chaque coup de Aatish sans toucher à la grille du joueur", () => {
    const bot = TUTORIAL_STEPS.find((step) => step.kind === "bot");
    if (bot === undefined) throw new Error("aucune étape de Aatish");
    const after = applyStep(initialState(), bot);
    expect(after.grids.me).toEqual(initialState().grids.me);
  });
});

describe("tutoriel : cibles des gestes", () => {
  it("désigne la pioche pour piocher, la défausse pour la prendre, la case sinon", () => {
    expect(targetOf({ type: "draw" })).toEqual({ kind: "pile" });
    expect(targetOf({ type: "takeDiscard" })).toEqual({ kind: "discard" });
    expect(targetOf({ type: "swap", column: 2, row: 1 })).toEqual({ kind: "cell", column: 2, row: 1 });
  });

  it("compare deux cibles sans confondre une case et la pioche", () => {
    expect(sameTarget({ kind: "pile" }, { kind: "pile" })).toBe(true);
    expect(sameTarget({ kind: "pile" }, { kind: "discard" })).toBe(false);
    expect(sameTarget({ kind: "cell", column: 0, row: 1 }, { kind: "cell", column: 1, row: 1 })).toBe(false);
  });
});
