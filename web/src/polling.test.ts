import { describe, expect, it } from "vitest";
import { POLL_IDLE_INTERVAL_MS, POLL_INTERVAL_MS, POLL_LIVE_INTERVAL_MS, POLL_LOBBY_INTERVAL_MS } from "./config";
import { pollInterval } from "./polling";

describe("rythme du polling", () => {
  it("rapide pendant une partie, onglet visible", () => {
    expect(pollInterval(false, "Playing")).toBe(POLL_INTERVAL_MS);
    expect(pollInterval(false, "InitialReveal")).toBe(POLL_INTERVAL_MS);
    expect(pollInterval(false, "RoundOver")).toBe(POLL_INTERVAL_MS);
    expect(pollInterval(false, null)).toBe(POLL_INTERVAL_MS);
  });

  it("un peu plus lent au salon", () => {
    expect(pollInterval(false, "Lobby")).toBe(POLL_LOBBY_INTERVAL_MS);
  });

  it("lent quand l'onglet est caché ou la partie finie", () => {
    expect(pollInterval(true, "Playing")).toBe(POLL_IDLE_INTERVAL_MS);
    expect(pollInterval(false, "GameOver")).toBe(POLL_IDLE_INTERVAL_MS);
  });

  it("ralentit quand le flux d'avis du serveur est ouvert", () => {
    expect(pollInterval(false, "Playing", true)).toBe(POLL_LIVE_INTERVAL_MS);
    expect(pollInterval(false, "Lobby", true)).toBe(POLL_LIVE_INTERVAL_MS);
    expect(pollInterval(true, "Playing", true)).toBe(POLL_IDLE_INTERVAL_MS);
  });

  it("reste sous le délai d'absence du serveur (15 s)", () => {
    expect(POLL_IDLE_INTERVAL_MS).toBeLessThan(15000);
    expect(POLL_LIVE_INTERVAL_MS).toBeLessThan(15000);
  });
});
