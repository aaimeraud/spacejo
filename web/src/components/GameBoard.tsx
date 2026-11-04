// Écran de partie, figé comme un jeu : une scène de taille fixe mise à l'échelle de l'écran (FitStage).
// - Paysage : table ovale inclinée, adversaires assis autour (grilles posées à plat et tournées vers eux,
//   étiquettes redressées face à moi), pioche et défausse au centre, ma grille devant moi sur le bord de la table.
// - Portrait : une bande d'adversaires (un bouton chacun), le jeu de l'un d'eux en grand (celui qui joue, ou
//   celui qu'on a choisi), pioche et défausse, puis ma grille en bas.
// La disposition vient de `game/seats.ts`, le relief du CSS (styles.css, « Table »). Le tableau des scores
// s'ouvre dans une fenêtre (il s'ouvre tout seul en fin de manche).
import { CircleCheck, Crown, Hourglass, ListOrdered, LogOut, Play, Repeat, RotateCcw, SkipForward, Trophy, UserX, Users, WifiOff } from "lucide-react";
import { useEffect, useState, type CSSProperties } from "react";
import type { Action, GameView, PlayerView, PresenceEntry, Waiting } from "../api/types";
import { hint, isMyTurn, playerName, ranking, slotAction, type HeldMode } from "../game/logic";
import { focusedOpponent } from "../game/focus";
import { hiddenCount, seatPlayers } from "../game/seats";
import { ValueCard } from "./Card";
import { FitStage, type Orientation } from "./FitStage";
import { GridView } from "./GridView";
import { Modal } from "./Modal";
import { Scoreboard } from "./Scoreboard";
import { ThemeToggle } from "./ThemeToggle";

interface Props {
  view: GameView;
  /** Code de la partie, rappelé dans la barre du haut. */
  code: string;
  busy: boolean;
  presence: PresenceEntry[];
  waiting: Waiting | null;
  onAction: (action: Action) => void;
  /** L'hôte passe le tour d'un joueur absent. */
  onSkip: (player: number) => void;
  onLeave: () => void;
}

/** Taille des sièges adverses selon le nombre de joueurs (plus il y en a, plus ils sont petits). */
function crowdClass(players: number): string {
  if (players <= 4) return "crowd-few";
  if (players <= 7) return "crowd-some";
  return "crowd-many";
}

/** Badge d'un joueur : nom, rôle, total, cartes cachées, tour. Tout est écrit (jamais la couleur seule). */
function PlayerBadge({
  view,
  player,
  connected,
  mine = false,
}: {
  view: GameView;
  player: PlayerView;
  connected: boolean;
  mine?: boolean;
}) {
  const playing = view.current_player === player.id;
  const hidden = hiddenCount(player.grid);
  return (
    <div className={`badge${playing ? " is-playing" : ""}${player.eliminated ? " is-eliminated" : ""}`}>
      <span className="badge-avatar" aria-hidden="true">
        {player.name.slice(0, 1).toUpperCase()}
      </span>
      <span className="badge-text">
        <strong className="badge-name">
          {player.id === view.host && (
            <>
              <Crown className="inline-icon" aria-hidden="true" />
              <span className="visually-hidden">(hôte) </span>
            </>
          )}
          {player.name}
          {mine && " (vous)"}
        </strong>
        <span className="badge-meta">
          {player.total_score} pts · {hidden} cachée{hidden > 1 ? "s" : ""}
        </span>
      </span>
      {playing && (
        <span className="pill pill-turn">
          <Play aria-hidden="true" /> joue
        </span>
      )}
      {view.finisher === player.id && (
        <span className="pill pill-finisher">
          <CircleCheck aria-hidden="true" /> a terminé
        </span>
      )}
      {player.eliminated ? (
        <span className="pill pill-eliminated">
          {player.abandoned ? <LogOut aria-hidden="true" /> : <UserX aria-hidden="true" />}
          {player.abandoned ? "a abandonné" : "éliminé"}
        </span>
      ) : (
        !connected && (
          <span className="pill pill-absent">
            <WifiOff aria-hidden="true" /> absent
          </span>
        )
      )}
    </div>
  );
}

