// Application : choisit l'écran selon la session et la phase, et tient l'état reçu du serveur.
// Le serveur est autoritaire : on n'envoie que des intentions et on affiche la vue renvoyée.
import { useCallback, useEffect, useRef, useState } from "react";
import { ApiError, SpaceJoApi } from "./api/client";
import type { Action, GameView, Rules } from "./api/types";
import { GameBoard } from "./components/GameBoard";
import { Home } from "./components/Home";
import { Lobby } from "./components/Lobby";
import { POLL_INTERVAL_MS } from "./config";
import { clearSession, loadSession, saveSession, type Session } from "./session";

const api = new SpaceJoApi();

export function App() {
  const [session, setSession] = useState<Session | null>(() => loadSession(window.sessionStorage));
  const [view, setView] = useState<GameView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // La version connue est lue par le polling et par les actions : une référence évite de
  // relancer le polling à chaque changement.
  const versionRef = useRef<number | null>(null);

  const leave = useCallback(() => {
    clearSession(window.sessionStorage);
    versionRef.current = null;
    setSession(null);
    setView(null);
    setError(null);
  }, []);

  const applyResponse = useCallback((version: number, next: GameView) => {
    versionRef.current = version;
    setView(next);
  }, []);

  /** Relit l'état ; avec `force`, on redemande la vue complète. */
  const refresh = useCallback(
    async (current: Session, force = false) => {
      const known = force ? undefined : (versionRef.current ?? undefined);
      const response = await api.state(current.code, current.token, known);
      if (response.changed) applyResponse(response.version, response.view);
    },
    [applyResponse],
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
          // Jeton refusé ou partie disparue : on revient à l'accueil.
          leave();
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

  const startSession = (code: string, token: string, playerId: number, version: number, next: GameView) => {
    const created = { code, token, playerId };
    saveSession(window.sessionStorage, created);
    applyResponse(version, next);
    setSession(created);
  };

  const onCreate = (name: string, rules: Rules) =>
    void run(async () => {
      const r = await api.create(name, rules);
      startSession(r.code, r.token, r.player_id, r.version, r.view);
    });

  const onJoin = (code: string, name: string) =>
    void run(async () => {
      const r = await api.join(code, name);
      startSession(r.code, r.token, r.player_id, r.version, r.view);
    });

  const write = (send: (s: Session, version: number) => Promise<{ version: number; view: GameView }>) =>
    void run(async () => {
      if (session === null || versionRef.current === null) return;
      const r = await send(session, versionRef.current);
      applyResponse(r.version, r.view);
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
            <button type="button" onClick={leave}>
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
        {session === null && <Home busy={busy} onCreate={onCreate} onJoin={onJoin} />}
        {session !== null && view === null && <p role="status">Chargement de la partie…</p>}
        {session !== null && view !== null && view.phase === "Lobby" && (
          <Lobby view={view} code={session.code} busy={busy} onRulesChange={onRulesChange} onStart={onStart} />
        )}
        {session !== null && view !== null && view.phase !== "Lobby" && (
          <GameBoard view={view} busy={busy} onAction={onAction} />
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
