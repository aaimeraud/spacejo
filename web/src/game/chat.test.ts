import { describe, expect, it } from "vitest";
import type { ChatMessage, ChatReaction } from "../api/types";
import { MAX_CHAT_KEPT } from "../config";
import { initial, isDoubleTap, lastMessageId, mergeMessages, nextReaction, reactionGroups, unreadCount } from "./chat";

const message = (id: number, player = 0, text = `m${id}`, reactions: ChatReaction[] = []): ChatMessage => ({
  id,
  player,
  name: `P${player}`,
  text,
  reactions,
});

describe("chat", () => {
  it("fusionne sans doublon, dans l'ordre des numéros", () => {
    const merged = mergeMessages([message(1), message(3)], [message(3), message(2), message(4)]);
    expect(merged.map((m) => m.id)).toEqual([1, 2, 3, 4]);
  });

  it("ne garde que les messages les plus récents", () => {
    const many = Array.from({ length: MAX_CHAT_KEPT + 3 }, (_, i) => message(i + 1));
    const merged = mergeMessages([], many);
    expect(merged).toHaveLength(MAX_CHAT_KEPT);
    expect(merged[0]?.id).toBe(4);
  });

  it("donne le numéro du dernier message", () => {
    expect(lastMessageId([])).toBe(0);
    expect(lastMessageId([message(5), message(9)])).toBe(9);
  });

  it("compte les messages des autres arrivés après le dernier vu", () => {
    const messages = [message(1, 1), message(2, 0), message(3, 1), message(4, 2)];
    expect(unreadCount(messages, 0, 0)).toBe(3);
    expect(unreadCount(messages, 2, 0)).toBe(2);
    expect(unreadCount(messages, 4, 0)).toBe(0);
    // Ses propres messages ne sont jamais « non lus ».
    expect(unreadCount([message(5, 0)], 0, 0)).toBe(0);
  });
});

describe("avatars et réactions", () => {
  it("l'avatar est l'initiale du nom, en majuscule", () => {
    expect(initial("alice")).toBe("A");
    expect(initial("  élodie")).toBe("É");
    expect(initial("🚀Rocket")).toBe("🚀");
    expect(initial("")).toBe("?");
  });

  it("regroupe les réactions dans l'ordre du sélecteur, avec les miennes repérées", () => {
    const m = message(1, 0, "salut", [
      { player: 0, reaction: "laugh" },
      { player: 1, reaction: "heart" },
      { player: 2, reaction: "laugh" },
    ]);
    const groups = reactionGroups(m, 2);
    expect(groups.map((g) => [g.name, g.players, g.mine])).toEqual([
      ["heart", [1], false],
      ["laugh", [0, 2], true],
    ]);
    expect(groups[0]?.emoji).toBe("❤️");
  });

  it("rechoisir sa réaction la retire, en choisir une autre la remplace", () => {
    const m = message(1, 0, "salut", [{ player: 1, reaction: "heart" }]);
    expect(nextReaction(m, 1, "heart")).toBeNull();
    expect(nextReaction(m, 1, "wow")).toBe("wow");
    expect(nextReaction(m, 2, "heart")).toBe("heart");
  });

  it("deux appuis rapprochés forment un double tap", () => {
    expect(isDoubleTap(null, 1000)).toBe(false);
    expect(isDoubleTap(1000, 1200)).toBe(true);
    expect(isDoubleTap(1000, 1400)).toBe(false);
  });
});
