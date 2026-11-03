// Scène à taille fixe, mise à l'échelle pour tenir exactement dans la place disponible : comme dans un jeu,
// rien ne défile et tout reste visible, quelle que soit la taille de l'écran. Deux formats de référence :
// paysage (ordinateur, tablette couchée) et portrait (téléphone).
import { useLayoutEffect, useRef, useState, type ReactNode } from "react";

export type Orientation = "landscape" | "portrait";

interface Props {
  /** Taille de référence en paysage, en pixels CSS. */
  landscape: [number, number];
  /** Taille de référence en portrait. */
  portrait: [number, number];
  /** Contenu, dessiné à la taille de référence ; reçoit l'orientation choisie. */
  children: (orientation: Orientation) => ReactNode;
}

/** En dessous de ce rapport largeur / hauteur, on prend le format portrait. */
const PORTRAIT_BELOW_RATIO = 0.9;

export function FitStage({ landscape, portrait, children }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ width: 0, height: 0 });

  // Mesure la place disponible, et la remesure à chaque changement de taille (rotation, fenêtre…).
  useLayoutEffect(() => {
    const element = host.current;
    if (element === null) return;
    const measure = () => setBox({ width: element.clientWidth, height: element.clientHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const orientation: Orientation =
    box.height > 0 && box.width / box.height < PORTRAIT_BELOW_RATIO ? "portrait" : "landscape";
  const [width, height] = orientation === "portrait" ? portrait : landscape;
  const scale = box.width > 0 ? Math.min(box.width / width, box.height / height) : 1;

  return (
    <div ref={host} className="fit-host">
      <div
        className={`fit-stage is-${orientation}`}
        style={{ width, height, transform: `translate(-50%, -50%) scale(${scale})` }}
      >
        {children(orientation)}
      </div>
    </div>
  );
}
