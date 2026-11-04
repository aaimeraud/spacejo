# Limitation de débit par adresse IP

## Contexte

Avec les parties publiques, la création, la jonction et la liste des parties sont ouvertes à tous, sans jeton. Un
script pourrait créer des milliers de parties (la base grossit) ou remplir des salons publics de faux joueurs.

## Décision

- **Fenêtre fixe d'une minute** par couple (adresse IP, route), dans `src/rate_limit.rs` : un compteur remis à zéro
  au début de chaque fenêtre. Limites : 10 créations, 30 jonctions, 60 lectures de la liste (l'accueil la relit
  toutes les 5 s). Au-delà : `429 rate_limited` avec `Retry-After`.
- **Compteurs en mémoire** (`Arc<Mutex<HashMap>>`), bornés : au-delà de 10 000 compteurs, ceux des fenêtres finies
  sont effacés. La règle « rien en mémoire entre deux requêtes » vise l'état des parties, pas ces compteurs : les
  perdre au redémarrage n'a pas de conséquence.
- **Adresse** : en-tête `CF-Connecting-IP`, que Cloudflare (devant Render) écrit à chaque requête en remplaçant ce
  qu'aurait envoyé le navigateur ; sinon l'adresse de la connexion (en local). Pas la première adresse de
  `X-Forwarded-For` : le navigateur peut l'inventer, et le proxy de Render ne fait qu'ajouter la sienne. L'adresse
  n'est jamais journalisée.
- Les routes avec jeton (actions, état) ne sont pas limitées : elles demandent déjà d'être joueur de la partie.

## Alternatives écartées

- **Seau à jetons** (token bucket) : plus souple, mais plus de code pour un gain faible ici.
- **Compteurs dans PostgreSQL** : partagés entre serveurs, mais une écriture de plus à chaque requête ; inutile avec
  un seul serveur.
- **Dépendance (`tower_governor`)** : une caisse et ses dépendances pour quelques dizaines de lignes.

## Conséquences

- Avec plusieurs serveurs, chaque serveur aurait ses propres compteurs (limite multipliée par le nombre de serveurs).
- Si l'hébergeur change, l'en-tête de l'adresse est à revoir (`CLIENT_IP_HEADER`, dans `src/server.rs`).
