// Application : choisit l'écran selon la session et la phase, et tient l'état reçu du serveur.
// Le serveur est autoritaire : on n'envoie que des intentions et on affiche la vue renvoyée.
import { useCallback, useEffect, useRef, useState } from "react";
import { ApiError, SpaceJoApi } from "./api/client";
import { listenGameEvents } from "./api/events";
import { pollInterval } from "./polling";
import type { Action, GameView, PresenceEntry, PresenceInfo, Rules, Waiting } from "./api/types";
import { codeFromSearch } from "./invite";
import { GameBoard } from "./components/GameBoard";
import { Home } from "./components/Home";
import { Lobby } from "./components/Lobby";
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

/** Code d'un lien d'invitation (`/?code=…`), lu une fois au chargement de la page. */
const inviteCode = codeFromSearch(window.location.search);

/** Session de départ : celle de l'onglet ; sinon, si le lien d'invitation mène à une partie déjà rejointe sur
 * cet appareil (l'hôte qui ouvre son propre lien, par exemple), on la reprend au lieu de la rejoindre à nouveau. */
function initialSession(): Session | null {
  const current = loadSession(window.sessionStorage);
  if (current !== null || inviteCode === null) return current;
  const known = knownSessions(window.localStorage).find((s) => s.code === inviteCode);
  if (known === undefined) return null;
  const resumed = { code: known.code, token: known.token, playerId: known.playerId };
  saveSession(window.sessionStorage, resumed);
  return resumed;
}

export function App() {
  const [session, setSession] = useState<Session | null>(initialSession);
  // Le code d'invitation ne sert qu'une fois : on le retire de l'adresse, pour qu'un rechargement ou un
  // retour à l'accueil ne rouvre pas la jonction.
  useEffect(() => {
    if (inviteCode !== null) window.history.replaceState(null, "", window.location.pathname);
  }, []);
  // Code d'invitation pas encore utilisé : oublié dès qu'on entre dans une partie ou qu'on la quitte.
  const [pendingInvite, setPendingInvite] = useState<string | null>(inviteCode);
  const [isPublic, setIsPublic] = useState(false);
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
  // Phase connue, pour régler le rythme du polling sans le relancer à chaque changement de vue.
  const phaseRef = useRef<GameView["phase"] | null>(null);
  phaseRef.current = view?.phase ?? null;

  // Réveil du serveur dès l'ouverture : l'hébergement gratuit (Render) l'endort après 15 minutes sans
  // requête, et le réveil prend environ une minute. Ainsi, il démarre pendant qu'on lit l'accueil et qu'on
  // tape son nom, plutôt qu'au clic sur « Créer la partie ». Un échec ici est sans importance.
  useEffect(() => {
    api.health().catch(() => undefined);
  }, []);

  /** Revient à l'accueil. `forget` : la partie n'est plus proposée à la reprise. */
  const leave = useCallback((forget: boolean) => {
    const current = loadSession(window.sessionStorage);
    if (forget && current !== null) forgetSession(window.localStorage, current.code);
    clearSession(window.sessionStorage);
    setPendingInvite(null);
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
    setIsPublic(info.public);
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

  // Lecture de l'état tant qu'une session est ouverte :
  // - aussitôt qu'un avis du serveur annonce une nouvelle version (api/events.ts) ;
  // - et par polling, à un rythme qui dépend de la phase, de la visibilité de l'onglet et du flux d'avis
  //   (polling.ts). Chaque lecture programme la suivante ; revenir sur l'onglet relit aussitôt.
  useEffect(() => {
    if (session === null) return;
    let stopped = false;
    let timer: number | undefined;
    let live = false;
    // Une seule lecture à la fois : un avis arrivé pendant une lecture en relance une juste après.
    let reading = false;
    let readAgain = false;
    const schedule = () => {
      window.clearTimeout(timer);
      if (!stopped) {
        timer = window.setTimeout(() => void tick(), pollInterval(document.hidden, phaseRef.current, live));
      }
    };
    const tick = async () => {
      if (reading) {
        readAgain = true;
        return;
      }
      reading = true;
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
      } finally {
        reading = false;
        if (readAgain && !stopped) {
          readAgain = false;
          void tick();
        } else {
          schedule();
        }
      }
    };
    const closeEvents = listenGameEvents(session.code, {
      onVersion: (version) => {
        if (version !== versionRef.current) {
          window.clearTimeout(timer);
          void tick();
        }
      },
      onLive: (open) => {
        live = open;
        // Flux rouvert après une coupure : on rattrape ce qui a pu se passer entre-temps.
        if (open) void tick();
      },
    });
    const onVisibility = () => {
      if (!document.hidden) {
        window.clearTimeout(timer);
        void tick();
      }
    };
    document.addEventListener("visibilitychange", onVisibility);
    void tick();
    return () => {
      stopped = true;
      window.clearTimeout(timer);
      closeEvents();
      document.removeEventListener("visibilitychange", onVisibility);
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
    setPendingInvite(null);
    setSession(created);
  };

  const onCreate = (name: string, rules: Rules, publicGame: boolean) =>
    void run(async () => {
      startSession(name.trim(), await api.create(name, rules, publicGame));
    });

  const loadPublicGames = useCallback(async () => (await api.publicGames()).games, []);

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
  const onSkip = (player: number) => write((s, version) => api.skip(s.code, s.token, version, player));

  return (
    // L'application occupe exactement l'écran : l'écran courant, mis à l'échelle pour tenir dans la place
    // disponible (aucun défilement de la page). Pas de barre de navigation : il n'y a nulle part où aller ;
    // le code de la partie et « Quitter » sont sur l'écran du salon et de la partie.
    <div className="app">

      <main className="app-main">
        {error !== null && (
          <p className="toast" role="alert">
            {error}
            <button type="button" className="toast-close" onClick={() => setError(null)} aria-label="Fermer le message">
              ×
            </button>
          </p>
        )}
        {session === null && (
          <Home
            busy={busy}
            resumable={resumable}
            initialCode={pendingInvite}
            loadPublicGames={loadPublicGames}
            onCreate={onCreate}
            onJoin={onJoin}
            onResume={onResume}
          />
        )}
        {session !== null && view === null && (
          <p className="loading" role="status">
            Chargement de la partie…
          </p>
        )}
        {session !== null && view !== null && view.phase === "Lobby" && (
          <Lobby
            view={view}
            code={session.code}
            isPublic={isPublic}
            busy={busy}
            presence={presence}
            onRulesChange={onRulesChange}
            onStart={onStart}
            onLeave={() => leave(true)}
          />
        )}
        {session !== null && view !== null && view.phase !== "Lobby" && (
          <GameBoard
            view={view}
            code={session.code}
            busy={busy}
            presence={presence}
            waiting={waiting}
            onAction={onAction}
            onSkip={onSkip}
            onLeave={() => leave(true)}
          />
        )}
      </main>
    </div>
  );
}
