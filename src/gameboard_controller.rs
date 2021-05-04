//! Gameboard controller.

use piston::{Button, GenericEvent, MouseButton};

use crate::{Gameboard, gameboard::PlayerCell};

/// The cell.
pub struct Cell {
    /// The row index of the cell.
    pub row: usize,
    /// The column index of the cell.
    pub col: usize,
}

/// Handles events for Sudoku game.
pub struct GameboardController {
    /// Stores the gameboard state.
    pub gameboard: Gameboard,
    /// The last selected cell, if any.
    pub selected_cell: Option<Cell>,
    /// The last mouse cursor position.
    cursor_pos: [f64; 2],
}

impl GameboardController {
    /// Creates a new gameboard controller.
    pub fn new(gameboard: Gameboard) -> Self {
        Self {
            gameboard,
            selected_cell: None,
            cursor_pos: [0.0; 2],
        }
    }

    /// Handles events.
    pub fn event(&mut self, col: usize, row: usize, e: &conrod_core::widget::button::ClickEvent) {
        match e {
            conrod_core::widget::button::ClickEvent::LeftClick => {
                self.selected_cell = Some(Cell { row, col });
                self.gameboard.set(col, row, PlayerCell::Revealed);
            },
            conrod_core::widget::button::ClickEvent::RightClick => {
                self.selected_cell = Some(Cell { row, col });
                match &self.selected_cell {
                    Some(c) => {
                        let cell = self.gameboard.get_cell(c.col, c.row);
                        let val = match cell.get_player_cell() {
                            PlayerCell::NotDetermined => PlayerCell::Flagged,
                            PlayerCell::Flagged => PlayerCell::Question,
                            PlayerCell::Question => PlayerCell::NotDetermined,
                            _ => return,
                        };
                        self.gameboard.set(c.col, c.row, val);
                    },
                    None => {},
                }
            }
        }
    }
}