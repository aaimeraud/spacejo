// Place des joueurs autour de la table. Indépendant du rendu (CSS 3D aujourd'hui, peut-être
// Three Fiber demain : voir docs/decisions/2026-11-03-vue-table-css-3d.md).
import type { ColumnView, GameView, PlayerView } from "../api/types";

/** Un joueur et sa place autour de la table. */
export interface Seat {
  player: PlayerView;
  /** 0 = moi (en bas), puis les autres dans l'ordre du tour. */
  index: number;
  /** Angle en degrés, repère de l'écran (y vers le bas) : 90 = en bas, 180 = à gauche, 270 = en haut,
   * 0 = à droite. Les angles croissants tournent dans le sens des aiguilles d'une montre. */
  angle: number;
  /** Position du centre du siège sur la table, en pourcentage de sa largeur et de sa hauteur. */
  x: number;
  y: number;
  /** Rotation de la grille (degrés, sens des aiguilles d'une montre) pour qu'elle soit tournée vers son
   * propriétaire, comme des cartes posées devant lui : 0 en bas (moi), 180 en face, 90 à gauche, 270 à droite. */
  rotation: number;
}

/** Demi-axes de l'ellipse des sièges, en pourcentage de la table. */
const SEAT_RADIUS_X = 42;
const SEAT_RADIUS_Y = 40;
/** Secteur laissé libre de chaque côté de ma place (en bas), en degrés : ma grille, posée devant moi sur le
 * bord de la table, occupe ce secteur. */
const MY_SECTOR_HALF_WIDTH = 45;

/** Répartit les joueurs autour d'une ellipse : moi en bas, puis chacun dans l'ordre du tour, dans le sens
 * des aiguilles d'une montre. Les adversaires se partagent l'arc qui reste, en parts égales (au centre de
 * chaque part). Fonctionne pour n'importe quel nombre de joueurs. */
export function seatPlayers(view: GameView): Seat[] {
  const count = view.players.length;
  if (count === 0) return [];
  const start = view.players.findIndex((p) => p.id === view.me);
  const first = start < 0 ? 0 : start;
  return view.players.map((_, offset) => {
    const player = view.players[(first + offset) % count] as PlayerView;
    const opponents = count - 1;
    const arc = 360 - 2 * MY_SECTOR_HALF_WIDTH;
    const angle =
      offset === 0 ? 90 : (90 + MY_SECTOR_HALF_WIDTH + ((offset - 0.5) * arc) / opponents) % 360;
    const radians = (angle * Math.PI) / 180;
    return {
      player,
      index: offset,
      angle,
      x: round(50 + SEAT_RADIUS_X * Math.cos(radians)),
      y: round(50 + SEAT_RADIUS_Y * Math.sin(radians)),
      rotation: (angle - 90 + 360) % 360,
    };
  });
}

function round(value: number): number {
  return Math.round(value * 10) / 10;
}

/** Nombre de cartes encore face cachée (simple comptage de la vue, pour le badge du joueur). */
export function hiddenCount(grid: ColumnView[]): number {
  return grid.reduce(
    (total, column) =>
      column.kind === "Removed"
        ? total
        : total + column.slots.filter((slot) => slot.kind === "Hidden").length,
    0,
  );
}
