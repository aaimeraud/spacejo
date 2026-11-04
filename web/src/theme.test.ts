import { describe, expect, it } from "vitest";
import { THEME_KEY } from "./config";
import { resolveTheme, saveTheme, storedTheme } from "./theme";

function memoryStorage() {
  const data = new Map<string, string>();
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => void data.set(key, value),
    removeItem: (key: string) => void data.delete(key),
  };
}

describe("thème", () => {
  it("suit le système tant que rien n'est choisi", () => {
    const storage = memoryStorage();
    expect(storedTheme(storage)).toBeNull();
    expect(resolveTheme(storedTheme(storage), true)).toBe("light");
    expect(resolveTheme(storedTheme(storage), false)).toBe("dark");
  });

  it("garde le choix de l'utilisateur, qui l'emporte sur le système", () => {
    const storage = memoryStorage();
    saveTheme(storage, "light");
    expect(resolveTheme(storedTheme(storage), false)).toBe("light");
  });

  it("ignore une valeur inconnue", () => {
    const storage = memoryStorage();
    storage.setItem(THEME_KEY, "violet");
    expect(storedTheme(storage)).toBeNull();
  });
});