/** Bandeau « on attend un joueur absent », avec le compte à rebours donné par le serveur. */
/** Bandeau « X est parti·e » : compte à rebours avant que son tour soit passé. L'hôte peut le passer tout
 * de suite, sans attendre. */
function WaitingBanner({
  view,
  waiting,
  busy,
  onSkip,
}: {
  view: GameView;
  waiting: Waiting;
  busy: boolean;
  onSkip: (player: number) => void;
}) {
  const name = playerName(view, waiting.player);
  const last = waiting.missed_turns + 1 >= waiting.max_missed_turns;
  const isHost = view.me === view.host;
  return (
    <div className="waiting-banner" role="status">
      <Hourglass aria-hidden="true" />
      <span>
        <strong>{name} est parti·e</strong> : on l'attend encore {waiting.seconds_left} s.{" "}
        {/* Conséquence détaillée : masquée en portrait, où le bandeau tient sur une ligne. */}
        <span className="waiting-consequence">
          {last
            ? "Sinon, ce sera son dernier tour manqué : il sera éliminé."
            : `Sinon, son tour sera passé (tour manqué ${waiting.missed_turns + 1} sur ${waiting.max_missed_turns}).`}
        </span>
      </span>
      {isHost && (
        <button type="button" className="ghost-button" disabled={busy} onClick={() => onSkip(waiting.player)}>
          <SkipForward aria-hidden="true" />{" "}
          {last ? (
            "Éliminer"
          ) : (
            <>
              Passer<span className="waiting-skip-more"> son tour</span>
            </>
          )}
        </button>
      )}
    </div>
  );
}

