# Architecture de SpaceJo

> Squelette : cette page sera complétée au fur et à mesure que le code existe.

## Vue d'ensemble

À compléter.

## Limites connues et évolutions prévues

- **Polling** : l'interface interroge le serveur toutes les 2 secondes. Évolution prévue : SSE ou service
  temps réel.
- **Expiration des parties** : les anciennes parties ne sont jamais supprimées.
- **Limitation de débit** : aucune limite sur la création et la jonction de parties.
- **Comptes utilisateurs** : aucun ; l'identité est un jeton secret par partie.
- **Nombre de joueurs** : plafonné à 4 en v1.
