import { describe, expect, it } from "vitest";
import type { ChatMessage } from "../api/types";
import { MAX_CHAT_KEPT } from "../config";
import { lastMessageId, mergeMessages, unreadCount } from "./chat";

const message = (id: number, player = 0, text = `m${id}`): ChatMessage => ({ id, player, name: `P${player}`, text });

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
