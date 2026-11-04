# Version et dernier commit en bas de l'accueil

## Contexte

Le propriétaire du dépôt veut voir, en bas de l'accueil, la dernière version du jeu (le tag de la release GitHub)
et le dernier commit, chacun avec un lien vers sa page GitHub. Le dépôt `aaimeraud/spacejo` est public.

## Décision

- **Le navigateur interroge directement l'API REST de GitHub**, sans jeton : `GET /repos/aaimeraud/spacejo/commits/main`
  et `GET /repos/aaimeraud/spacejo/releases/latest` (`web/src/github.ts`). L'API autorise ces appels depuis une
  page web (`Access-Control-Allow-Origin: *`, vérifié). On montre ainsi la dernière version **publiée sur GitHub**,
  même si le serveur déployé n'a pas encore été mis à jour.
- **Cache de 10 minutes** dans `localStorage` : sans jeton, l'API n'accepte que 60 requêtes par heure et par
  adresse. Un échec n'est pas gardé (on réessaiera au prochain affichage de l'accueil).
- **Réponses vérifiées** : on ne garde que le tag, le numéro court et le titre du commit, et des liens qui
  commencent par `https://github.com/` (jamais un lien `javascript:` ou vers un autre site).
- Rien ne s'affiche tant que GitHub n'a pas répondu, ni s'il refuse : l'accueil ne dépend pas de GitHub.

## Alternatives écartées

- **Version inscrite au moment du build** (variables de Vite) : montre la version déployée, pas la dernière
  publiée ; et l'image Docker n'a pas l'historique Git.
- **Le serveur Rust interroge GitHub et met en cache** : une seule requête pour tous les visiteurs, mais il faudrait
  un client HTTP (une dépendance de plus, `reqwest` et ses dépendances) pour un affichage secondaire.

## Conséquences

- Le navigateur du visiteur contacte `api.github.com` : GitHub voit son adresse (noté dans « Limites connues »).
- Si le dépôt change de nom ou devient privé, modifier `GITHUB_REPO` dans `web/src/config.ts`.
