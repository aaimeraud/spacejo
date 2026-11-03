import { describe, expect, it } from "vitest";
import { CHOICES, OFFICIAL_RULES, describe as describeOption, endScoreStatus, type ChoiceInfo, type ChoiceKey } from "./rulesInfo";

describe("rulesInfo", () => {
  it("les règles officielles correspondent au contrat (Rules::default)", () => {
    expect(OFFICIAL_RULES).toEqual({
      penalty: "DoubleUnlessStrictlyLowest",
      penalty_scope: "RoundTotal",
      first_player_tie: "JoinOrder",
      next_round_starter: "Finisher",
      empty_pile: "ReshuffleDiscard",
      end_tie: "SharedVictory",
      end_score: 100,
    });
  });

  it("chaque réglage a une explication et une étiquette pour chacune de ses valeurs", () => {
    for (const info of CHOICES) {
      expect(info.options.length).toBeGreaterThan(1);
      for (const option of info.options) {
        expect(option.explanation.length).toBeGreaterThan(10);
        expect(["officiel", "variante"]).toContain(option.status);
      }
      // La valeur officielle par défaut est décrite.
      expect(describeOption(info as ChoiceInfo<ChoiceKey>, OFFICIAL_RULES[info.key])).toBeDefined();
    }
  });

  it("couvre les 6 réglages à choix, avec les valeurs du contrat", () => {
    const values = Object.fromEntries(CHOICES.map((info) => [info.key, info.options.map((o) => o.value)]));
    expect(values).toEqual({
      penalty: ["DoubleUnlessStrictlyLowest", "DoubleOnlyIfBeaten", "Disabled"],
      penalty_scope: ["RoundTotal", "PositiveCardsOnly"],
      first_player_tie: ["JoinOrder", "Random"],
      next_round_starter: ["Finisher", "RoundWinner", "Rotate"],
      empty_pile: ["ReshuffleDiscard", "EndRound"],
      end_tie: ["SharedVictory", "ExtraRound"],
    });
  });

  it("n'étiquette « officiel » que les valeurs par défaut des règles officielles", () => {
    for (const info of CHOICES) {
      for (const option of info.options) {
        if (option.status === "officiel") expect(option.value).toBe(OFFICIAL_RULES[info.key]);
      }
    }
    expect(endScoreStatus(100)).toBe("officiel");
    expect(endScoreStatus(150)).toBe("variante");
  });
});
