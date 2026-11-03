// Session du joueur : code de partie, jeton secret et identifiant, gardés dans le navigateur pour
// survivre à un rechargement de page. Le jeton n'est jamais affiché.
import { KNOWN_SESSIONS_KEY, MAX_KNOWN_SESSIONS, SESSION_STORAGE_KEY } from "./config";

export interface Session {
  code: string;
  token: string;
  playerId: number;
}

/** Partie mémorisée sur cet appareil (pour la reprendre après avoir fermé l'onglet). */
export interface KnownSession extends Session {
  name: string;
  savedAt: number;
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

function isKnownSession(value: unknown): value is KnownSession {
  if (!isSession(value)) return false;
  const v = value as unknown as Record<string, unknown>;
  return typeof v.name === "string" && typeof v.savedAt === "number";
}

/** Parties mémorisées, de la plus récente à la plus ancienne. */
export function knownSessions(storage: KeyValueStorage): KnownSession[] {
  try {
    const raw = storage.getItem(KNOWN_SESSIONS_KEY);
    if (raw === null) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter(isKnownSession).sort((a, b) => b.savedAt - a.savedAt) : [];
  } catch {
    return [];
  }
}

/** Mémorise (ou rafraîchit) une partie. `now` est passé en paramètre pour des tests déterministes. */
export function rememberSession(storage: KeyValueStorage, session: Session, name: string, now: number): void {
  const others = knownSessions(storage).filter((s) => s.code !== session.code);
  const list = [{ ...session, name, savedAt: now }, ...others].slice(0, MAX_KNOWN_SESSIONS);
  try {
    storage.setItem(KNOWN_SESSIONS_KEY, JSON.stringify(list));
  } catch {
    // Pas de mémorisation possible : on ne pourra simplement pas reprendre cette partie.
  }
}

/** Oublie une partie (quittée, terminée, ou refusée par le serveur). */
export function forgetSession(storage: KeyValueStorage, code: string): void {
  const list = knownSessions(storage).filter((s) => s.code !== code);
  try {
    storage.setItem(KNOWN_SESSIONS_KEY, JSON.stringify(list));
  } catch {
    // Rien à faire.
  }
}

