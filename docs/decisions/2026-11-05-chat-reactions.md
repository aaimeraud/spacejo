# Réactions aux messages du chat

## Contexte

Le propriétaire du dépôt veut pouvoir réagir aux messages du chat par un emoji, comme dans les messageries, avec un
double tap (ou double clic) pour poser un cœur. Il veut aussi une « photo de profil » faite de l'initiale du pseudo.

## Décision

- **Une réaction par joueur et par message**, stockée dans une table à part (`chat_reactions`, clé primaire
  `message_id, player_id`). En choisir une autre la remplace ; `null` la retire. C'est le comportement de WhatsApp
  et d'iMessage, et il évite qu'un message se couvre de réactions identiques.
- **Liste fermée de 6 réactions, envoyées par leur nom** (`heart`, `thumbs_up`…) et non par l'emoji : la validation
  est simple (valeur inconnue = refus), et deux écritures Unicode d'un même emoji (avec ou sans sélecteur de
  variante) ne deviennent pas deux réactions différentes. L'interface traduit le nom en emoji.
- **Rafraîchissement** : un avis `reaction` (numéro du message) sur le flux existant ; le navigateur relit alors les
  50 derniers messages (`GET /api/messages` sans `after`) et remplace ceux qu'il connaît.
- **Avatar** : calculé par l'interface à partir du nom (aucune donnée de plus côté serveur).

## Alternatives écartées

- **Réactions dans `games.state`** : chaque réaction changerait la `version`, donc provoquerait des conflits (409)
  avec les coups joués au même moment, comme l'aurait fait le chat lui-même.
- **Plusieurs réactions par joueur** (comme Slack) : plus de lignes, une interface plus chargée sur téléphone, pour
  un besoin qui n'a pas été exprimé.
- **Emoji libre** : il faudrait valider un emoji quelconque (graphèmes, séquences ZWJ), et un picker complet
  alourdirait l'interface.
- **Relire seulement le message concerné** : il faudrait une route de plus ; relire les 50 derniers messages (des
  textes de 200 caractères au plus) reste léger.

## Conséquences

- Sans flux d'avis, le polling du chat relit les 50 derniers messages à chaque passage (et non plus seulement les
  nouveaux), un peu plus de données transférées dans ce cas dégradé.
- Une réaction ne compte pas comme un message non lu.
