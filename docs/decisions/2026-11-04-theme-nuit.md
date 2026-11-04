# Thème « nuit » de l'interface, et son pendant clair

## Contexte

Le premier thème était un décor « espace » : fond bleu nuit, violet, dégradés, étoiles et halos lumineux sur
les cartes et le joueur actif. Le propriétaire du dépôt le trouvait daté et a demandé une interface plus
moderne.

## Décision

- **Fond noir neutre**, surfaces séparées par un simple palier de gris (`#000` → `#111` → `#1c1c1c` →
  `#2a2a2a`) : pas de dégradé, pas de halo, pas d'ombre sur ce qui est posé (seuls la notification d'erreur et
  la carte en main, qui « flottent », en gardent une).
- **Une seule couleur d'interface**, un vert citron (`--accent`, `#c6f24e`), réservée à ce qui est actif ou
  choisi : le joueur qui joue, les cartes jouables, l'option sélectionnée, la colonne retirée. Les boutons
  n'en portent pas : le bouton principal est une pilule blanche à texte noir, les autres sont gris ou en
  contour.
- **Les cartes portent les seules autres couleurs**, en aplats, selon leur valeur : bleu (négatif), bleu clair
  (zéro), vert citron (1 à 4), ambre (5 à 8), rouge (9 à 12), chiffre en noir. Le dos est gris foncé avec la
  marque « S ». La valeur reste toujours écrite.
- **Une seule police, Inter** (Google Fonts, graisses 400 à 600), chiffres de même largeur (`tabular-nums`)
  pour les scores et compteurs. Si la police ne se charge pas, la police système prend le relais.
- Libellés en casse normale (plus de petites capitales espacées), coins arrondis : 20 px pour les panneaux,
  8 px pour les cartes, pilules pour les boutons et étiquettes.
- L'accueil montre un éventail de cinq cartes du jeu, une par famille de couleur.

Le style suit la direction « True-black flat » de la compétence de design utilisée (tirage du 2026-11-03,
graine 719397715).

## Thème clair

Ajouté ensuite à la demande du propriétaire du dépôt. Mêmes rôles, valeurs inversées : fond gris clair
(`#ebebeb`), panneaux blancs, texte presque noir, bouton principal noir à texte blanc. Les contours de l'accent
passent à un vert plus foncé (`--accent-line`, `#5f8a00`), car le vert citron ne se voit pas sur du blanc ; il
reste en aplat (« joue », colonne retirée). Les cartes ne changent pas : dos foncés sur une table blanche.

- Par défaut, l'interface suit le réglage du système (`prefers-color-scheme`), et le suit encore s'il change.
- Un bouton soleil / lune (accueil, salon, partie) force l'un ou l'autre ; ce choix est gardé sur l'appareil
  (`localStorage`, clé `spacejo.theme`) et l'emporte sur le système.
- Toutes les couleurs passent par des variables CSS : le thème clair ne redéfinit que ces variables
  (`:root[data-theme="light"]`), sauf les tuiles de l'accueil, blanches sur le fond gris.

## Alternatives écartées

- Garder le décor spatial en l'épurant : le violet et les dégradés en sont l'essentiel.
- Un thème clair seul : une partie se joue souvent le soir, et les couleurs des cartes ressortent mieux sur du noir ; d'où le sombre par défaut quand le système ne dit rien, et le clair en option.

## Conséquences

- Le vert citron sert à la fois d'accent et de couleur des cartes 1 à 4 : sur la table, « vert » veut toujours
  dire « bon » ou « actif ». Une carte n'est jamais distinguée par la couleur seule (le chiffre est écrit), et
  un état choisi ou actif se voit aussi par un contour qui apparaît.
- La police est chargée depuis Google Fonts : une requête vers un service tiers au chargement de la page. Pour
  s'en passer, il suffirait d'héberger les fichiers d'Inter dans `web/public/`.
