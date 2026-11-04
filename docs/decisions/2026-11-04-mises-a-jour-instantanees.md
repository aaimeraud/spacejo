# Mises à jour instantanées : avis poussés par le serveur, moins d'allers-retours vers la base

## Contexte

Une fois le jeu déployé (Render à Francfort, base Supabase en Irlande), le propriétaire du dépôt trouvait la
latence entre les actions beaucoup trop grande. Trois causes, de la plus lourde à la plus légère :

1. **Le polling** : un coup n'apparaissait chez les autres qu'à leur prochaine lecture de l'état, jusqu'à
   2 secondes plus tard.
2. **Les allers-retours vers la base** : 3 requêtes SQL par lecture d'état et 4 par action, entre Francfort et
   l'Irlande (quelques dizaines de millisecondes chacune).
3. **Le cache de requêtes préparées de sqlx, désactivé** pour le pooler en mode transaction : chaque requête
   demandait un aller-retour de plus.

## Décision

- **Avis poussés par le serveur** (server-sent events, `GET /api/events?code=…`, `src/events.rs`) : quand une
  partie change de version (un coup, une jonction, un tour d'absent passé), le serveur l'annonce aussitôt aux
  navigateurs abonnés, qui relisent l'état. Le flux ne transporte que le numéro de version : aucune carte, aucun
  nom. Il n'exige donc pas de jeton (que `EventSource` ne sait pas envoyer en en-tête) et ne change rien à la
  règle de la vue filtrée : chaque joueur relit sa propre vue avec son jeton.
- **Le polling reste**, plus lent quand le flux est ouvert (toutes les 5 s, sous le délai d'absence de 15 s) :
  il signale la présence du joueur et rattrape un avis perdu. Sans flux (coupure, navigateur ancien), le rythme
  d'avant revient (2 s en partie).
- **Présence notée et lue en une requête** (`Store::touch_presence`, une CTE qui écrit puis lit) : 2 requêtes
  SQL par lecture d'état, 3 par action.
- **Cache de requêtes préparées de sqlx réactivé** : avec le pooler de Supabase en mode session (port 5432),
  chaque connexion est à nous ; une requête déjà vue part en un seul aller-retour. Le mode transaction (port
  6543) n'est donc plus pris en charge.

Mesure en local : le coup d'un adversaire s'affiche 35 à 70 ms après son envoi, contre jusqu'à 2 s avant.

## Alternatives écartées

- **WebSocket** : bidirectionnel, inutile ici (les actions passent déjà par des requêtes HTTP), et demande plus de
  code des deux côtés. SSE passe par une simple réponse HTTP qui ne se termine pas.
- **Envoyer la vue dans l'avis** : il faudrait un flux par joueur, authentifié, et filtrer la vue pour chacun ;
  relire l'état ne coûte qu'une requête.
- **Supabase Realtime** : un service de plus à brancher, alors que le serveur, désormais toujours démarré, sait
  pousser les avis lui-même.
- **Rapprocher la base du serveur** (base Render à Francfort) : la base gratuite de Render expire au bout de
  30 jours.

## Conséquences

- Les avis vivent dans la mémoire du serveur : avec plusieurs serveurs, un navigateur ne recevrait que ceux de
  son serveur (les autres coups arriveraient au polling, en 5 s au plus). Un seul serveur sur l'offre gratuite.
- Chaque onglet ouvert garde une connexion : sur Render, le service ne s'endort pas tant qu'un joueur a le jeu
  ouvert.
- À l'arrêt du serveur, les flux sont fermés (`Events::close_all`), sinon l'arrêt propre les attendrait.
