# PLAN — SpaceJo v1

Plan courant de la version 1 jouable. Les hypothèses de règles sont dans `ASSUMPTIONS.md`, les décisions
importantes dans `docs/decisions/`.

## Étapes

1. [x] Hook Git, `scripts/check-commit.sh` et ses tests, squelette de `docs/`.
2. [x] `docs/RULES.md`, ce plan, `docs/CONTRACT.md`.
3. [x] `spacejo-core` : tests d'abord (critères 1 à 8, 13 à 15), puis implémentation.
4. [x] Vue filtrée `view_for` (critère 10) et jetons (critère 11).
5. [x] API, persistance, verrou optimiste, migrations, version du format d'état (critères 9, 12, 16, 17, 18).
6. [x] Interface React (accueil, salon, partie, fin de partie).
7. [x] CI, release (git-cliff), `vercel.json`, `.env.example`, `docker-compose.yml`, README.
8. [x] Relecture critère par critère ; `docs/` décrit le code réel.

## Évolution en cours : vue « table de jeu » en CSS 3D (prototype)

Demandée après la v1 par le propriétaire du dépôt, qui pourra préférer ensuite React Three Fiber
(voir `docs/decisions/2026-11-03-vue-table-css-3d.md`). Touche uniquement `web/` (l'API ne change pas).

1. [x] `web/src/game/seats.ts` : place des joueurs autour de la table (moi en bas, les autres dans l'ordre
   du tour, sens des aiguilles d'une montre), sans nombre de joueurs en dur ; tests.
2. [x] Table inclinée en perspective, sièges avec badge (nom, total, cartes cachées, hôte, tour, a terminé),
   pioche et défausse au centre, carte en main au-dessus.
3. [x] Cartes à deux faces qui se retournent (`rotateY`), apparition d'une carte posée, défausse animée ;
   aucune animation si `prefers-reduced-motion`.
4. [x] Petit écran : table à plat, adversaires côte à côte, cartes et badges compacts.
5. [x] Vérification dans le navigateur à 2 et 4 joueurs ; documentation à jour.
6. [x] Retours du propriétaire : table plus inclinée, grille de l'adversaire actif agrandie, animation propre
   aux colonnes retirées, cartes plus grandes et plus espacées.
7. [x] Deuxième retour : inclinaison ramenée à 30°, grilles des adversaires tournées vers eux ; prototype
   React Three Fiber essayé puis abandonné (voir la décision).

Aucune nouvelle dépendance.

## Évolution en cours : écran des réglages (branche `claude/rules-ux`)

Piste choisie par le propriétaire du dépôt parmi deux propositions : préréglages + cartes de règles.

1. [x] `rulesInfo.ts` : titres courts, pictogrammes, exemples chiffrés, préréglages, résumé ; tests.
2. [x] `RulesPanel.tsx` : tuiles de préréglage (dont le bouton « Règles officielles »), cartes dépliables avec
   choix en pastilles (boutons radio accessibles), curseur et raccourcis pour le score de fin, résumé en lecture
   seule pour les autres joueurs.
3. [x] Accueil : préréglages visibles directement ; vérification dans le navigateur (accueil, salon hôte et invité).

## Structure des dossiers

Un seul serveur Rust sert l'API et l'interface (voir `docs/decisions/2026-11-04-serveur-unique-render.md` ;
la première structure, une fonction Vercel par route, est décrite dans `docs/decisions/2026-11-03-structure-vercel.md`).

```
Cargo.toml            espace de travail Cargo + paquet `spacejo-api`
src/                  bibliothèque `spacejo-api` : routes, jetons, stockage, serveur HTTP (axum)
src/bin/server.rs     le serveur : l'API et l'interface construite, en local comme en production
src/bin/migrate.rs    applique les migrations SQL sur la base désignée par DATABASE_URL
crates/spacejo-core/  logique du jeu, sans réseau, sans base, sans horloge
migrations/           migrations SQL versionnées (seul moyen de faire évoluer le schéma)
tests/                tests d'intégration de l'API et du serveur (+ tests/scripts/ pour les scripts shell)
web/                  interface React + TypeScript + Vite
docs/                 documentation vivante
scripts/              check-commit.sh, release.sh, changelog.sh
Dockerfile            image de production (interface puis serveur) ; render.yaml : le service Render
.github/workflows/    ci.yml, release.yml
```

Routes : `/api/health`, `/api/create`, `/api/join`, `/api/rules`, `/api/start`, `/api/action`, `/api/skip`,
`/api/state`, `/api/events`, `/api/public-games`, `/api/visibility`, `/api/leave`, `/api/kick`, `/api/chat`, `/api/messages` ; toute autre adresse reçoit l'interface.

## Base de données et driver

- **PostgreSQL** hébergé (Supabase), dans une région proche du serveur (Irlande pour la base, Francfort pour
  le serveur Render). En local : PostgreSQL via `docker-compose.yml`.
- **Driver : `sqlx` 0.9** (fonctions `sqlx::query`, sans les macros vérifiées à la compilation, qui
  exigeraient une base pendant la compilation). Raisons : asynchrone sur `tokio` (le moteur du serveur
  axum), TLS intégré (`rustls`, sans OpenSSL à installer, obligatoire pour une base hébergée),
  et un système de migrations versionnées (`sqlx::migrate!`) qui embarque le dossier `migrations/` dans le
  binaire et note les migrations appliquées dans la table `_sqlx_migrations`.
- Alternative écartée : `tokio-postgres` + `refinery` + une caisse TLS : trois dépendances au lieu
  d'une, et TLS à câbler à la main.
- Une connexion par requête (pas de pool en mémoire : fonctions sans état). En production, utiliser
  l'URL « poolée » du fournisseur (PgBouncer) pour ne pas épuiser les connexions.

## Dépendances (justification)

| Caisse | Où | Pourquoi |
|---|---|---|
| `serde` (+ `derive`) | core, api | sérialiser l'état en JSON (JSONB) et les vues |
| `serde_json` | core (tests), api | lire et écrire le JSON |
| `axum` 0.8 | api | serveur HTTP (routes, extraction des requêtes), de l'écosystème tokio |
| `tower-http` 0.7 | api | fichiers de l'interface (`ServeDir`, versions gzip) et compression des réponses de l'API |
| `tower` 0.5 | api | appeler ces services depuis une route axum, et le serveur dans les tests sans ouvrir de port |
| `tokio` | api | moteur asynchrone d'axum et de `sqlx` (réseau, fichiers, signaux d'arrêt) |
| `sqlx` 0.9 | api | driver PostgreSQL + migrations (voir ci-dessus) |
| `getrandom` | api | entropie du système : jetons, codes de partie, graines |
| `sha2` | api | hachage SHA-256 des jetons (seul le hachage est stocké) |
| `react`, `react-dom` | web | imposé |
| `typescript`, `vite`, `@vitejs/plugin-react` | web | imposé (Vite + TypeScript strict) |
| `vitest` | web | tests du front, même configuration que Vite |
| `qrcode-generator` 2.0.4 | web | matrice du code QR du lien d'invitation (MIT, aucune dépendance) ; voir l'évolution « visibilité modifiable au salon » ci-dessous |
| police Inter (Google Fonts) | web | une police lisible et moderne avec chiffres de même largeur ; ressource chargée par la page, pas un paquet npm (voir `docs/decisions/2026-11-04-theme-nuit.md`) |

Pas de caisse `rand` dans le core : le mélange utilise un petit générateur SplitMix64 écrit dans le
core (voir `docs/decisions/2026-11-03-generateur-aleatoire.md`), pour un résultat identique quelle que
soit la version des dépendances.

Outils (hors dépendances du code) : `git-cliff` pour le changelog et les versions (justifié dans
`docs/decisions/2026-11-03-releases-git-cliff.md`).

## Risques

- **Hébergement gratuit (Render)** : mise en veille après 15 minutes sans visite (réveil d'environ une
  minute), 512 Mo de mémoire ; l'image a été testée en local, pas encore sur la plateforme.
- **Connexions PostgreSQL** : une connexion par requête coûte une poignée de main TLS ; acceptable
  avec un polling toutes les 2 s et 4 joueurs, à revoir au-delà.
- **Polling** : charge proportionnelle au nombre de joueurs connectés ; la réponse est minimale quand
  rien n'a changé.
- **GitButler** ne lance pas le hook `commit-msg` : le contrôle est lancé à la main avant chaque commit
  et la CI le refait (voir `docs/decisions/2026-11-03-controle-des-commits.md`).

## Licence (au choix du propriétaire du dépôt)

Aucun fichier `LICENSE` n'est ajouté. Options possibles : MIT (permissive, courte), Apache-2.0
(permissive, avec clause de brevets), AGPL-3.0 (oblige à publier les modifications d'un service en ligne),
ou rien (tous droits réservés, ce qui est le cas actuel).

## Évolution en cours : jusqu'à 10 joueurs, absence et reprise (branche `claude/players-presence`)

Demandée par le propriétaire du dépôt. Les durées sont des constantes nommées (voir `ASSUMPTIONS.md`, n° 18 à 22).

1. [x] Core (tests d'abord) : `MAX_PLAYERS` = 10 ; tour d'un joueur absent passé par le serveur (`skip_absent`) ;
   après `MAX_MISSED_TURNS` (2) tours manqués d'affilée, élimination ; un joueur éliminé ne joue plus, ne reçoit
   plus de cartes, ne compte plus pour la pénalité, la fin de partie ni le classement ; s'il ne reste qu'un joueur,
   il gagne.
2. [x] Format enregistré v2 (`eliminated`, `missed_turns`) avec migration depuis la v1 et exemple figé v2.
3. [x] API : table `presence` (migration 0002) mise à jour à chaque requête d'un joueur, sans changer la version de
   la partie ; un joueur est absent s'il n'a rien demandé depuis `PRESENCE_TIMEOUT_SECS` ; si c'est son tour et que
   la partie n'a pas bougé depuis `ABSENT_TURN_WAIT_SECS`, la prochaine requête d'un autre joueur passe son tour ;
   les réponses donnent la présence de chacun et le compte à rebours.
4. [x] Interface : reprise d'une partie après fermeture de l'onglet (sessions gardées dans `localStorage`),
   indicateurs « absent », « on l'attend encore N s », « éliminé ».

## Évolution en cours : écran de jeu figé, création pas à pas, icônes (branche `claude/app-layout`)

1. [x] Aucun défilement : l'application occupe exactement l'écran ; la table est dessinée à une taille de référence
   puis mise à l'échelle pour tenir dans la fenêtre (paysage ou portrait), jusqu'à 10 joueurs.
2. [x] Création de partie pas à pas (nom, type de partie, puis un réglage par écran si « Personnalisée »,
   récapitulatif), avec retour en arrière ; même parcours pour modifier les règles au salon.
3. [x] Icônes `lucide-react` 1.59.0 (licence ISC, aucune dépendance) : seule dépendance ajoutée, pour des icônes
   cohérentes et accessibles sans dessiner chaque pictogramme.


## Évolution en cours : un seul serveur Rust, hébergé sur Render (branche `claude/render`)

Demandée par le propriétaire du dépôt : le runtime Rust de Vercel (bêta) est trop lent à construire et son quota
d'appels convient mal au polling. Un serveur unique, toujours démarré, sert l'API et l'interface. Décision :
`docs/decisions/2026-11-04-serveur-unique-render.md`.

1. [x] Serveur `axum` (`src/server.rs`, binaire `server`, remplace `dev_server`) : `/api/*` vers `handlers`,
   `/api/health` pour la plateforme, fichiers de `web/dist` avec retour à `index.html`, compression gzip,
   en-têtes de cache, arrêt propre sur SIGTERM ; migrations appliquées au démarrage. Port : `PORT`.
2. [x] Suppression de ce qui ne servait qu'à Vercel : `api/*.rs`, `src/vercel.rs`, `vercel_runtime`,
   `vercel.json`, `.vercelignore`.
3. [x] `Dockerfile` (interface puis serveur, image finale minimale), `.dockerignore`, `render.yaml` (offre
   gratuite, Francfort, contrôle de santé, variable `DATABASE_URL`).
4. [x] Interface : réveil du serveur dès l'ouverture de l'accueil (l'offre gratuite de Render l'endort après
   15 minutes sans requête).
5. [x] Vérifications : tests, image Docker construite et lancée en local avec PostgreSQL, partie jouée dans le
   navigateur ; documentation et guide de déploiement pas à pas.

Dépendances ajoutées : `axum` (serveur HTTP de l'écosystème tokio, le plus courant en Rust), `tower-http`
(fichiers de l'interface, compression) et `tower` (pour appeler ces services). `vercel_runtime` et
`http-body-util` sont retirées.

## Évolution en cours : lien d'invitation, parties publiques et privées (branche `claude/invite-public-games`)

Demandée par le propriétaire du dépôt après la release v1.0.0. Décision :
`docs/decisions/2026-11-04-parties-publiques.md`.

1. [x] Base : migration 0003, colonne `games.public` (faux par défaut : les parties existantes restent privées)
   et index partiel des parties publiques. La visibilité n'est pas une règle du jeu : elle reste hors de `Game`
   (le core ne change pas, pas de nouvelle version du format enregistré).
2. [x] API (tests d'abord) : `create` accepte `public` (facultatif, faux par défaut) ; toutes les réponses qui
   donnent l'état contiennent `public` ; nouvelle route `GET /api/public-games` (sans jeton) : parties publiques
   encore au salon, pas pleines, dont l'hôte est présent, les plus récentes d'abord (au plus
   `PUBLIC_GAMES_LIMIT`), avec code, nom de l'hôte et nombre de joueurs.
3. [x] Interface : à la création, étape « Qui peut rejoindre ? » (privée ou publique) ; au salon, bouton
   « Copier le lien d'invitation » (partage natif sur téléphone) et mention publique/privée ; à l'accueil, tuile
   « Parties publiques » (liste rafraîchie, clic → nom → rejoindre) ; un lien `/?code=XXXXXX` ouvre directement
   l'étape du nom avec le code rempli.
4. [x] Documentation (`CONTRACT.md`, `ARCHITECTURE.md`, décision, `ASSUMPTIONS.md`) ; vérification dans le navigateur.

Aucune nouvelle dépendance.

## Évolution en cours : visibilité modifiable au salon, code QR du lien (branche `claude/lobby-visibility-qr`)

Demandée par le propriétaire du dépôt après la v1.1.0 (idées notées dans `IDEAS.md`).

1. [x] API (tests d'abord) : `POST /api/visibility` (hôte, au salon seulement, avec `expected_version`) rend la
   partie publique ou privée ; écrit la colonne `games.public` et incrémente `version` (même verrou optimiste), ce
   qui prévient les autres joueurs par le flux d'avis.
2. [x] Interface : au salon, l'hôte bascule « publique / privée » ; bouton « Code QR » qui affiche le lien
   d'invitation en code QR (dessiné en SVG par React, toujours sombre sur fond clair pour rester lisible).
3. [x] Documentation, vérification dans le navigateur.

Dépendance ajoutée : `qrcode-generator` 2.0.4 (licence MIT, aucune dépendance, ~20 Ko), pour calculer la matrice du
code QR. Écrire un encodeur QR soi-même (correction d'erreurs Reed-Solomon, masques) serait long et risqué pour un
gain nul. Écartée : `qrcode` (trois dépendances, dont `yargs`, pour la ligne de commande).

## Évolution en cours : quitter, exclure, limitation de débit, retournements rejoués (branche `claude/lobby-visibility-qr`)

Demandée par le propriétaire du dépôt pour la v1.2.0, avec la bascule de visibilité et le code QR.

1. [x] Core (tests d'abord) :
   - `leave_lobby` : un joueur quitte le salon ; il est retiré et les suivants avancent d'une place (si c'est l'hôte,
     le joueur arrivé ensuite devient l'hôte). Le dernier joueur ne peut pas être retiré (`last_player`).
   - `kick` : l'hôte retire un joueur du salon (jamais lui-même : `cannot_kick_host`).
   - `abandon` : un joueur quitte une partie lancée ; il est marqué `abandoned` et éliminé tout de suite (mêmes
     conséquences qu'une élimination pour absence : son tour passe, il ne compte plus ; seul restant = gagnant).
   - Format enregistré v3 (`abandoned` sur chaque joueur), avec migration depuis la v2 et exemple figé.
2. [x] API : `POST /api/leave` (jeton ; sans `expected_version`, réessayé par le serveur comme la jonction, pour
   qu'un départ ne soit jamais refusé pour conflit) et `POST /api/kick` (hôte, `expected_version`). Au salon, le
   jeton et la présence du joueur retiré sont supprimés, ceux des suivants décalés. La vue donne `abandoned`.
3. [x] Interface : « Quitter » prévient le serveur (avec confirmation pendant la partie) ; bouton « Retirer » de
   l'hôte au salon ; tout le monde voit « X a abandonné la partie » (message et pastille), ou « X a quitté le
   salon » ; un joueur retiré revient à l'accueil avec un message clair.
4. [x] API : limitation de débit par adresse IP sur `create`, `join` et `public-games` (fenêtre fixe d'une minute,
   compteurs en mémoire, `429 rate_limited` avec `Retry-After`).
5. [x] Interface : un retournement de mise en place refusé pour conflit de version (409) est relu puis rejoué
   automatiquement (quelques essais), au lieu d'afficher une erreur.
6. [x] Documentation, vérifications, essai dans le navigateur.

Aucune nouvelle dépendance.

## Évolution en cours : chat pendant les parties (branche `claude/game-chat`)

Demandée par le propriétaire du dépôt, pour une release mineure (v1.3.0).

1. [x] Base : migration 0004, table `chat_messages` (partie, joueur, nom au moment de l'envoi, texte), effacée avec
   la partie. Les messages ne font pas partie de l'état du jeu : envoyer un message ne change pas `version` (aucun
   conflit avec les coups).
2. [x] API (tests d'abord) : `POST /api/chat` (jeton ; texte de 1 à `MAX_CHAT_CHARS` caractères, sans caractère de
   contrôle) et `GET /api/messages?code=…&after=…` (jeton ; messages plus récents que `after`, au plus
   `MAX_CHAT_HISTORY`). Le flux d'avis (`GET /api/events`) envoie un évènement `chat` avec le numéro du nouveau
   message ; sans flux, le navigateur relit les messages au rythme du polling. Limitation de débit sur l'envoi.
   Le texte des messages n'est jamais journalisé.
3. [x] Interface : bouton « Chat » sur l'écran de partie, avec le nombre de messages non lus (en chiffres, pas
   seulement une couleur) ; panneau avec la liste (annoncée aux lecteurs d'écran) et un champ d'envoi (Entrée).
4. [x] Documentation, vérifications, essai dans le navigateur à plusieurs joueurs.

Aucune nouvelle dépendance.

## Évolution en cours : version et dernier commit sur l'accueil (branche `claude/repo-trackers`)

Demandée par le propriétaire du dépôt.

1. [x] `web/src/github.ts` (tests d'abord) : lit le dernier commit de `main` et la dernière release du dépôt public
   sur l'API REST de GitHub (sans jeton), vérifie la forme des réponses, et garde le résultat 10 minutes dans
   `localStorage` (l'API publique n'autorise que 60 requêtes par heure et par adresse).
2. [x] Accueil : en bas, « Version v1.3.0 » (lien vers la release) et « Dernier commit 0ceeadf » (lien vers le
   commit, avec son titre) ; rien ne s'affiche si GitHub ne répond pas.
3. [x] Documentation (décision : appel direct du navigateur à GitHub), vérifications, essai dans le navigateur.

Aucune nouvelle dépendance. Service externe : `api.github.com`, appelé par le navigateur du visiteur.

## Évolution en cours : pioche réduite et animation du remélange (branche `claude/reduced-deck`)

Demandée par le propriétaire du dépôt : moins de cartes dans la pioche, proportionnellement au nombre de joueurs,
pour que la défausse soit remélangée en pioche plus souvent ; et une animation à ce moment-là.

1. [x] Core (tests d'abord) : nouveau réglage `deck` de `Rules` (CLAUDE.md, règle 6 : ce que la notice ne prévoit
   pas est un réglage, défaut = règles officielles). `Full` (défaut, officiel) : les 150 cartes. `Reduced` (variante) :
   après le mélange, on ne garde que `REDUCED_DECK_CARDS_PER_PLAYER` (40) cartes par joueur en jeu, au plus 150 —
   12 dans sa grille et 28 pour la pioche. Les cartes gardées sont tirées au hasard (graine de la partie) : la
   proportion des valeurs reste celle du paquet complet en moyenne.
2. [x] Format enregistré v4 (`rules.deck`), avec migration (`Full` pour les parties existantes) et exemple figé.
3. [x] Interface : réglage « Taille de la pioche » dans le parcours « Personnalisée » et le récapitulatif ; animation
   des cartes qui passent de la défausse à la pioche, avec un message (« La défausse est mélangée… »), détectée
   en comparant deux vues ; aucune animation si `prefers-reduced-motion`.
4. [x] Documentation (`RULES.md`, `CONTRACT.md`, `ASSUMPTIONS.md`), vérifications, essai dans le navigateur.

Aucune nouvelle dépendance.

## Évolution en cours : tutoriel et mentions légales (branche `claude/tutorial-legal`)

Demandés par le propriétaire du dépôt : un court tutoriel pour les nouveaux joueurs, sur une partie aux règles
officielles (réglages par défaut), proposé plus discrètement que les autres choix de l'accueil ; et une page de
mentions légales pour un projet personnel, scolaire, open source et non commercial, sans données personnelles
de l'éditeur.

1. [x] Tutoriel (`web/src/tutorial.ts` pour le contenu, `components/Tutorial.tsx` pour l'affichage) : quelques
   étapes avec des cartes d'exemple (but, mise en place, tour, colonne identique, fin de manche et pénalité, fin
   de partie). Contenu tiré de `docs/RULES.md`, règles officielles uniquement. Lien discret sous les trois tuiles
   de l'accueil.
2. [x] Mentions légales (`components/LegalNotice.tsx`) : éditeur non professionnel anonyme (LCEN, art. 1-1 II :
   seul l'hébergeur est nommé), hébergeurs (Render, Supabase), données traitées, stockage local sans cookie,
   services tiers (Google Fonts, API GitHub), propriété intellectuelle et marque Skyjo. Lien en pied de l'accueil.
3. [x] Documentation (`ARCHITECTURE.md`, décision), vérifications, essai dans le navigateur (téléphone et ordinateur).

Aucune nouvelle dépendance.

## Évolution en cours : issues #10, #11 et #12 (branche `daniel/issues-10-11-12`)

1. [x] #10 — Téléphone : la pioche et la défausse étaient cachées sous ma grille (le centre de la table héritait du
   `top: 50%` du paysage). La scène portrait devient une colonne (la table prend la hauteur restante) et les cartes
   de l'adversaire regardé suivent la place disponible.
2. [x] #11 — Fin de manche : chaque joueur vote pour la manche suivante, tous voient le nombre de votes, l'hôte
   lance avec les votants ; les autres quittent la partie et reviennent à l'accueil. Core (`next_round_votes`,
   `next_round_decider`, erreur `not_enough_votes`), format enregistré v5 avec migration, interface, documentation.
3. [x] #12 — README : section « Ce qui ne marche pas encore » remise à jour (déploiement sur Render).
4. [ ] Vérifications, essai dans le navigateur, release.

Aucune nouvelle dépendance.
