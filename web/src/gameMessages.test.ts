import { describe, expect, it } from "vitest";
import { PLAYER_THEMES } from "./config";
import { endGameMessage, playerMessagesFor } from "./gameMessages";

const { messages } = PLAYER_THEMES.kitty;

describe("message de fin de partie", () => {
  it("donne le message de victoire seule au joueur seul gagnant", () => {
    expect(endGameMessage(messages, 2, [2])).toBe(messages.winner);
  });

  it("donne le message de victoire partagée à chaque gagnant à égalité", () => {
    expect(endGameMessage(messages, 0, [0, 3])).toBe(messages.sharedWinner);
    expect(endGameMessage(messages, 3, [0, 3])).toBe(messages.sharedWinner);
  });

  it("donne le message de défaite à tous les autres, y compris les joueurs éliminés", () => {
    expect(endGameMessage(messages, 1, [2])).toBe(messages.loser);
    expect(endGameMessage(messages, 4, [0, 3])).toBe(messages.loser);
  });

  it("chaque thème a les quatre messages, non vides", () => {
    for (const theme of Object.values(PLAYER_THEMES)) {
      for (const text of Object.values(theme.messages)) expect(text.trim().length).toBeGreaterThan(0);
    }
  });

  it("donne les messages de chaque pseudo de la liste, sans tenir compte de la casse", () => {
    for (const name of ["mass", "MASSILVA", " masspace ", "massky", "MissLegendR"]) {
      expect(playerMessagesFor(name)).toBe(PLAYER_THEMES.kitty.messages);
    }
    for (const name of ["dan", "Daniel", "AAIMERAUD", "MisterLegendR"]) {
      expect(playerMessagesFor(name)).toBe(PLAYER_THEMES.daniel.messages);
    }
  });

  it("ne donne aucun message aux autres pseudos, dont ceux qui ressemblent à un pseudo de la liste", () => {
    for (const name of ["massive", "MissLegendR2", "Alice", "danny", ""]) {
      expect(playerMessagesFor(name)).toBeNull();
    }
  });
});
