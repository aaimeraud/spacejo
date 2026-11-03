// Application : choisit l'écran selon la session et la phase, et tient l'état reçu du serveur.
// Le serveur est autoritaire : on n'envoie que des intentions et on affiche la vue renvoyée.
import { useCallback, useEffect, useRef, useState } from "react";
import { ApiError, SpaceJoApi } from "./api/client";
import type { Action, GameView, PresenceEntry, PresenceInfo, Rules, Waiting } from "./api/types";
import { GameBoard } from "./components/GameBoard";
import { Home } from "./components/Home";
import { Lobby } from "./components/Lobby";
import { POLL_INTERVAL_MS } from "./config";
import {
  clearSession,
  forgetSession,
  knownSessions,
  loadSession,
  rememberSession,
  saveSession,
  type KnownSession,
  type Session,
} from "./session";

const api = new SpaceJoApi();

export function App() {
  const [session, setSession] = useState<Session | null>(() => loadSession(window.sessionStorage));
  const [view, setView] = useState<GameView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [presence, setPresence] = useState<PresenceEntry[]>([]);
  const [waiting, setWaiting] = useState<Waiting | null>(null);
  // Parties mémorisées sur cet appareil, proposées à la reprise sur l'accueil.
  const [resumable, setResumable] = useState<KnownSession[]>(() => knownSessions(window.localStorage));
  // La version connue est lue par le polling et par les actions : une référence évite de
  // relancer le polling à chaque changement.
  const versionRef = useRef<number | null>(null);

  /** Revient à l'accueil. `forget` : la partie n'est plus proposée à la reprise. */
  const leave = useCallback((forget: boolean) => {
    const current = loadSession(window.sessionStorage);
    if (forget && current !== null) forgetSession(window.localStorage, current.code);
    clearSession(window.sessionStorage);
    versionRef.current = null;
    setSession(null);
    setView(null);
    setError(null);
    setPresence([]);
    setWaiting(null);
    setResumable(knownSessions(window.localStorage));
  }, []);

  const applyPresence = useCallback((info: PresenceInfo) => {
    setPresence(info.presence);
    setWaiting(info.waiting);
  }, []);

  const applyResponse = useCallback(
    (version: number, next: GameView, info: PresenceInfo) => {
      versionRef.current = version;
      setView(next);
      applyPresence(info);
    },
    [applyPresence],
  );

  /** Relit l'état ; avec `force`, on redemande la vue complète. */
  const refresh = useCallback(
    async (current: Session, force = false) => {
      const known = force ? undefined : (versionRef.current ?? undefined);
      const response = await api.state(current.code, current.token, known);
      if (response.changed) applyResponse(response.version, response.view, response);
      else applyPresence(response);
    },
    [applyResponse, applyPresence],
  );

  // Polling toutes les POLL_INTERVAL_MS tant qu'une session est ouverte.
  useEffect(() => {
    if (session === null) return;
    let stopped = false;
    const tick = async () => {
      try {
        await refresh(session);
        if (!stopped) setError((previous) => (previous?.startsWith("Connexion") ? null : previous));
      } catch (caught) {
        if (stopped) return;
        if (caught instanceof ApiError && (caught.status === 401 || caught.status === 404)) {
          // Jeton refusé ou partie disparue : on revient à l'accueil et on l'oublie.
          leave(true);
          setError(caught.message);
        } else if (caught instanceof ApiError) {
          setError(`Connexion : ${caught.message}`);
        }
      }
    };
    void tick();
    const timer = window.setInterval(() => void tick(), POLL_INTERVAL_MS);
    return () => {
      stopped = true;
      window.clearInterval(timer);
    };
  }, [session, refresh, leave]);

  /** Exécute une requête d'écriture et gère les erreurs communes. */
  const run = useCallback(
    async (task: () => Promise<void>) => {
      setBusy(true);
      setError(null);
      try {
        await task();
      } catch (caught) {
        if (caught instanceof ApiError) {
          setError(caught.message);
          // Conflit : l'écran était périmé, on relit l'état complet.
          if (caught.isConflict && session !== null) await refresh(session, true).catch(() => undefined);
        } else {
          setError("Erreur inattendue.");
        }
      } finally {
        setBusy(false);
      }
    },
    [refresh, session],
  );

  // Une partie terminée n'est plus proposée à la reprise.
  useEffect(() => {
    if (view?.phase === "GameOver" && session !== null) forgetSession(window.localStorage, session.code);
  }, [view?.phase, session]);

  const startSession = (name: string, r: { code: string; token: string; player_id: number; version: number; view: GameView } & PresenceInfo) => {
    const created = { code: r.code, token: r.token, playerId: r.player_id };
    saveSession(window.sessionStorage, created);
    // Mémorisée sur l'appareil : on pourra revenir dans la partie après avoir fermé l'onglet.
    rememberSession(window.localStorage, created, name, Date.now());
    applyResponse(r.version, r.view, r);
    setSession(created);
  };

  const onCreate = (name: string, rules: Rules) =>
    void run(async () => {
      startSession(name.trim(), await api.create(name, rules));
    });

  const onJoin = (code: string, name: string) =>
    void run(async () => {
      startSession(name.trim(), await api.join(code, name));
    });

  /** Reprend une partie mémorisée : le polling recharge aussitôt la vue complète. */
  const onResume = (known: KnownSession) => {
    const resumed = { code: known.code, token: known.token, playerId: known.playerId };
    saveSession(window.sessionStorage, resumed);
    versionRef.current = null;
    setSession(resumed);
  };

  const write = (
    send: (s: Session, version: number) => Promise<{ version: number; view: GameView } & PresenceInfo>,
  ) =>
    void run(async () => {
      if (session === null || versionRef.current === null) return;
      const r = await send(session, versionRef.current);
      applyResponse(r.version, r.view, r);
    });

  const onRulesChange = (rules: Rules) =>
    write((s, version) => api.updateRules(s.code, s.token, version, rules));
  const onStart = () => write((s, version) => api.start(s.code, s.token, version));
  const onAction = (action: Action) => write((s, version) => api.act(s.code, s.token, version, action));

  return (
    <div className="app">
      <header className="app-header">
        <h1>SpaceJo</h1>
        {session !== null && (
          <div className="session-info">
            <span>
              Partie <strong>{session.code}</strong>
            </span>
            <button type="button" onClick={() => leave(true)}>
              Quitter
            </button>
          </div>
        )}
      </header>

      {error !== null && (
        <p className="error" role="alert">
          {error}
        </p>
      )}

      <main>
        {session === null && (
          <Home busy={busy} resumable={resumable} onCreate={onCreate} onJoin={onJoin} onResume={onResume} />
        )}
        {session !== null && view === null && <p role="status">Chargement de la partie…</p>}
        {session !== null && view !== null && view.phase === "Lobby" && (
          <Lobby
            view={view}
            code={session.code}
            busy={busy}
            presence={presence}
            onRulesChange={onRulesChange}
            onStart={onStart}
          />
        )}
        {session !== null && view !== null && view.phase !== "Lobby" && (
          <GameBoard view={view} busy={busy} presence={presence} waiting={waiting} onAction={onAction} />
        )}
      </main>

      <footer className="app-footer">
        <p>
          SpaceJo reprend les règles du jeu de cartes Skyjo. Projet indépendant, non affilié à Magilano.
        </p>
      </footer>
    </div>
  );
}
