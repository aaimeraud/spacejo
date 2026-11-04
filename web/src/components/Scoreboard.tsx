// Tableau des scores : scores de chaque manche et totaux, tels que calculés par le serveur.
import type { GameView } from "../api/types";

export function Scoreboard({ view }: { view: GameView }) {
  return (
    <table className="scoreboard">
      <caption>Tableau des scores (fin de partie à {view.rules.end_score} points)</caption>
      <thead>
        <tr>
          <th scope="col">Manche</th>
          {view.players.map((player) => (
            <th scope="col" key={player.id}>
              {player.name}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {view.rounds.map((round) => (
          <tr key={round.round}>
            <th scope="row">{round.round}</th>
            {view.players.map((player) => {
              // Un joueur éliminé n'a plus de score dans les manches suivantes.
              const score = round.scores.find((s) => s.player === player.id);
              if (score === undefined) return <td key={player.id}>—</td>;
              return (
                <td key={player.id}>
                  {score.score}
                  {score.penalized && (
                    <span className="penalty">
                      {" "}
                      (doublé, {score.raw} avant pénalité)
                    </span>
                  )}
                  {round.finisher === score.player && <span className="note"> · a terminé</span>}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
      <tfoot>
        <tr>
          <th scope="row">Total</th>
          {view.players.map((player) => (
            <td key={player.id}>
              <strong>{player.total_score}</strong>
              {player.eliminated && (
                <span className="note"> · {player.abandoned ? "a abandonné" : "éliminé"}</span>
              )}
            </td>
          ))}
        </tr>
      </tfoot>
    </table>
  );
}
