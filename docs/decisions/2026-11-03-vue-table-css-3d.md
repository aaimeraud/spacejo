# Vue « table de jeu » en CSS 3D

## Contexte

L'interface de la v1 empile les grilles les unes sous les autres, avec des cartes plates. Le propriétaire du
dépôt souhaite une vue où l'on est assis autour d'une table, avec des cartes en relief qui se retournent.
Il veut d'abord essayer en CSS 3D, et pourra demander React Three Fiber (WebGL) ensuite.

## Décision

- Rendu en **CSS 3D** : `perspective` sur la scène, plateau incliné (`rotateX`), cartes à deux faces
  (`backface-visibility: hidden`) retournées par `rotateY(180deg)`, transitions et animations CSS.
- La **logique de placement** (`web/src/game/seats.ts`) est séparée du rendu : elle donne, pour chaque
  joueur, sa place dans l'ordre du tour et un angle autour de la table. Un rendu Three Fiber pourrait la
  réutiliser telle quelle (l'angle devient une position dans la scène 3D).
- Les **grilles des adversaires** sont posées à plat sur la table et tournées vers leur propriétaire (celles d'en
  face apparaissent à l'envers, comme sur une vraie table) ; leurs **étiquettes** (nom, score) sont redressées
  par la rotation inverse de la table et restent lisibles. Écartées à la demande du propriétaire du dépôt : des
  grilles toutes droites face à moi (lisibles mais sans effet de table) et une légère inclinaison vers le
  centre.
- Les cartes restent des `button` HTML : clavier, focus et libellés pour lecteur d'écran ne changent pas.
- `prefers-reduced-motion: reduce` supprime transitions et animations.
- Aucune nouvelle dépendance.

## Alternatives écartées (pour l'instant)

- **React Three Fiber** (`three`, `@react-three/fiber`, `@react-three/drei`) : vraie 3D (volume, lumière,
  caméra), mais plus de 150 Ko de JavaScript, accessibilité à reconstruire (une scène WebGL n'a pas de
  boutons), et plus exigeant pour les téléphones. Reste l'option si le CSS 3D ne convainc pas.
- **Bibliothèque d'animation** (Framer Motion…) : inutile pour un retournement et quelques glissements.

## Conséquences

- La demande de la v1 classait les animations hors périmètre : elles entrent ici à la demande explicite du
  propriétaire du dépôt.
- Le rendu dépend de la perspective CSS, bien prise en charge par les navigateurs récents ; sur petit écran,
  la table est affichée à plat pour rester lisible.

## Essai de React Three Fiber, abandonné

Un prototype WebGL (React Three Fiber, avec un sélecteur pour comparer les deux rendus sur la même partie) a été
essayé sur une branche jetable, puis supprimé à la demande du propriétaire du dépôt, qui préfère le rendu CSS 3D.
Constats retenus : environ 244 Ko compressés de plus (chargés à la demande), étiquettes trop petites sur
téléphone, scène inaccessible au clavier sans repli HTML, pour un gain visuel jugé faible.
