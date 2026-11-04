// Avis de changement poussés par le serveur (server-sent events, route `GET /api/events`, voir
// docs/CONTRACT.md). Le navigateur garde une connexion ouverte et reçoit le numéro de chaque nouvelle
// version de la partie ; il relit alors l'état aussitôt, au lieu d'attendre le prochain polling.
// `EventSource` se reconnecte tout seul si la connexion tombe.

export interface GameEventHandlers {
  /** Une nouvelle version de la partie existe. */
  onVersion: (version: number) => void;
  /** Le flux est ouvert (`true`) ou interrompu (`false`, en attendant la reconnexion automatique). */
  onLive: (live: boolean) => void;
}

/** Écoute les avis de la partie `code`. Renvoie la fonction qui ferme le flux. Sans `EventSource`
 * (navigateur très ancien, tests), ne fait rien : le polling suffit. */
export function listenGameEvents(code: string, handlers: GameEventHandlers): () => void {
  if (typeof EventSource === "undefined") return () => undefined;
  const source = new EventSource(`/api/events?code=${encodeURIComponent(code)}`);
  source.addEventListener("open", () => handlers.onLive(true));
  source.addEventListener("error", () => handlers.onLive(false));
  source.addEventListener("version", (event) => {
    const version = Number((event as MessageEvent<string>).data);
    if (Number.isInteger(version)) handlers.onVersion(version);
  });
  return () => source.close();
}
