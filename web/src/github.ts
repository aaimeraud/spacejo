// Version et dernier commit du jeu, lus sur le dépôt public GitHub, pour le bas de l'accueil. Le navigateur appelle
// directement l'API REST de GitHub (sans jeton : le dépôt est public). Cette API n'accepte que 60 requêtes par heure
// et par adresse : le résultat est gardé `GITHUB_CACHE_MS` dans `localStorage`. Si GitHub ne répond pas, l'accueil
// n'affiche simplement rien. Voir docs/decisions/2026-11-04-version-sur-accueil.md.
import { GITHUB_API, GITHUB_BRANCH, GITHUB_CACHE_KEY, GITHUB_CACHE_MS, GITHUB_REPO } from "./config";
import type { KeyValueStorage } from "./session";

export interface CommitInfo {
  /** Les 7 premiers caractères de l'identifiant, comme sur GitHub. */
  shortSha: string;
  /** Première ligne du message de commit. */
  title: string;
  url: string;
}

export interface ReleaseInfo {
  /** Tag de la release, par exemple « v1.3.0 ». */
  tag: string;
  url: string;
}

export interface RepoStatus {
  commit: CommitInfo | null;
  release: ReleaseInfo | null;
}

/** `fetch` utilisé (remplaçable dans les tests). */
export type Fetcher = (url: string) => Promise<Response>;

const EMPTY: RepoStatus = { commit: null, release: null };

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/** Lien sûr : uniquement une page de github.com (jamais `javascript:` ni un autre site). */
function isGitHubUrl(value: unknown): value is string {
  return typeof value === "string" && value.startsWith("https://github.com/");
}

/** Commit d'une réponse de `GET /repos/{dépôt}/commits/{branche}` ; `null` si la forme est inattendue. */
export function parseCommit(body: unknown): CommitInfo | null {
  if (!isObject(body) || typeof body.sha !== "string" || !isGitHubUrl(body.html_url)) return null;
  const message = isObject(body.commit) && typeof body.commit.message === "string" ? body.commit.message : "";
  return { shortSha: body.sha.slice(0, 7), title: message.split("\n")[0] ?? "", url: body.html_url };
}

/** Release d'une réponse de `GET /repos/{dépôt}/releases/latest` ; `null` si la forme est inattendue. */
export function parseRelease(body: unknown): ReleaseInfo | null {
  if (!isObject(body) || typeof body.tag_name !== "string" || !isGitHubUrl(body.html_url)) return null;
  return { tag: body.tag_name, url: body.html_url };
}

/** Appelle l'API ; `null` en cas d'erreur réseau, de statut d'erreur ou de JSON invalide. */
async function getJson(fetcher: Fetcher, path: string): Promise<unknown> {
  try {
    const response = await fetcher(`${GITHUB_API}/repos/${GITHUB_REPO}${path}`);
    return response.ok ? ((await response.json()) as unknown) : null;
  } catch {
    return null;
  }
}

/** Résultat gardé en cache, s'il est encore frais. */
function readCache(storage: KeyValueStorage, now: number): RepoStatus | null {
  try {
    const raw = storage.getItem(GITHUB_CACHE_KEY);
    if (raw === null) return null;
    const cached: unknown = JSON.parse(raw);
    if (!isObject(cached) || typeof cached.savedAt !== "number" || now - cached.savedAt >= GITHUB_CACHE_MS) {
      return null;
    }
    return { commit: parseCached(cached.commit, "shortSha"), release: parseCached(cached.release, "tag") };
  } catch {
    return null;
  }
}

/** Relit une entrée du cache (même vérification de lien que pour une réponse de GitHub). */
function parseCached<T extends CommitInfo | ReleaseInfo>(value: unknown, key: "shortSha" | "tag"): T | null {
  if (!isObject(value) || typeof value[key] !== "string" || !isGitHubUrl(value.url)) return null;
  return value as unknown as T;
}

/** Version et dernier commit : depuis le cache s'il est frais, sinon depuis GitHub (puis mis en cache, si les
 * deux réponses sont arrivées). Ne lève jamais d'erreur. */
export async function loadRepoStatus(fetcher: Fetcher, storage: KeyValueStorage, now: number): Promise<RepoStatus> {
  const cached = readCache(storage, now);
  if (cached !== null) return cached;
  const [commitBody, releaseBody] = await Promise.all([
    getJson(fetcher, `/commits/${GITHUB_BRANCH}`),
    getJson(fetcher, "/releases/latest"),
  ]);
  const status: RepoStatus = { commit: parseCommit(commitBody), release: parseRelease(releaseBody) };
  if (status.commit === null && status.release === null) return EMPTY;
  if (status.commit !== null && status.release !== null) {
    try {
      storage.setItem(GITHUB_CACHE_KEY, JSON.stringify({ savedAt: now, ...status }));
    } catch {
      // Stockage plein ou bloqué (navigation privée) : on se passe du cache.
    }
  }
  return status;
}
