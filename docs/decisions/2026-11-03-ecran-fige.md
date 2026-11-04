# Écran figé, mis à l'échelle comme un jeu

## Contexte

Le propriétaire du dépôt ne veut aucun défilement : l'application doit tenir exactement dans l'écran, comme un
vrai jeu, sur ordinateur comme sur téléphone, et jusqu'à 10 joueurs.

## Décision

- Chaque écran est dessiné à une **taille de référence** (paysage ou portrait, choisi selon la forme de la
  fenêtre) puis **mis à l'échelle** avec `transform: scale(...)` pour tenir dans la place disponible
  (`FitStage.tsx`, qui mesure la place avec `ResizeObserver`). `html`, `body` et la racine ont `overflow: hidden`.
- Deux dispositions seulement, choisies par classe (`.is-landscape`, `.is-portrait`) et non par requêtes média,
  puisque c'est la scène, et non la fenêtre, qui a une taille fixe.
- Le nombre de joueurs règle la taille des sièges (`crowd-few`, `crowd-some`, `crowd-many`).
- **Pas de barre de navigation** : il n'y a nulle part où aller. Chaque écran occupe toute la fenêtre, avec la
  même marge des quatre côtés (24 px, 12 px sur téléphone). Le code de la partie et « Quitter » sont sur l'écran
  du salon et dans la barre de la partie ; l'accueil est posé directement sur le fond, sans panneau.
- Ce qui peut être long (tableau des scores, explications des règles) s'ouvre dans une fenêtre dont seul le
  contenu défile.

## Alternatives écartées

- **Mise en page fluide avec requêtes média** : chaque taille d'écran demande ses propres réglages, et rien ne
  garantit que tout tienne sans défilement.
- **Unités `vw`/`vh` partout** : les proportions changent d'un écran à l'autre (cartes déformées, textes trop
  petits ou trop grands).

## Conséquences

- Sur un très petit écran (téléphone couché), tout est plus petit, mais tout reste visible.
- Les textes suivent l'échelle : à forte réduction, ils peuvent devenir petits ; la disposition portrait est
  prévue pour les téléphones tenus droits.
