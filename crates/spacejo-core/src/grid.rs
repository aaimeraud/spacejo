//! La grille d'un joueur : 4 colonnes de 3 cartes, certaines colonnes pouvant être retirées.

use serde::{Deserialize, Serialize};

use crate::cards::CardValue;
use crate::error::GameError;

/// Nombre de lignes de la grille.
pub const GRID_ROWS: usize = 3;
/// Nombre de colonnes de la grille.
pub const GRID_COLUMNS: usize = 4;
/// Nombre de cartes distribuées à chaque joueur.
pub const CARDS_PER_PLAYER: usize = GRID_ROWS * GRID_COLUMNS;

/// Une carte posée dans la grille.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub value: CardValue,
    pub face_up: bool,
}

/// Une colonne est présente (avec ses cartes, de haut en bas) ou retirée du jeu.
/// Enum avec données : la variante `Present` transporte un `Vec<Cell>`.
/// https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Column {
    Present(Vec<Cell>),
    Removed,
}

/// La grille d'un joueur. Les colonnes retirées gardent leur place, pour que les
/// positions (colonne, ligne) ne changent pas au cours de la manche.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Grid {
    pub columns: Vec<Column>,
}

impl Grid {
    /// Pose les cartes face cachée, colonne par colonne (les `GRID_ROWS` premières dans la
    /// colonne 0, etc.).
    pub fn from_cards(cards: Vec<CardValue>) -> Grid {
        let columns = cards
            .chunks(GRID_ROWS)
            .map(|chunk| {
                Column::Present(
                    chunk
                        .iter()
                        .map(|&value| Cell {
                            value,
                            face_up: false,
                        })
                        .collect(),
                )
            })
            .collect();
        Grid { columns }
    }

    /// Carte à une position, ou `InvalidPosition` si la case n'existe pas.
    pub fn cell(&self, column: usize, row: usize) -> Result<&Cell, GameError> {
        // `?` n'est pas utilisable sur une `Option` dans une fonction qui renvoie un `Result` :
        // `ok_or` transforme l'absence en erreur. https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html
        match self.columns.get(column) {
            Some(Column::Present(cells)) => cells.get(row).ok_or(GameError::InvalidPosition),
            _ => Err(GameError::InvalidPosition),
        }
    }

    /// Même chose, avec un emprunt modifiable (`&mut`) pour changer la carte.
    pub fn cell_mut(&mut self, column: usize, row: usize) -> Result<&mut Cell, GameError> {
        match self.columns.get_mut(column) {
            Some(Column::Present(cells)) => cells.get_mut(row).ok_or(GameError::InvalidPosition),
            _ => Err(GameError::InvalidPosition),
        }
    }

    /// Toutes les cartes encore présentes.
    pub fn cells(&self) -> impl Iterator<Item = &Cell> {
        self.columns.iter().flat_map(|column| match column {
            Column::Present(cells) => cells.as_slice(),
            Column::Removed => &[],
        })
    }

    /// Positions (colonne, ligne) de toutes les cartes encore présentes.
    pub fn positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for (c, column) in self.columns.iter().enumerate() {
            if let Column::Present(cells) = column {
                for r in 0..cells.len() {
                    positions.push((c, r));
                }
            }
        }
        positions
    }

    /// Nombre de cartes encore présentes.
    pub fn card_count(&self) -> usize {
        self.cells().count()
    }

    /// Nombre de cartes face cachée.
    pub fn hidden_count(&self) -> usize {
        self.cells().filter(|cell| !cell.face_up).count()
    }

    /// Somme des cartes visibles.
    pub fn visible_sum(&self) -> i32 {
        self.cells().filter(|c| c.face_up).map(|c| c.value).sum()
    }

    /// Valeurs de toutes les cartes encore présentes.
    pub fn values(&self) -> Vec<CardValue> {
        self.cells().map(|cell| cell.value).collect()
    }

    /// Première case cachée (dans l'ordre colonne puis ligne), s'il y en a une.
    pub fn first_hidden(&self) -> Option<(usize, usize)> {
        self.positions()
            .into_iter()
            .find(|&(c, r)| matches!(self.cell(c, r), Ok(cell) if !cell.face_up))
    }

    /// Première case présente, s'il y en a une.
    pub fn first_present(&self) -> Option<(usize, usize)> {
        self.positions().into_iter().next()
    }

    /// Retourne toutes les cartes (fin de manche).
    pub fn reveal_all(&mut self) {
        for column in &mut self.columns {
            if let Column::Present(cells) = column {
                for cell in cells {
                    cell.face_up = true;
                }
            }
        }
    }

    /// Retire chaque colonne complète dont toutes les cartes sont visibles et de même valeur
    /// (docs/RULES.md, « Colonne identique »). Renvoie les cartes retirées, à poser sur la défausse.
    pub fn remove_identical_columns(&mut self) -> Vec<CardValue> {
        let mut removed = Vec::new();
        for column in &mut self.columns {
            if let Column::Present(cells) = column {
                let complete = cells.len() == GRID_ROWS;
                let all_visible = cells.iter().all(|cell| cell.face_up);
                let same_value = cells.iter().all(|cell| cell.value == cells[0].value);
                if complete && all_visible && same_value {
                    removed.extend(cells.iter().map(|cell| cell.value));
                    *column = Column::Removed;
                }
            }
        }
        removed
    }
}
