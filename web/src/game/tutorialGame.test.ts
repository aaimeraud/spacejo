import { describe, expect, it } from "vitest";
import { applyAction, initialState, slotOf, type Cell, type TutorialState } from "./tutorialGame";

/** Joue un geste et lève une erreur si le moteur le refuse : ces tests décrivent des gestes permis. */
function play(state: TutorialState, who: "me" | "bot", action: Parameters<typeof applyAction>[2]): TutorialState {
  const result = applyAction(state, who, action);
  if (!result.ok) throw new Error(result.error);
  return result.state;
}

/** Renvoie le message d'erreur d'un geste refusé. */
function refusal(state: TutorialState, who: "me" | "bot", action: Parameters<typeof applyAction>[2]): string {
  const result = applyAction(state, who, action);
  if (result.ok) throw new Error("le geste aurait dû être refusé");
  return result.error;
}

describe("moteur du tutoriel", () => {
  it("ne modifie pas l'état d'origine", () => {
    const before = initialState();
    const snapshot = JSON.stringify(before);
    play(before, "me", { type: "reveal", column: 0, row: 0 });
    expect(JSON.stringify(before)).toBe(snapshot);
  });

  it("masque la valeur d'une carte cachée", () => {
    const hidden: Cell = { value: 9, visible: false };
    const visible: Cell = { value: 9, visible: true };
    expect(slotOf(hidden)).toEqual({ kind: "Hidden" });
    expect(slotOf(visible)).toEqual({ kind: "Visible", value: 9 });
  });

  it("limite la mise en place à deux retournements par joueur", () => {
    let state = play(initialState(), "me", { type: "reveal", column: 0, row: 0 });
    state = play(state, "me", { type: "reveal", column: 0, row: 1 });
    expect(refusal(state, "me", { type: "reveal", column: 0, row: 2 })).toContain("mise en place");
  });

  it("refuse de retourner une carte déjà visible", () => {
    const state = play(initialState(), "me", { type: "reveal", column: 1, row: 0 });
    expect(refusal(state, "me", { type: "reveal", column: 1, row: 0 })).toContain("déjà face visible");
  });

  it("pioche une carte puis la place : la carte remplacée part sur la défausse, face visible", () => {
    let state = play(initialState(), "me", { type: "draw" });
    expect(state.held).toEqual({ value: 6, from: "pile" });
    state = play(state, "me", { type: "swap", column: 0, row: 0 });
    expect(state.held).toBeNull();
    expect(state.discard.at(-1)).toBe(10);
    expect(state.grids.me[0]?.[0]).toEqual({ value: 6, visible: true });
  });

  it("ne défausse pas une carte prise sur la défausse", () => {
    const state = play(initialState(), "me", { type: "takeDiscard" });
    expect(refusal(state, "me", { type: "discardAndReveal", column: 0, row: 0 })).toContain("doit être échangée");
  });

  it("défausse la carte piochée et retourne une carte cachée", () => {
    let state = play(initialState(), "me", { type: "draw" });
    state = play(state, "me", { type: "discardAndReveal", column: 0, row: 1 });
    expect(state.discard.at(-1)).toBe(6);
    expect(state.grids.me[0]?.[1]).toEqual({ value: 4, visible: true });
  });

  it("refuse de piocher pendant qu'une carte est en main", () => {
    const state = play(initialState(), "me", { type: "draw" });
    expect(refusal(state, "me", { type: "draw" })).toContain("déjà en main");
  });

  it("retire une colonne de trois cartes identiques visibles, qui vont sur la défausse", () => {
    // État construit à la main pour tester seulement la règle : la colonne 2 a deux 9 visibles, on en place un
    // troisième à la place de la carte cachée.
    const state: TutorialState = {
      ...initialState(),
      grids: {
        me: [
          [{ value: 1, visible: true }, { value: 2, visible: true }, { value: 3, visible: true }],
          [
            { value: 9, visible: true },
            { value: 9, visible: true },
            { value: 5, visible: false },
          ],
          [],
          [],
        ],
        bot: initialState().grids.bot,
      },
      held: { value: 9, from: "pile" },
    };
    const after = play(state, "me", { type: "swap", column: 1, row: 2 });
    expect(after.grids.me[1]).toEqual([]);
    expect(after.discard.slice(-4)).toEqual([5, 9, 9, 9]);
  });

  it("ne retire pas une colonne dont une carte est encore cachée", () => {
    const state: TutorialState = {
      ...initialState(),
      held: { value: 9, from: "pile" },
    };
    const after = play(state, "me", { type: "swap", column: 1, row: 2 });
    // Le 9 posé n'est pas accompagné de deux autres 9 visibles : la colonne reste en jeu.
    expect(after.grids.me[1]).toHaveLength(3);
  });
});
