# Un seul serveur Rust, hébergé sur Render (à la place des fonctions Vercel)

## Contexte

Le projet visait des fonctions Rust sur Vercel (runtime en bêta, une fonction par route). Le premier déploiement
a montré deux problèmes : un build très long (Rust compilé sur 2 cœurs, sans cache, une édition des liens par
fonction) et un quota mal adapté au polling (1 000 000 d'appels de fonctions par mois sur l'offre gratuite, qu'un
seul joueur assidu peut presque épuiser). Le propriétaire du dépôt choisit Render, sur une instance gratuite.

Documentation lue le 2026-11-04 : https://render.com/docs/free, https://render.com/docs/blueprint-spec,
https://render.com/docs/docker, https://render.com/docs/web-services.

## Décision

- **Un serveur HTTP unique** (`src/server.rs`, binaire `server`), écrit avec **axum** : `/api/health`, les sept
  routes `/api/*` (toujours traitées par `handlers::handle`, inchangé), et l'interface construite (`web/dist`)
  pour toutes les autres adresses. Une seule adresse pour le jeu et l'API : pas de CORS, et l'application
  installable (PWA) reste sur la même origine que son API.
- **Toujours sans état de partie en mémoire** : chaque requête recharge la partie en base et l'écrit avec le
  verrou optimiste. Plusieurs serveurs pourraient tourner côte à côte.
- **Fichiers de l'interface** servis par `tower-http` (`ServeDir`, la référence de l'écosystème axum) : types de
  contenu, `Last-Modified`, requêtes partielles, version `.gz` préparée au build quand le navigateur l'accepte.
  Le serveur y ajoute ses règles de cache (`assets/` gardés un an, car leurs noms sont hachés ; `index.html`,
  `sw.js` et le manifeste toujours revalidés), renvoie `index.html` pour une adresse sans extension et refuse
  tout chemin contenant `..`.
- **Réponses de l'API compressées** (gzip, `CompressionLayer`) : l'état d'une partie est relu toutes les 2 s.
- **Migrations appliquées au démarrage** du serveur (`sqlx::migrate!`) : plus d'étape manuelle au déploiement.
- **Image Docker** en trois étapes (interface avec Node 24, serveur avec Rust 1.98, image finale Debian minimale
  sous un utilisateur sans droits) ; la couche des dépendances Rust reste en cache tant que `Cargo.lock` ne change
  pas. **`render.yaml`** décrit le service : offre gratuite, Francfort (la région Render la plus proche de la base
  Supabase en Irlande), contrôle de santé `/api/health`, déploiement à chaque commit sur `main`.
- **Base** : pooler Supabase en **mode session** (port 5432). Un serveur toujours démarré garde un petit pool de
  connexions (5), ce que le mode session gère bien ; les migrations au démarrage en ont besoin (elles prennent un
  verrou de session). Le réglage de sqlx pour le mode transaction est gardé : il ne gêne pas.
- **Réveil** : l'offre gratuite endort le service après 15 minutes sans requête (réveil d'environ une minute).
  L'interface appelle `/api/health` dès l'ouverture, pour que le serveur se réveille pendant qu'on lit l'accueil.
- Supprimés : `api/*.rs`, `src/vercel.rs`, `vercel_runtime`, `vercel.json`, `.vercelignore`.

## Alternatives écartées

- **Garder Vercel** : build lent, quota d'appels, runtime en bêta.
- **Interface sur Vercel, API sur Render** : interface plus rapide à ouvrir, mais deux déploiements, du CORS à
  gérer, et chaque polling relayé par Vercel compterait encore dans son quota.
- **Servir les fichiers à la main** : faisable en quelques dizaines de lignes, mais sans revalidation
  (`Last-Modified`), ni requêtes partielles, ni décodage des adresses ; `tower-http` le fait déjà, et bien testé.
- **Autres hébergeurs gratuits** : Koyeb (instance gratuite, mise en veille aussi), Google Cloud Run (quota
  gratuit généreux, mais facturé à la requête et carte bancaire exigée), Fly.io (plus d'offre gratuite pour les
  nouveaux comptes), Oracle Cloud (machine virtuelle gratuite, mais tout à administrer soi-même).

## Conséquences

- Sur l'offre gratuite : 750 heures d'instance par mois (un service allumé en permanence en consomme 744), 512 Mo
  de mémoire, 0,1 processeur ; premier chargement d'environ une minute après une période sans joueur.
- Le premier build Docker sur Render compile tout le Rust (plusieurs minutes) ; les suivants réutilisent la couche
  des dépendances quand Render garde le cache.
- `cargo run` lance le même serveur en local (port 3000, en mémoire sans `DATABASE_URL`) ; s'il trouve
  `web/dist`, il sert aussi l'interface construite.
