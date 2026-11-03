// Session du joueur : code de partie, jeton secret et identifiant, gardés dans le navigateur pour
// survivre à un rechargement de page. Le jeton n'est jamais affiché.
import { SESSION_STORAGE_KEY } from "./config";

export interface Session {
  code: string;
  token: string;
  playerId: number;
}

function isSession(value: unknown): value is Session {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.code === "string" && typeof v.token === "string" && typeof v.playerId === "number";
}

/** Stockage minimal (interface de `sessionStorage`), remplaçable dans les tests. */
export interface KeyValueStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export function loadSession(storage: KeyValueStorage): Session | null {
  try {
    const raw = storage.getItem(SESSION_STORAGE_KEY);
    if (raw === null) return null;
    const parsed: unknown = JSON.parse(raw);
    return isSession(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function saveSession(storage: KeyValueStorage, session: Session): void {
  try {
    storage.setItem(SESSION_STORAGE_KEY, JSON.stringify(session));
  } catch {
    // Stockage indisponible (navigation privée…) : la partie continue, sans reprise après rechargement.
  }
}

export function clearSession(storage: KeyValueStorage): void {
  try {
    storage.removeItem(SESSION_STORAGE_KEY);
  } catch {
    // Rien à faire.
  }
}
