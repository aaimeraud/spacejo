import { describe, expect, it } from "vitest";
import { codeFromSearch, inviteLink } from "./invite";

describe("lien d'invitation", () => {
  it("ajoute le code à l'adresse du site", () => {
    expect(inviteLink("https://spacejo.example", "K7MPQX")).toBe("https://spacejo.example/?code=K7MPQX");
  });

  it("relit le code d'un lien, en majuscules", () => {
    expect(codeFromSearch("?code=K7MPQX")).toBe("K7MPQX");
    expect(codeFromSearch("?code=k7mpqx")).toBe("K7MPQX");
    expect(codeFromSearch("?code=%20K7MPQX%20")).toBe("K7MPQX");
  });

  it("ignore un code absent ou mal formé", () => {
    expect(codeFromSearch("")).toBeNull();
    expect(codeFromSearch("?autre=1")).toBeNull();
    expect(codeFromSearch("?code=K7MP")).toBeNull();
    // 0, O, 1 et I ne font pas partie de l'alphabet des codes.
    expect(codeFromSearch("?code=K7MPQ0")).toBeNull();
    expect(codeFromSearch("?code=K7MPQI")).toBeNull();
  });

  it("fait l'aller-retour", () => {
    const link = inviteLink("http://localhost:5173", "ABCDEF");
    expect(codeFromSearch(new URL(link).search)).toBe("ABCDEF");
  });
});
