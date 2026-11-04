import { describe, expect, it } from "vitest";
import { ApiError, SpaceJoApi, TOKEN_HEADER, type FetchLike } from "./client";

interface Call {
  url: string;
  init: RequestInit | undefined;
}

function fakeFetch(status: number, body: unknown, calls: Call[]): FetchLike {
  return (url, init) => {
    calls.push({ url, init });
    return Promise.resolve(new Response(JSON.stringify(body), { status }));
  };
}

describe("SpaceJoApi", () => {
  it("envoie l'action avec la version attendue et le jeton dans l'en-tête", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(200, { version: 8, view: {} }, calls));
    await api.act("K7MPQX", "secret", 7, { type: "DrawFromPile" });
    const call = calls[0];
    expect(call?.url).toBe("/api/action");
    expect(call?.init?.method).toBe("POST");
    expect((call?.init?.headers as Record<string, string>)[TOKEN_HEADER]).toBe("secret");
    expect(JSON.parse(String(call?.init?.body))).toEqual({
      code: "K7MPQX",
      expected_version: 7,
      action: { type: "DrawFromPile" },
    });
  });

  it("demande à passer le tour d'un joueur absent", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(200, { version: 9, view: {} }, calls));
    await api.skip("K7MPQX", "secret", 8, 2);
    expect(calls[0]?.url).toBe("/api/skip");
    expect(JSON.parse(String(calls[0]?.init?.body))).toEqual({ code: "K7MPQX", expected_version: 8, player: 2 });
  });

  it("interroge le contrôle de santé sans jeton", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(200, { status: "ok" }, calls));
    expect(await api.health()).toEqual({ status: "ok" });
    expect(calls[0]?.url).toBe("/api/health");
    expect((calls[0]?.init?.headers as Record<string, string>)[TOKEN_HEADER]).toBeUndefined();
  });

  it("ne met pas le jeton dans l'URL de lecture de l'état", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(200, { changed: false, version: 3 }, calls));
    const response = await api.state("K7MPQX", "secret", 3);
    expect(response).toEqual({ changed: false, version: 3 });
    expect(calls[0]?.url).toBe("/api/state?code=K7MPQX&known_version=3");
    expect(calls[0]?.url).not.toContain("secret");
  });

  it("crée une partie sans réglages quand aucun n'est donné", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(201, {}, calls));
    await api.create("Alice");
    expect(JSON.parse(String(calls[0]?.init?.body))).toEqual({ name: "Alice" });
  });

  it("demande une partie publique seulement quand on le choisit", async () => {
    const calls: Call[] = [];
    const api = new SpaceJoApi(fakeFetch(201, {}, calls));
    await api.create("Alice", undefined, true);
    expect(JSON.parse(String(calls[0]?.init?.body))).toEqual({ name: "Alice", public: true });
  });

  it("liste les parties publiques sans jeton", async () => {
    const calls: Call[] = [];
    const games = [{ code: "K7MPQX", host: "Alice", players: 2, max_players: 10 }];
    const api = new SpaceJoApi(fakeFetch(200, { games }, calls));
    expect(await api.publicGames()).toEqual({ games });
    expect(calls[0]?.url).toBe("/api/public-games");
    expect(calls[0]?.init?.method).toBe("GET");
    expect((calls[0]?.init?.headers as Record<string, string>)[TOKEN_HEADER]).toBeUndefined();
  });

  it("transforme une réponse d'erreur en ApiError", async () => {
    const api = new SpaceJoApi(
      fakeFetch(409, { error: { code: "version_conflict", message: "La partie a changé" } }, []),
    );
    const error = await api.start("K7MPQX", "secret", 1).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).code).toBe("version_conflict");
    expect((error as ApiError).isConflict).toBe(true);
    expect((error as ApiError).message).toBe("La partie a changé");
  });

  it("signale un serveur injoignable", async () => {
    const api = new SpaceJoApi(() => Promise.reject(new Error("offline")));
    const error = await api.join("K7MPQX", "Bob").catch((e: unknown) => e);
    expect((error as ApiError).code).toBe("network");
  });
});
