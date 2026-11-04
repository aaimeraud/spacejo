// Code QR dessiné en SVG à partir de la matrice de `qr.ts`. Toujours sombre sur fond clair, quel que soit le
// thème : beaucoup de lecteurs de codes QR ne savent pas lire un code inversé (clair sur fond sombre).
import { useMemo } from "react";
import { qrMatrix } from "../qr";

/** Marge blanche autour du code, en cases : la norme en demande 4 pour que le code soit bien détecté. */
const QUIET_ZONE = 4;

export function QrCode({ text, label }: { text: string; label: string }) {
  // Calcul refait seulement si le texte change.
  const { size, path } = useMemo(() => {
    const matrix = qrMatrix(text);
    // Un seul chemin SVG, un petit carré par case sombre : bien plus léger qu'un élément par case.
    const squares: string[] = [];
    matrix.forEach((row, y) =>
      row.forEach((dark, x) => {
        if (dark) squares.push(`M${x + QUIET_ZONE} ${y + QUIET_ZONE}h1v1h-1z`);
      }),
    );
    return { size: matrix.length + 2 * QUIET_ZONE, path: squares.join("") };
  }, [text]);

  return (
    <svg
      className="qr-code"
      viewBox={`0 0 ${size} ${size}`}
      role="img"
      aria-label={label}
      shapeRendering="crispEdges"
    >
      <rect width={size} height={size} fill="#ffffff" />
      <path d={path} fill="#000000" />
    </svg>
  );
}
