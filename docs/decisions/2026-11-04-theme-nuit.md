# Thème « nuit » de l'interface

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

## Alternatives écartées

- Garder le décor spatial en l'épurant : le violet et les dégradés en sont l'essentiel.
- Un thème clair : une partie se joue surtout le soir, et les couleurs des cartes ressortent mieux sur du noir.

## Conséquences

- Le vert citron sert à la fois d'accent et de couleur des cartes 1 à 4 : sur la table, « vert » veut toujours
  dire « bon » ou « actif ». Une carte n'est jamais distinguée par la couleur seule (le chiffre est écrit), et
  un état choisi ou actif se voit aussi par un contour qui apparaît.
- La police est chargée depuis Google Fonts : une requête vers un service tiers au chargement de la page. Pour
  s'en passer, il suffirait d'héberger les fichiers d'Inter dans `web/public/`.
