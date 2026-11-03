import { describe, expect, it } from "vitest";
import { SESSION_STORAGE_KEY } from "./config";
import {
  clearSession,
  forgetSession,
  knownSessions,
  loadSession,
  rememberSession,
  saveSession,
  type KeyValueStorage,
} from "./session";

function memoryStorage(): KeyValueStorage & { data: Map<string, string> } {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => void data.set(key, value),
    removeItem: (key) => void data.delete(key),
  };
}

describe("session", () => {
  it("sauvegarde, relit et efface la session", () => {
    const storage = memoryStorage();
    expect(loadSession(storage)).toBeNull();
    saveSession(storage, { code: "K7MPQX", token: "t", playerId: 1 });
    expect(loadSession(storage)).toEqual({ code: "K7MPQX", token: "t", playerId: 1 });
    clearSession(storage);
    expect(loadSession(storage)).toBeNull();
  });

  it("ignore une valeur corrompue", () => {
    const storage = memoryStorage();
    storage.setItem(SESSION_STORAGE_KEY, "{pas du json");
    expect(loadSession(storage)).toBeNull();
    storage.setItem(SESSION_STORAGE_KEY, JSON.stringify({ code: 1 }));
    expect(loadSession(storage)).toBeNull();
  });

  it("survit à un stockage indisponible", () => {
    const broken: KeyValueStorage = {
      getItem: () => {
        throw new Error("bloqué");
      },
      setItem: () => {
        throw new Error("bloqué");
      },
      removeItem: () => {
        throw new Error("bloqué");
      },
    };
    expect(loadSession(broken)).toBeNull();
    expect(() => saveSession(broken, { code: "A", token: "t", playerId: 0 })).not.toThrow();
    expect(() => clearSession(broken)).not.toThrow();
  });
});

describe("parties mémorisées", () => {
  it("garde les parties rejointes, la plus récente d'abord, sans doublon", () => {
    const storage = memoryStorage();
    rememberSession(storage, { code: "AAAAAA", token: "a", playerId: 0 }, "Alice", 1);
    rememberSession(storage, { code: "BBBBBB", token: "b", playerId: 2 }, "Alice", 2);
    rememberSession(storage, { code: "AAAAAA", token: "a", playerId: 0 }, "Alice", 3);
    expect(knownSessions(storage).map((s) => s.code)).toEqual(["AAAAAA", "BBBBBB"]);
    forgetSession(storage, "AAAAAA");
    expect(knownSessions(storage).map((s) => s.code)).toEqual(["BBBBBB"]);
  });

  it("limite le nombre de parties mémorisées", () => {
    const storage = memoryStorage();
    for (let i = 0; i < 8; i++) {
      rememberSession(storage, { code: `CODE0${i}`, token: "t", playerId: 0 }, "Bob", i);
    }
    expect(knownSessions(storage)).toHaveLength(5);
    expect(knownSessions(storage)[0]?.code).toBe("CODE07");
  });
});

