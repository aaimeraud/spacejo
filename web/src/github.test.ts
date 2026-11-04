import { describe, expect, it } from "vitest";
import { GITHUB_CACHE_KEY, GITHUB_CACHE_MS } from "./config";
import { loadRepoStatus, parseCommit, parseRelease, type RepoStatus } from "./github";
import type { KeyValueStorage } from "./session";

const COMMIT = {
  sha: "0ceeadf0afb96dda183b1e3ceaf69234df20236a",
  html_url: "https://github.com/aaimeraud/spacejo/commit/0ceeadf0afb96dda183b1e3ceaf69234df20236a",
  commit: { message: "chore(release): v1.3.0\n\nCorps du message", committer: { date: "2026-11-04T15:05:45Z" } },
};
const RELEASE = { tag_name: "v1.3.0", html_url: "https://github.com/aaimeraud/spacejo/releases/tag/v1.3.0" };

function memoryStorage(): KeyValueStorage & { data: Map<string, string> } {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => void data.set(key, value),
    removeItem: (key) => void data.delete(key),
  };
}

/** Faux `fetch` : répond selon la fin de l'adresse, et compte les appels. */
function fakeGitHub(responses: { commit?: unknown; release?: unknown; status?: number }) {
  const calls: string[] = [];
  const fetcher = (url: string) => {
    calls.push(url);
    const body = url.endsWith("/releases/latest") ? responses.release : responses.commit;
    return Promise.resolve(new Response(JSON.stringify(body ?? {}), { status: responses.status ?? 200 }));
  };
  return { calls, fetcher };
}

describe("réponses de GitHub", () => {
  it("garde du commit le numéro court, le titre et le lien", () => {
    expect(parseCommit(COMMIT)).toEqual({
      shortSha: "0ceeadf",
      title: "chore(release): v1.3.0",
      url: COMMIT.html_url,
    });
  });

  it("garde de la release son tag et son lien", () => {
    expect(parseRelease(RELEASE)).toEqual({ tag: "v1.3.0", url: RELEASE.html_url });
  });

  it("refuse une réponse inattendue ou un lien hors de GitHub", () => {
    expect(parseCommit({ message: "Not Found" })).toBeNull();
    expect(parseCommit({ ...COMMIT, html_url: "javascript:alert(1)" })).toBeNull();
    expect(parseRelease({ tag_name: 3, html_url: RELEASE.html_url })).toBeNull();
    expect(parseRelease({ ...RELEASE, html_url: "https://exemple.org/x" })).toBeNull();
    expect(parseRelease(null)).toBeNull();
  });
});

describe("lecture avec cache", () => {
  const now = 1_000_000;

  it("interroge GitHub puis garde le résultat", async () => {
    const storage = memoryStorage();
    const github = fakeGitHub({ commit: COMMIT, release: RELEASE });
    const status = await loadRepoStatus(github.fetcher, storage, now);
    expect(status.commit?.shortSha).toBe("0ceeadf");
    expect(status.release?.tag).toBe("v1.3.0");
    expect(github.calls).toHaveLength(2);

    // Avant la fin du délai : aucune nouvelle requête.
    const again = await loadRepoStatus(github.fetcher, storage, now + GITHUB_CACHE_MS - 1);
    expect(again).toEqual(status);
    expect(github.calls).toHaveLength(2);

    // Après : on redemande.
    await loadRepoStatus(github.fetcher, storage, now + GITHUB_CACHE_MS + 1);
    expect(github.calls).toHaveLength(4);
  });

  it("donne ce qu'il peut quand GitHub échoue, sans le garder", async () => {
    const storage = memoryStorage();
    const failing = fakeGitHub({ status: 403 });
    const status: RepoStatus = await loadRepoStatus(failing.fetcher, storage, now);
    expect(status).toEqual({ commit: null, release: null });
    expect(storage.data.has(GITHUB_CACHE_KEY)).toBe(false);

    const offline = () => Promise.reject(new Error("hors ligne"));
    expect(await loadRepoStatus(offline, storage, now)).toEqual({ commit: null, release: null });
  });

  it("ignore un cache illisible et un stockage inutilisable", async () => {
    const storage = memoryStorage();
    storage.setItem(GITHUB_CACHE_KEY, "{pas du json");
    const github = fakeGitHub({ commit: COMMIT, release: RELEASE });
    expect((await loadRepoStatus(github.fetcher, storage, now)).release?.tag).toBe("v1.3.0");

    const broken: KeyValueStorage = {
      getItem: () => {
        throw new Error("bloqué");
      },
      setItem: () => {
        throw new Error("bloqué");
      },
      removeItem: () => undefined,
    };
    expect((await loadRepoStatus(github.fetcher, broken, now)).commit?.shortSha).toBe("0ceeadf");
  });
});
