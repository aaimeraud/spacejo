import { describe, expect, it } from "vitest";
import { SESSION_STORAGE_KEY } from "./config";
import { clearSession, loadSession, saveSession, type KeyValueStorage } from "./session";

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
