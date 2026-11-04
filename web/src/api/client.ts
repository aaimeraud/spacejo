// Client HTTP de l'API (docs/CONTRACT.md). Il n'envoie que des intentions et des réglages ;
// l'état affiché vient toujours du serveur.
import type {
  Action,
  ChatMessages,
  ChatSent,
  ErrorBody,
  JoinedResponse,
  PublicGamesResponse,
  Rules,
  StateResponse,
  WriteResponse,
} from "./types";

export const TOKEN_HEADER = "X-Player-Token";

/** Erreur renvoyée par l'API (ou erreur réseau, avec le code `network`). */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }

  /** La partie a changé entre-temps (409) : il faut relire l'état. */
  get isConflict(): boolean {
    return this.status === 409;
  }
}

function isErrorBody(value: unknown): value is ErrorBody {
  if (typeof value !== "object" || value === null || !("error" in value)) return false;
  const error = (value as { error: unknown }).error;
  return (
    typeof error === "object" &&
    error !== null &&
    typeof (error as { code?: unknown }).code === "string" &&
    typeof (error as { message?: unknown }).message === "string"
  );
}

/** Fonction `fetch` utilisée (remplaçable dans les tests). */
export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export class SpaceJoApi {
  private readonly fetcher: FetchLike;

  constructor(fetcher: FetchLike = (input, init) => fetch(input, init)) {
    this.fetcher = fetcher;
  }

  private async request<T>(path: string, init: RequestInit, token?: string): Promise<T> {
    const headers: Record<string, string> = { "Content-Type": "application/json" };
    if (token !== undefined) headers[TOKEN_HEADER] = token;
    let response: Response;
    try {
      response = await this.fetcher(path, { ...init, headers });
    } catch {
      throw new ApiError(0, "network", "Serveur injoignable. Vérifiez votre connexion.");
    }
    let body: unknown = null;
    try {
      body = await response.json();
    } catch {
      body = null;
    }
    if (!response.ok) {
      if (isErrorBody(body)) {
        throw new ApiError(response.status, body.error.code, body.error.message);
      }
      throw new ApiError(response.status, "unknown", `Erreur inattendue (${response.status}).`);
    }
    return body as T;
  }

  /** Crée une partie ; `isPublic` : elle sera listée sur l'accueil de tous (privée sinon). */
  create(name: string, rules?: Rules, isPublic = false): Promise<JoinedResponse> {
    // Champs facultatifs omis quand ils valent leur défaut côté serveur (règles officielles, partie privée).
    const body: { name: string; rules?: Rules; public?: boolean } = { name };
    if (rules !== undefined) body.rules = rules;
    if (isPublic) body.public = true;
    return this.request("/api/create", { method: "POST", body: JSON.stringify(body) });
  }

  /** L'hôte rend la partie publique ou privée (au salon seulement). */
  setVisibility(code: string, token: string, expectedVersion: number, isPublic: boolean): Promise<WriteResponse> {
    return this.request(
      "/api/visibility",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion, public: isPublic }) },
      token,
    );
  }

  /** Quitter la partie : au salon, on en est retiré ; pendant la partie, on abandonne (vu par tous). */
  leave(code: string, token: string): Promise<{ version: number }> {
    return this.request("/api/leave", { method: "POST", body: JSON.stringify({ code }) }, token);
  }

  /** L'hôte retire un joueur du salon. */
  kick(code: string, token: string, expectedVersion: number, player: number): Promise<WriteResponse> {
    return this.request(
      "/api/kick",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion, player }) },
      token,
    );
  }

  /** Envoie un message dans le chat de la partie. */
  say(code: string, token: string, text: string): Promise<ChatSent> {
    return this.request("/api/chat", { method: "POST", body: JSON.stringify({ code, text }) }, token);
  }

  /** Messages du chat plus récents que `after` (les derniers si `after` vaut 0). */
  messages(code: string, token: string, after: number): Promise<ChatMessages> {
    const params = new URLSearchParams({ code, after: String(after) });
    return this.request(`/api/messages?${params.toString()}`, { method: "GET" }, token);
  }

  /** Parties publiques que l'on peut rejoindre (sans jeton). */
  publicGames(): Promise<PublicGamesResponse> {
    return this.request("/api/public-games", { method: "GET" });
  }

  join(code: string, name: string): Promise<JoinedResponse> {
    return this.request("/api/join", { method: "POST", body: JSON.stringify({ code, name }) });
  }

  updateRules(code: string, token: string, expectedVersion: number, rules: Rules): Promise<WriteResponse> {
    return this.request(
      "/api/rules",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion, rules }) },
      token,
    );
  }

  start(code: string, token: string, expectedVersion: number): Promise<WriteResponse> {
    return this.request(
      "/api/start",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion }) },
      token,
    );
  }

  act(code: string, token: string, expectedVersion: number, action: Action): Promise<WriteResponse> {
    return this.request(
      "/api/action",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion, action }) },
      token,
    );
  }

  /** L'hôte passe le tour du joueur absent qui bloque la partie, sans attendre la fin du délai. */
  skip(code: string, token: string, expectedVersion: number, player: number): Promise<WriteResponse> {
    return this.request(
      "/api/skip",
      { method: "POST", body: JSON.stringify({ code, expected_version: expectedVersion, player }) },
      token,
    );
  }

  /** Contrôle de santé : ne touche pas à la base. Sert aussi à réveiller le serveur (voir App.tsx). */
  health(): Promise<{ status: string }> {
    return this.request("/api/health", { method: "GET" });
  }

  state(code: string, token: string, knownVersion?: number): Promise<StateResponse> {
    const params = new URLSearchParams({ code });
    if (knownVersion !== undefined) params.set("known_version", String(knownVersion));
    return this.request(`/api/state?${params.toString()}`, { method: "GET" }, token);
  }
}
