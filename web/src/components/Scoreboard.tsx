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
            {round.scores.map((score) => (
              <td key={score.player}>
                {score.score}
                {score.penalized && (
                  <span className="penalty">
                    {" "}
                    (doublé, {score.raw} avant pénalité)
                  </span>
                )}
                {round.finisher === score.player && <span className="note"> · a terminé</span>}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
      <tfoot>
        <tr>
          <th scope="row">Total</th>
          {view.players.map((player) => (
            <td key={player.id}>
              <strong>{player.total_score}</strong>
            </td>
          ))}
        </tr>
      </tfoot>
    </table>
  );
}
