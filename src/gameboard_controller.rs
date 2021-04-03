//! Gameboard controller.

use piston::{Button, GenericEvent, MouseButton};

use crate::{Gameboard, gameboard::PlayerCell};

/// Handles events for Sudoku game.
pub struct GameboardController {
    /// Stores the gameboard state.
    pub gameboard: Gameboard,
    /// The last selected cell, if any.
    pub selected_cell: Option<[usize; 2]>,
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

    /// Set the selected cell, or None if it is out of the grid.
    fn find_selected_cell(&mut self, pos: [f64; 2], size: [f64; 2]) {
        // Find coordinates relative to upper left corner.
        let x = self.cursor_pos[0] - pos[0];
        let y = self.cursor_pos[1] - pos[1];
        // Check that coordinates are inside board boundaries.
        self.selected_cell = if x >= 0.0 && x < size[0] && y >= 0.0 && y < size[1] {
            // Compute the cell position.
            let cell_x = (x / size[0] * (self.gameboard.size[0] as f64)) as usize;
            let cell_y = (y / size[1] * (self.gameboard.size[1] as f64)) as usize;
            Some([cell_x, cell_y])
        } else {
            None
        };
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, pos: [f64; 2], size: [f64; 2], e: &E) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            self.find_selected_cell(pos, size);
            match self.selected_cell {
                Some(ind) => {
                    self.gameboard.set(ind, PlayerCell::Revealed);
                },
                None => {},
            }
        }

        if let Some(Button::Mouse(MouseButton::Right)) = e.press_args() {
            self.find_selected_cell(pos, size);
            match self.selected_cell {
                Some(ind) => {
                    let cell = self.gameboard.get_cell(ind[0], ind[1]);
                    let val = match cell.get_player_cell() {
                        PlayerCell::NotDetermined => PlayerCell::Flagged,
                        PlayerCell::Flagged => PlayerCell::Question,
                        PlayerCell::Question => PlayerCell::NotDetermined,
                        _ => return,
                    };
                    self.gameboard.set(ind, val);
                },
                None => {},
            }
        }
    }
}