// Écran de partie, vu comme une table de jeu : les adversaires assis autour (grilles posées sur une table
// inclinée en perspective), pioche et défausse au centre, ma grille devant moi en bas ; puis le classement
// en fin de partie. La disposition vient de `game/seats.ts`, le relief du CSS (styles.css, « Table »).
import { useState, type CSSProperties } from "react";
import type { Action, GameView, PlayerView } from "../api/types";
import { hint, isMyTurn, playerName, ranking, slotAction, type HeldMode } from "../game/logic";
import { hiddenCount, seatPlayers } from "../game/seats";
import { ValueCard } from "./Card";
import { GridView } from "./GridView";
import { Scoreboard } from "./Scoreboard";

interface Props {
  view: GameView;
  busy: boolean;
  onAction: (action: Action) => void;
}

/** Badge d'un joueur : nom, rôle, total, cartes cachées, tour. Tout est écrit (jamais la couleur seule). */
function PlayerBadge({ view, player, mine = false }: { view: GameView; player: PlayerView; mine?: boolean }) {
  const playing = view.current_player === player.id;
  return (
    <div className={`badge${playing ? " is-playing" : ""}`}>
      <span className="badge-avatar" aria-hidden="true">
        {player.name.slice(0, 1).toUpperCase()}
      </span>
      <span className="badge-text">
        <strong className="badge-name">
          {player.name}
          {mine && " (vous)"}
        </strong>
        <span className="badge-meta">
          {player.total_score} pts · {hiddenCount(player.grid)} cachée{hiddenCount(player.grid) > 1 ? "s" : ""}
          {player.id === view.host && " · hôte"}
        </span>
      </span>
      {playing && <span className="pill pill-turn">joue</span>}
      {view.finisher === player.id && <span className="pill pill-finisher">a terminé</span>}
    </div>
  );
}

export function GameBoard({ view, busy, onAction }: Props) {
  const [mode, setMode] = useState<HeldMode>("swap");
  const seats = seatPlayers(view);
  const me = view.players[view.me];
  const opponents = seats.filter((seat) => seat.index !== 0);
  const myTurn = isMyTurn(view);
  const canTake = myTurn && view.held === null && !busy;
  const heldFromPile = myTurn && view.held?.source === "Pile";

  const send = (action: Action) => {
    onAction(action);
    // Après un coup, on revient au mode par défaut pour le tour suivant.
    if (action.type === "SwapHeld" || action.type === "DiscardHeldAndReveal") setMode("swap");
  };

  if (view.phase === "GameOver") {
    const winners = view.winners.map((id) => playerName(view, id));
    return (
      <div className="game-over">
        <section className="panel" aria-labelledby="ranking-title">
          <h2 id="ranking-title">Fin de partie</h2>
          <p className="winner" role="status">
            {winners.length > 1 ? `Victoire partagée : ${winners.join(", ")}` : `Victoire de ${winners[0] ?? ""}`}
          </p>
          <ol className="ranking">
            {ranking(view).map((player) => (
              <li key={player.id}>
                {player.name} : {player.total_score} points
                {view.winners.includes(player.id) && " (gagnant)"}
              </li>
            ))}
          </ol>
        </section>
        <Scoreboard view={view} />
      </div>
    );
  }

  return (
    <div className="game">
      <p className="table-info">
        {view.players.length} joueurs · Manche {view.round} · fin à {view.rules.end_score} points
      </p>
      <p className="hint" role="status" aria-live="polite">
        {hint(view, mode)}
      </p>

      <div className="scene">
        <div className="table" aria-label="Table de jeu" role="region">
          {opponents.map((seat) => (
            <section
              key={seat.player.id}
              className={`seat${view.current_player === seat.player.id ? " is-active" : ""}`}
              style={
                {
                  "--seat-x": `${seat.x}%`,
                  "--seat-y": `${seat.y}%`,
                  // Un siège agrandi grandit vers le centre de la table (son bord extérieur reste en place).
                  transformOrigin: `${seat.x}% ${seat.y}%`,
                } as CSSProperties
              }
              aria-label={`Place de ${seat.player.name}`}
            >
              <PlayerBadge view={view} player={seat.player} />
              {/* Les cartes sont tournées vers leur propriétaire, comme posées devant lui. */}
                <div className="seat-grid" style={{ "--seat-rotation": `${seat.rotation}deg` } as CSSProperties}>
                  <GridView player={seat.player} size="sm" />
                </div>
            </section>
          ))}

          <div className="table-center">
            {view.phase === "RoundOver" ? (
              <section className="round-over" aria-labelledby="round-over-title">
                <h2 id="round-over-title">Fin de la manche {view.round}</h2>
                <p>Toutes les cartes sont retournées. Scores dans le tableau ci-dessous.</p>
                <button type="button" className="primary" onClick={() => send({ type: "NextRound" })} disabled={busy}>
                  Manche suivante
                </button>
              </section>
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
                    view.discard_top === null
                      ? "Défausse vide"
                      : `Prendre la carte ${view.discard_top} de la défausse`
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
                      {view.held.source === "Discard" ? " (défausse)" : " (pioche)"}
                    </span>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      </div>

      {me !== undefined && (
        <div className="my-seat-scene">
          <section className={`my-seat${myTurn ? " is-active" : ""}`} aria-labelledby="my-grid-title">
            <h2 id="my-grid-title" className="visually-hidden">
              Ma grille
            </h2>
            <PlayerBadge view={view} player={me} mine />
            {heldFromPile && (
              <fieldset className="mode">
                <legend>Que faire de la carte piochée ?</legend>
                <label>
                  <input type="radio" name="mode" checked={mode === "swap"} onChange={() => setMode("swap")} />
                  L'échanger avec une carte de ma grille
                </label>
                <label>
                  <input type="radio" name="mode" checked={mode === "reveal"} onChange={() => setMode("reveal")} />
                  La défausser et retourner une carte cachée
                </label>
              </fieldset>
            )}
            <GridView
              player={me}
              size="md"
              actionFor={(column, row) => slotAction(view, mode, column, row)}
              onAction={send}
              disabled={busy}
            />
          </section>
        </div>
      )}

      <details className="scores" open={view.phase === "RoundOver"}>
        <summary>Tableau des scores</summary>
        <Scoreboard view={view} />
      </details>
    </div>
  );
}