export function GameBoard({ view, code, busy, presence, waiting, onAction, onSkip, onLeave }: Props) {
  const connected = (id: number) => presence.find((p) => p.player === id)?.connected ?? true;
  const [mode, setMode] = useState<HeldMode>("swap");
  // Quitter une partie en cours, c'est l'abandonner pour de bon : on demande confirmation (pas après la fin).
  const [confirmingQuit, setConfirmingQuit] = useState(false);
  const quit = () => (view.phase === "GameOver" || me?.eliminated === true ? onLeave() : setConfirmingQuit(true));
  const [scoresOpen, setScoresOpen] = useState(false);
  const seats = seatPlayers(view);
  const me = view.players[view.me];
  const opponents = seats.filter((seat) => seat.index !== 0);
  const myTurn = isMyTurn(view);
  const canTake = myTurn && view.held === null && !busy;
  const heldFromPile = myTurn && view.held?.source === "Pile";
  // Portrait : un seul adversaire en grand. `pinned` = choisi à la main (jusqu'au tour suivant),
  // `lastActive` = dernier adversaire qui a joué (voir game/focus.ts).
  const [pinned, setPinned] = useState<number | null>(null);
  const [lastActive, setLastActive] = useState<number | null>(null);
  useEffect(() => {
    setPinned(null);
    if (view.current_player !== null && view.current_player !== view.me) setLastActive(view.current_player);
  }, [view.current_player, view.me]);
  const focusId = focusedOpponent(
    opponents.map((seat) => seat.player),
    view.current_player,
    pinned,
    lastActive,
  );
  const focused = opponents.find((seat) => seat.player.id === focusId)?.player;

  // En fin de manche, le tableau des scores s'ouvre tout seul.
  useEffect(() => {
    if (view.phase === "RoundOver") setScoresOpen(true);
  }, [view.phase, view.round]);

  const send = (action: Action) => {
    onAction(action);
    // Après un coup, on revient au mode par défaut pour le tour suivant.
    if (action.type === "SwapHeld" || action.type === "DiscardHeldAndReveal") setMode("swap");
    if (action.type === "NextRound") setScoresOpen(false);
  };

  const quitModal = confirmingQuit && (
    <Modal
      title="Abandonner la partie ?"
      onClose={() => setConfirmingQuit(false)}
      footer={
        <>
          <button type="button" className="ghost-button" onClick={() => setConfirmingQuit(false)}>
            Continuer à jouer
          </button>
          <button type="button" className="primary danger" onClick={onLeave}>
            <LogOut aria-hidden="true" /> Abandonner
          </button>
        </>
      }
    >
      <p>
        Si vous quittez maintenant, vous abandonnez la partie : tous les joueurs verront que vous l'avez quittée, et
        vous ne pourrez plus y revenir. La partie continue sans vous.
      </p>
    </Modal>
  );

  const scoresModal = scoresOpen && (
    <Modal
      title={view.phase === "RoundOver" ? `Fin de la manche ${view.round}` : "Tableau des scores"}
      onClose={() => setScoresOpen(false)}
      wide
      footer={
        view.phase === "RoundOver" ? (
          <button type="button" className="primary" onClick={() => send({ type: "NextRound" })} disabled={busy}>
            <Play aria-hidden="true" /> Manche suivante
          </button>
        ) : undefined
      }
    >
      <Scoreboard view={view} />
    </Modal>
  );

  if (view.phase === "GameOver") {
    const winners = view.winners.map((id) => playerName(view, id));
    return (
      <>
        <FitStage landscape={[900, 640]} portrait={[440, 900]}>
          {() => (
            <section className="panel game-over" aria-labelledby="ranking-title">
              <Trophy className="trophy" aria-hidden="true" />
              <h2 id="ranking-title">Fin de partie</h2>
              <p className="winner" role="status">
                {winners.length > 1 ? `Victoire partagée : ${winners.join(", ")}` : `Victoire de ${winners[0] ?? ""}`}
              </p>
              <ol className="ranking">
                {ranking(view).map((player) => (
                  <li key={player.id} className={view.winners.includes(player.id) ? "is-winner" : ""}>
                    <span>{player.name}</span>
                    <strong>{player.total_score} pts</strong>
                    {view.winners.includes(player.id) && <span className="pill pill-turn">gagnant</span>}
                    {player.eliminated && (
                      <span className="pill pill-eliminated">{player.abandoned ? "a abandonné" : "éliminé"}</span>
                    )}
                  </li>
                ))}
              </ol>
              <div className="game-over-actions">
                <button type="button" className="ghost-button" onClick={() => setScoresOpen(true)}>
                  <ListOrdered aria-hidden="true" /> Tableau des scores
                </button>
                <button type="button" className="primary" onClick={onLeave}>
                  <LogOut aria-hidden="true" /> Retour à l'accueil
                </button>
              </div>
            </section>
          )}
        </FitStage>
        {scoresModal}
        {quitModal}
      </>
    );
  }

  const center = (
    <div className="table-center">
      {view.phase === "RoundOver" ? (
        <div className="round-over">
          <p>Manche {view.round} terminée</p>
          <button type="button" className="primary" onClick={() => setScoresOpen(true)}>
            <ListOrdered aria-hidden="true" /> Voir les scores
          </button>
        </div>
      ) : (
        <>
          <button
            type="button"
            className="pile"
            onClick={() => send({ type: "DrawFromPile" })}
            disabled={!canTake}
            aria-label={`Piocher (${view.draw_pile_count} cartes)`}
          >
            <span className="pile-stack" aria-hidden="true">
              <ValueCard value={null} />
            </span>
            <span className="pile-label">Pioche · {view.draw_pile_count}</span>
          </button>
          <button
            type="button"
            className="pile"
            onClick={() => send({ type: "TakeDiscard" })}
            disabled={!canTake || view.discard_top === null}
            aria-label={
              view.discard_top === null ? "Défausse vide" : `Prendre la carte ${view.discard_top} de la défausse`
            }
          >
            {view.discard_top === null ? (
              <span className="card3d card3d-lg card-empty" aria-hidden="true" />
            ) : (
              // `key` sur le nombre de cartes : chaque carte posée sur la défausse rejoue l'animation.
              <span key={view.discard_count} className="discard-top" aria-hidden="true">
                <ValueCard value={view.discard_top} />
              </span>
            )}
            <span className="pile-label">Défausse · {view.discard_count}</span>
          </button>
          {view.held !== null && (
            <div className="held" role="status">
              <span className="held-card">
                <ValueCard value={view.held.value} />
              </span>
              <span className="pile-label">
                {myTurn ? "Votre carte" : `Carte de ${playerName(view, view.current_player)}`}
                {view.held.value !== null && ` : ${view.held.value}`}
              </span>
            </div>
          )}
        </>
      )}
    </div>
  );

  // Paysage : chaque adversaire assis autour de la table.
  const opponentSeats = (orientation: Orientation) =>
    opponents.map((seat) => (
      <section
        key={seat.player.id}
        className={`seat${view.current_player === seat.player.id ? " is-active" : ""}${seat.player.eliminated ? " is-eliminated" : ""}`}
        style={
          orientation === "landscape"
            ? ({
                "--seat-x": `${seat.x}%`,
                "--seat-y": `${seat.y}%`,
                // Un siège agrandi grandit vers le centre de la table (son bord extérieur reste en place).
                transformOrigin: `${seat.x}% ${seat.y}%`,
              } as CSSProperties)
            : undefined
        }
        aria-label={`Place de ${seat.player.name}`}
      >
        {/* Paysage : les cartes sont posées à plat devant leur propriétaire et tournées vers lui, comme sur
            une vraie table ; l'étiquette, elle, se redresse face à moi pour rester lisible. */}
        <div
          className="seat-grid"
          style={{ "--seat-rotation": orientation === "landscape" ? `${seat.rotation}deg` : "0deg" } as CSSProperties}
        >
          <GridView player={seat.player} size="sm" />
        </div>
        <div className="seat-label">
          <PlayerBadge view={view} player={seat.player} connected={connected(seat.player.id)} />
        </div>
      </section>
    ));

  // Portrait : la bande des adversaires (un bouton par adversaire pour voir son jeu), puis le jeu choisi en grand.
  const opponentStrip = opponents.length > 1 && (
    <nav className="opponent-strip" aria-label="Adversaires">
      {opponents.map(({ player }) => {
        const playing = view.current_player === player.id;
        const absent = !player.eliminated && !connected(player.id);
        const status = [
          playing && "joue",
          absent && "absent",
          player.eliminated && (player.abandoned ? "a abandonné" : "éliminé"),
        ]
          .filter(Boolean)
          .join(", ");
        return (
          <button
            key={player.id}
            type="button"
            className={`opponent-chip${playing ? " is-playing" : ""}${player.eliminated ? " is-eliminated" : ""}`}
            aria-pressed={player.id === focusId}
            aria-label={`Voir le jeu de ${player.name}, ${player.total_score} points${status ? `, ${status}` : ""}`}
            onClick={() => setPinned(player.id)}
          >
            <span className="opponent-chip-avatar" aria-hidden="true">
              {player.name.slice(0, 1).toUpperCase()}
              {playing && <Play className="opponent-chip-mark" />}
              {absent && <WifiOff className="opponent-chip-mark is-absent" />}
              {player.eliminated && <UserX className="opponent-chip-mark is-absent" />}
            </span>
            <span className="opponent-chip-name" aria-hidden="true">
              {player.name}
            </span>
            <span className="opponent-chip-score" aria-hidden="true">
              {player.total_score} pts
            </span>
          </button>
        );
      })}
    </nav>
  );

  const focusSeat = focused !== undefined && (
    // `key` : changer d'adversaire rejoue la petite animation d'apparition.
    <section
      key={focused.id}
      className={`focus-seat${view.current_player === focused.id ? " is-active" : ""}${focused.eliminated ? " is-eliminated" : ""}`}
      aria-label={`Jeu de ${focused.name}`}
    >
      <PlayerBadge view={view} player={focused} connected={connected(focused.id)} />
      <GridView player={focused} size="md" />
    </section>
  );

  const mySeat = me !== undefined && (
    <section className={`my-seat${myTurn ? " is-active" : ""}`} aria-labelledby="my-grid-title">
      <h2 id="my-grid-title" className="visually-hidden">
        Ma grille
      </h2>
      <div className="my-side">
        <PlayerBadge view={view} player={me} connected mine />
        {heldFromPile && (
          // Deux tuiles à bascule (aria-pressed) plutôt que des boutons radio : même style que les préréglages.
          <div className="held-choice" role="group" aria-labelledby="held-choice-title">
            <p id="held-choice-title" className="held-choice-title">
              Que faire de la carte piochée ?
            </p>
            <div className="held-choice-tiles">
              <button
                type="button"
                className={`choice-tile${mode === "swap" ? " is-selected" : ""}`}
                aria-pressed={mode === "swap"}
                onClick={() => setMode("swap")}
              >
                <Repeat className="choice-icon" aria-hidden="true" />
                <span className="choice-text">
                  <span className="choice-title">Échanger</span>
                  <span className="choice-description">Elle remplace une carte de ma grille.</span>
                </span>
              </button>
              <button
                type="button"
                className={`choice-tile${mode === "reveal" ? " is-selected" : ""}`}
                aria-pressed={mode === "reveal"}
                onClick={() => setMode("reveal")}
              >
                <RotateCcw className="choice-icon" aria-hidden="true" />
                <span className="choice-text">
                  <span className="choice-title">Défausser et retourner</span>
                  <span className="choice-description">Elle part ; je retourne une carte cachée.</span>
                </span>
              </button>
            </div>
          </div>
        )}
      </div>
      <GridView
        player={me}
        size="md"
        actionFor={(column, row) => slotAction(view, mode, column, row)}
        onAction={send}
        disabled={busy}
      />
    </section>
  );

  return (
    <>
      <FitStage landscape={[1280, 900]} portrait={[480, 1000]}>
        {(orientation) => (
          <div className={`game-stage ${crowdClass(view.players.length)}${waiting !== null ? " has-waiting" : ""}`}>
            <header className="stage-bar">
              <p className="stage-info">
                <span className="stage-code">
                  Partie <strong>{code}</strong> ·{" "}
                </span>
                <Users aria-hidden="true" /> {view.players.length} joueurs · Manche {view.round} · fin à{" "}
                {view.rules.end_score} pts
              </p>
              <p className="hint" role="status" aria-live="polite">
                {me?.eliminated === true
                  ? "Vous avez été éliminé·e : vous pouvez regarder la suite de la partie."
                  : hint(view, mode)}
              </p>
              <div className="stage-actions">
                {/* En portrait, les icônes seules (le libellé reste lu par les lecteurs d'écran). */}
                <button type="button" className="ghost-button" onClick={() => setScoresOpen(true)} aria-label="Scores">
                  <ListOrdered aria-hidden="true" />
                  <span className="stage-action-label">Scores</span>
                </button>
                <ThemeToggle />
                <button type="button" className="ghost-button" onClick={quit} aria-label="Quitter la partie">
                  <LogOut aria-hidden="true" />
                  <span className="stage-action-label">Quitter</span>
                </button>
              </div>
            </header>
            {waiting !== null && <WaitingBanner view={view} waiting={waiting} busy={busy} onSkip={onSkip} />}

            {orientation === "landscape" ? (
              <div className="scene">
                <div className="table" aria-label="Table de jeu" role="region">
                  {opponentSeats(orientation)}
                  {center}
                </div>
              </div>
            ) : (
              <div className="flat-table" aria-label="Table de jeu" role="region">
                {opponentStrip}
                {focusSeat}
                {center}
              </div>
            )}
            {mySeat}
          </div>
        )}
      </FitStage>
      {scoresModal}
        {quitModal}
    </>
  );
}
