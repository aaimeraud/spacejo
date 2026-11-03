# Idées hors périmètre

Idées notées pendant le travail, à étudier plus tard.

- Comptes utilisateurs, classement, historique des parties.
- SSE ou WebSocket à la place du polling.
- Reconnexion avancée (changer d'appareil sans perdre son jeton).
- Joueurs automatiques, plus de 4 joueurs.
- Animations plus riches (carte qui vole de la pioche vers la grille), ou passage de la table à React Three
  Fiber (vraie 3D) si le CSS 3D ne convient pas (voir docs/decisions/2026-11-03-vue-table-css-3d.md).
- Réglage pour décider si les colonnes identiques sont retirées au retournement final (ASSUMPTIONS.md, n° 5).
- Rejouer automatiquement un retournement de mise en place refusé pour conflit de version (409), fréquent
  quand tous les joueurs retournent leurs cartes en même temps.
