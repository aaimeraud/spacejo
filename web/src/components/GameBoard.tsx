// Écran de partie : ma grille, les grilles des autres, pioche, défausse, tour, scores ;
// puis le classement en fin de partie.
import { useState } from "react";
import type { Action, GameView } from "../api/types";
import { hint, isMyTurn, playerName, ranking, slotAction, type HeldMode } from "../game/logic";
import { ValueCard } from "./Card";
import { GridView } from "./GridView";
import { Scoreboard } from "./Scoreboard";

interface Props {
  view: GameView;
  busy: boolean;
  onAction: (action: Action) => void;
}

export function GameBoard({ view, busy, onAction }: Props) {
  const [mode, setMode] = useState<HeldMode>("swap");
  const me = view.players[view.me];
  const others = view.players.filter((player) => player.id !== view.me);
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
      <p className="hint" role="status" aria-live="polite">
        Manche {view.round} · {hint(view, mode)}
      </p>

      <section className="table-center panel" aria-label="Pioche et défausse">
        <div className="pile">
          <button type="button" onClick={() => send({ type: "DrawFromPile" })} disabled={!canTake}>
            <ValueCard value={null} />
            <span>Piocher ({view.draw_pile_count})</span>
          </button>
        </div>
        <div className="pile">
          <button
            type="button"
            onClick={() => send({ type: "TakeDiscard" })}
            disabled={!canTake || view.discard_top === null}
            aria-label={
              view.discard_top === null ? "Défausse vide" : `Prendre la carte ${view.discard_top} de la défausse`
            }
          >
            {view.discard_top === null ? <span className="card card-empty" /> : <ValueCard value={view.discard_top} />}
            <span>Défausse ({view.discard_count})</span>
          </button>
        </div>
        {view.held !== null && (
          <div className="held">
            <ValueCard value={view.held.value} />
            <span>
              {myTurn ? "Votre carte en main" : `Carte en main de ${playerName(view, view.current_player)}`}
              {view.held.value !== null && ` : ${view.held.value}`}
              {view.held.source === "Discard" ? " (prise sur la défausse)" : " (piochée)"}
            </span>
          </div>
        )}
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
      </section>

      {me !== undefined && (
        <section className={`panel player mine${myTurn ? " active" : ""}`} aria-labelledby="my-grid-title">
          <h2 id="my-grid-title">
            Ma grille ({me.name}){myTurn && " · à vous de jouer"}
          </h2>
          <GridView
            player={me}
            actionFor={(column, row) => slotAction(view, mode, column, row)}
            onAction={send}
            disabled={busy}
          />
        </section>
      )}

      <div className="others">
        {others.map((player) => (
          <section
            key={player.id}
            className={`panel player${view.current_player === player.id ? " active" : ""}`}
            aria-labelledby={`grid-title-${player.id}`}
          >
            <h2 id={`grid-title-${player.id}`}>
              {player.name}
              {view.current_player === player.id && " · joue"}
              {view.finisher === player.id && " · a terminé"}
            </h2>
            <GridView player={player} />
          </section>
        ))}
      </div>

      {view.phase === "RoundOver" && (
        <section className="panel round-over" aria-labelledby="round-over-title">
          <h2 id="round-over-title">Fin de la manche {view.round}</h2>
          <button type="button" className="primary" onClick={() => send({ type: "NextRound" })} disabled={busy}>
            Manche suivante
          </button>
        </section>
      )}

      <Scoreboard view={view} />
    </div>
  );
}
