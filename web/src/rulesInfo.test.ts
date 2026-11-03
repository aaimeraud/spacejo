import { describe, expect, it } from "vitest";
import {
  CHOICES,
  END_SCORE_LIMITS,
  END_SCORE_SHORTCUTS,
  OFFICIAL_RULES,
  PRESETS,
  describe as describeOption,
  differencesFromOfficial,
  endScoreStatus,
  matchPreset,
  rulesSummary,
  type ChoiceInfo,
  type ChoiceKey,
} from "./rulesInfo";

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

  it("chaque valeur a un exemple concret", () => {
    for (const info of CHOICES) {
      for (const option of info.options) expect(option.example.length).toBeGreaterThan(20);
    }
  });
});

describe("préréglages", () => {
  it("le préréglage officiel est exactement Rules::default", () => {
    expect(PRESETS.find((p) => p.id === "official")?.rules).toEqual(OFFICIAL_RULES);
  });

  it("chaque préréglage est complet et dans les bornes", () => {
    for (const preset of PRESETS) {
      expect(Object.keys(preset.rules).sort()).toEqual(Object.keys(OFFICIAL_RULES).sort());
      expect(preset.rules.end_score).toBeGreaterThanOrEqual(END_SCORE_LIMITS.min);
      expect(preset.rules.end_score).toBeLessThanOrEqual(END_SCORE_LIMITS.max);
    }
    for (const value of END_SCORE_SHORTCUTS) {
      expect(value).toBeGreaterThanOrEqual(END_SCORE_LIMITS.min);
      expect(value).toBeLessThanOrEqual(END_SCORE_LIMITS.max);
    }
  });

  it("reconnaît un préréglage ou des réglages personnalisés", () => {
    expect(matchPreset(OFFICIAL_RULES)?.id).toBe("official");
    expect(matchPreset({ ...OFFICIAL_RULES, penalty: "Disabled" })?.id).toBe("relaxed");
    expect(matchPreset({ ...OFFICIAL_RULES, end_score: 50 })?.id).toBe("quick");
    expect(matchPreset({ ...OFFICIAL_RULES, penalty: "Disabled", end_score: 50 })).toBeNull();
  });

  it("résume les réglages pour le salon", () => {
    expect(rulesSummary(OFFICIAL_RULES)).toBe("Règles officielles");
    const custom = { ...OFFICIAL_RULES, penalty: "Disabled" as const, end_score: 50 };
    expect(differencesFromOfficial(custom)).toBe(2);
    expect(rulesSummary(custom)).toBe("Règles officielles, sauf 2 réglages");
    expect(rulesSummary({ ...OFFICIAL_RULES, end_tie: "ExtraRound" })).toBe("Règles officielles, sauf 1 réglage");
  });
});
