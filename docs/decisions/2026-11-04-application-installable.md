# Application installable (PWA), icônes et aperçus de liens

## Contexte

Le propriétaire du dépôt veut pouvoir installer SpaceJo sur un téléphone, comme une application, et a fourni
ses visuels : une icône iOS (PNG 1024 px), l'icône et l'image d'aperçu en SVG.

## Décision

- **Fichiers servis tels quels** depuis `web/public/` (Vite les copie dans `web/dist/`) :
  - `favicon.ico` (16 et 32 px) et `icons/favicon-16|32|96.png`, tirés de l'icône iOS ; `favicon.svg`, l'icône
    vectorielle, pour les navigateurs récents ;
  - `apple-touch-icon.png` (180 px), l'icône iOS sur fond noir (iOS arrondit les coins lui-même) ;
  - `icons/icon-192|512.png` (icône iOS, coins transparents) et `icons/icon-maskable-192|512.png` (l'icône
    SVG sans ses coins arrondis, carrée et pleine : Android la découpe à sa forme) ;
  - `og-image.png` (1200 × 630), l'aperçu des liens partagés, rendu depuis le SVG en 2400 px puis réduit.
  Les PNG ont été produits une fois, en dehors du projet (rendu des SVG par un navigateur sans interface,
  regroupement des PNG en `.ico` par un petit script) : le projet ne gagne aucune dépendance.
- **Manifeste** `manifest.webmanifest` : nom, langue, `display: standalone`, couleurs, icônes.
- **Service worker** `public/sw.js`, écrit à la main (quelques lignes) plutôt qu'avec un greffon comme
  `vite-plugin-pwa` : « réseau d'abord » pour les fichiers du site (copie en cache seulement si le réseau ne
  répond pas), et **jamais de cache pour `/api/`** : l'état d'une partie vient toujours du serveur. Il n'est
  enregistré que sur le site construit (`main.tsx`, `import.meta.env.PROD`), pas avec le serveur de
  développement. Le serveur le sert sans cache (`src/server.rs`).
- **iOS** : `apple-mobile-web-app-capable`, titre, barre d'état translucide ; l'écran va donc sous l'encoche,
  et `.app` garde des marges `env(safe-area-inset-*)`. En portrait, l'application installée sur iPhone ajoute 16 px
  sous la zone sûre (au moins 50 px au total sous le haut de l'écran) : iOS floute une bande sous la barre d'état,
  plus haute que la zone sûre, et certaines versions d'iOS 26 annoncent une zone sûre nulle.
  Cette marge du haut vit dans la variable CSS `--safe-top`, reprise par les fenêtres (`.modal-backdrop`), qui
  couvrent tout l'écran ; le chat, posé en bas, respecte la zone sûre du bas.
- **Couleur de la barre** (`theme-color`) : celle du fond du thème affiché, mise à jour par `theme.ts`.
- **Aperçus Open Graph et X/Twitter** dans `index.html`. Les robots exigent une adresse absolue : elle est
  préfixée au moment de la construction par la variable `VITE_SITE_URL` (vide par défaut, adresses relatives).

## Alternatives écartées

- `vite-plugin-pwa` (Workbox) : génère le manifeste et un service worker précachant tout, mais ajoute une
  dépendance lourde pour un jeu qui ne marche de toute façon pas hors ligne (il lui faut le serveur).
- Écrans de démarrage iOS (`apple-touch-startup-image`) : une image par taille d'écran d'iPhone et d'iPad ;
  sans eux, iOS affiche un écran de la couleur du fond, ce qui suffit.

## Conséquences

- Installable depuis Chrome et Edge (Android, ordinateur : « Installer l'application ») et depuis Safari sur
  iOS (Partager → « Sur l'écran d'accueil »). L'installation n'a pas encore été essayée sur un vrai téléphone.
- Hors ligne, l'application s'ouvre (fichiers en cache) mais ne peut pas jouer : chaque action passe par l'API.
- Pour changer une icône, remplacer les fichiers de `web/public/` aux mêmes noms et tailles ; pour forcer les
  appareils à oublier l'ancien cache, changer `CACHE` dans `sw.js`.
