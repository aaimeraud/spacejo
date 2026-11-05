# Durées de conservation et page « Confidentialité »

## Contexte

Sans compte ni historique, garder les parties et les chats indéfiniment n'avait aucune utilité et allait contre le
principe de limitation de la conservation du RGPD (art. 5.1.e). Le propriétaire du dépôt a demandé d'effacer les chats
après chaque partie, de supprimer les parties après une journée, et de réunir sur une page tout ce qu'il faut dire aux
joueurs sur leurs données.

## Décision

- **Chat** : effacé 15 minutes après la fin de la partie (`CHAT_KEPT_AFTER_GAME_OVER`).
- **Partie** : supprimée 24 heures après sa dernière écriture (`GAME_KEPT_WHEN_IDLE`), avec sa présence, ses messages et
  ses réactions (`ON DELETE CASCADE`).
- **Ménage au passage** (démarrage, création, jonction) : le serveur ne lance aucune tâche de fond (règle du projet).
- **Page « Confidentialité »** (`PrivacyPolicy.tsx`), distincte des mentions légales, qui ne gardent que l'éditeur,
  l'hébergeur, la propriété intellectuelle et un renvoi. Contenu : en bref, responsable du traitement, une fiche par
  donnée enregistrée (finalité, base légale, durée), journaux, stockage dans le navigateur (pas de cookie),
  destinataires et transferts, sécurité, droits et CNIL, ce que le site ne fait pas, évolutions.
- **Base légale** : intérêt légitime (art. 6.1.f) pour faire fonctionner le jeu demandé et protéger le service. Le
  stockage local est strictement nécessaire (loi Informatique et Libertés, art. 82) : pas de bandeau de consentement.

## Alternatives écartées

- **Tâche planifiée** (cron Render ou `pg_cron` de Supabase) : plus régulière, mais une pièce de plus à déployer et à
  surveiller, contraire à la règle « pas d'état ni de tâche entre deux requêtes ».
- **Tout dans les mentions légales** : une seule page, mais longue, et qui mélange deux obligations différentes (LCEN
  pour l'éditeur, RGPD pour les données).
- **Base légale « contrat »** (art. 6.1.b) : il n'y a ni compte ni conditions d'utilisation acceptées.

## Conséquences

- Une partie inactive depuis 24 heures disparaît : un joueur qui revient la trouve supprimée (`404 game_not_found`).
- Les durées existent à deux endroits (`src/handlers.rs` qui les applique, `web/src/config.ts` qui les annonce) : les
  changer, c'est changer les deux, et la date de mise à jour de la page.
- À faire si le site évolue : un classement ou des comptes demanderont de nouvelles fiches, une nouvelle durée de
  conservation et sans doute une autre base légale.
- Points laissés au propriétaire du dépôt : l'identité du responsable du traitement (le RGPD, art. 13.1.a, demande
  l'identité et les coordonnées ; la page ne donne que l'adresse de contact) ; l'hébergement de la police Inter sur le
  site, qui éviterait d'envoyer l'adresse IP des visiteurs à Google.
