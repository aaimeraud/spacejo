# Mentions légales d'un projet non professionnel et anonyme

## Contexte

SpaceJo est en ligne : la loi française demande à tout site de publier des mentions légales. Le propriétaire
du dépôt édite le site seul, à titre personnel et scolaire, sans aucun revenu, et ne veut pas publier de
données personnelles.

## Décision

- Page « Mentions légales » dans l'interface (`web/src/components/LegalNotice.tsx`), ouverte depuis le pied
  de l'accueil.
- **Éditeur anonyme** : la loi n° 2004-575 du 21 juin 2004 (LCEN), article 1-1 II (créé par la loi SREN du
  21 mai 2024), permet aux « personnes éditant à titre non professionnel » de ne publier que le nom et l'adresse
  de l'hébergeur, « sous réserve d'avoir communiqué à ce fournisseur les éléments d'identification
  personnelle » (nom, prénoms, domicile, téléphone). Le compte Render du propriétaire doit donc porter son
  identité réelle. Contact public : une adresse électronique choisie par le propriétaire pour ce rôle (danielfortune.pro@proton.me),
  ou une demande (« issue ») sur le dépôt GitHub ; les demandes sur les données passent par l'adresse électronique,
  pour ne rien publier.
- **Hébergeurs** : Render Inc. (site et serveur ; adresse de San Francisco relevée sur des annuaires
  d'entreprises, render.com n'en publie pas) et Supabase Pte. Ltd. (base de données, adresse tirée de ses
  conditions d'utilisation ; données en Europe, Irlande).
- **Données** : décrites telles que le code les traite (nom de joueur, état de partie, chat, empreinte du
  jeton ; adresse IP en mémoire pour la limitation de débit, jamais journalisée par le serveur ; stockage local
  du navigateur sans cookie ; Google Fonts et API GitHub appelés par le navigateur).

## Alternatives écartées

- Publier les coordonnées de l'éditeur : refusé par le propriétaire, et non obligatoire pour un éditeur non
  professionnel.
- Bandeau de consentement : inutile, le site ne dépose aucun cookie et ne stocke localement que ce qui est
  strictement nécessaire au service demandé.

## Conséquences

- Les mentions doivent suivre le code : tout nouveau traitement (journal, mesure d'audience, nouveau service
  tiers, changement d'hébergeur) met à jour `LegalNotice.tsx` dans le même commit.
- Limites assumées : pas de suppression automatique des parties, Google Fonts reçoit l'adresse IP des visiteurs
  (pistes dans `IDEAS.md`). Ce texte n'est pas un avis juridique.
