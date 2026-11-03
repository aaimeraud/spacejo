// Salon : code à partager, joueurs, règles actives ; l'hôte peut modifier les réglages et lancer.
import type { GameView, Rules } from "../api/types";
import { RulesEditor, RulesSummary } from "./RulesPanel";

interface Props {
  view: GameView;
  code: string;
  busy: boolean;
  onRulesChange: (rules: Rules) => void;
  onStart: () => void;
}

export function Lobby({ view, code, busy, onRulesChange, onStart }: Props) {
  const isHost = view.me === view.host;
  const enoughPlayers = view.players.length >= 2;
  return (
    <div className="lobby">
      <section className="panel" aria-labelledby="code-title">
        <h2 id="code-title">Code de la partie</h2>
        <p className="game-code" aria-label={`Code : ${code.split("").join(" ")}`}>
          {code}
        </p>
        <p>Partagez ce code avec les autres joueurs (2 à 4 joueurs).</p>
        <button
          type="button"
          onClick={() => {
            void navigator.clipboard?.writeText(code);
          }}
        >
          Copier le code
        </button>
      </section>

      <section className="panel" aria-labelledby="players-title">
        <h2 id="players-title">Joueurs ({view.players.length})</h2>
        <ul>
          {view.players.map((player) => (
            <li key={player.id}>
              {player.name}
              {player.id === view.host && " (hôte)"}
              {player.id === view.me && " (vous)"}
            </li>
          ))}
        </ul>
        {isHost ? (
          <>
            <button type="button" className="primary" onClick={onStart} disabled={busy || !enoughPlayers}>
              Lancer la partie
            </button>
            {!enoughPlayers && <p className="note">Il faut au moins 2 joueurs.</p>}
          </>
        ) : (
          <p>En attente du lancement par l'hôte.</p>
        )}
      </section>

      <section className="panel rules-panel" aria-labelledby="rules-title">
        <h2 id="rules-title">Règles actives</h2>
        {isHost ? (
          <RulesEditor rules={view.rules} onChange={onRulesChange} disabled={busy} />
        ) : (
          <RulesSummary rules={view.rules} />
        )}
      </section>
    </div>
  );
}
