//! Game board logic.

use std::cmp::min;

use graphics::types::Color;
use rand::{self, Rng};

/// The different values of a cell from the user.
#[derive(Clone, Copy, Debug)]
pub enum PlayerCell {
    /// Not determined yet, the default value.
    NotDetermined,
    /// Flagged as containing a bomb.
    Flagged,
    /// Question, not flagged but show a question mark on the cell.
    Question,
    /// Revealed, show either the value or the bomb.
    Revealed,
}

impl Default for PlayerCell {
    fn default() -> Self {
        Self::NotDetermined
    }
}

/// The actual content of the cell.
#[derive(Clone, Copy, Debug)]
pub enum CellContent {
    /// Nothing, but indicates the number of bombs directly around it.
    Nothing(u8),
    /// A bomb.
    Bomb,
}

impl Default for CellContent {
    fn default() -> Self {
        Self::Nothing(0)
    }
}

/// A sweeper cell, containing information about its real value and what the
/// player thinks about it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cell {
    /// The interaction that the player has with the cell.
    player: PlayerCell,
    /// The actual content of the cell.
    content: CellContent,
}

impl Cell {
    pub fn get_player_cell(&self) -> PlayerCell {
        self.player
    }
}

/// Indicates the game state.
#[derive(Copy, Clone, Debug)]
pub enum GameState {
    /// The initial status, the player did not interaction yet with the board.
    /// The game stays in this state as long as the player did not revealed any
    /// celll. This prevents the player from loosing at the first revealed cell,
    /// when no information about the board is available.
    Initial,
    /// The state where the game takes place. Bomb locations are determined and
    /// the player did not revealed one of them yet.
    Alive,
    /// The player flagged all the bombs and revealed all the other cells, the
    /// game is won.
    Won,
    /// The player revealed a bomb, the game is over.
    Lost,
}

/// Stores game board information.
pub struct Gameboard {
    /// The size of the gameboard (cols, rows).
    pub size: [usize; 2],
    /// The number of bombs in the game.
    pub bombs: usize,
    /// The number of cells flagged by the player.
    pub flagged: usize,
    /// Indicates the game state.
    pub state: GameState,
    /// The game cells.
    cells: Vec<Vec<Cell>>,
}

const BOMB_BACKGROUND: Color = [0.9, 0.0, 0.0, 1.0];
const WRONG_BACKGROUND: Color = [0.5, 0.0, 0.5, 1.0];
const ND_BACKGROUND: Color = [1.0, 1.0, 1.0, 1.0];
const REV_BACKGROUND: Color = [0.7, 0.7, 0.7, 1.0];
const QUESTION_BACKGROUND: Color = [0.7, 0.7, 1.0, 1.0];
const FLAGGED_BACKGROUND: Color = [1.0, 0.64, 0.0, 1.0];
const BLACK: Color = [0.0, 0.0, 0.1, 1.0];

const ONE_COLOR: Color = [0.0, 0.0, 1.0, 1.0];
const TWO_COLOR: Color = [0.0, 1.0, 0.0, 1.0];
const THREE_COLOR: Color = [1.0, 0.0, 0.0, 1.0];
const FOUR_COLOR: Color = [0.675, 0.4875, 0.8, 1.0];
const FIVE_COLOR: Color = [0.64, 0.16, 0.16, 1.0];
const SIX_COLOR: Color = [0.5, 1.0, 0.5, 1.0];
const SEVEN_COLOR: Color = [0.9, 0.8, 1.0, 1.0];
const EIGTH_COLOR: Color = [1.0, 0.6, 0.6, 1.0];

impl Gameboard {
    /// Creates a new game board.
    pub fn new(cols: usize, rows: usize, bombs: usize) -> Self {
        assert!(rows * cols > bombs, "Too many bombs to be placed");
        Self {
            size: [cols, rows],
            bombs,
            flagged: 0,
            state: GameState::Initial,
            cells: vec![vec![Cell::default(); cols]; rows],
        }
    }

    fn count_neighbor_bombs(&self, col: usize, raw: usize) -> u8 {
        let mut res = 0;
        for ny in raw.saturating_sub(1)..=min(raw + 1, self.size[1] - 1) {
            for nx in col.saturating_sub(1)..=min(col + 1, self.size[0] - 1) {
                // We do not handle ourselve, but if we are a bomb this has
                // no much sense.
                match self.get_cell(nx, ny).content {
                    CellContent::Bomb => res += 1,
                    _ => {},
                }
            }
        }
        res
    }

    /// Gets a immutable reference to a Cell.
    pub fn get_cell(&self, col: usize, row: usize) -> &Cell {
        & self.cells[row][col]
    }

    /// Gets a mutable reference to a Cell.
    pub fn get_mut_cell(&mut self, col: usize, row: usize) -> &mut Cell {
        &mut self.cells[row][col]
    }

    /// Returns true if the coordinates of 1 are a direct neighbour of 2 (also if 1 is 2).
    fn is_neighbour(&self, col1: usize, row1: usize, col2: usize, row2: usize) -> bool {
        col1.saturating_sub(1) <= col2
            && col2 <= col1 + 1
            && row1.saturating_sub(1) <= row2
            && row2 <= row1 + 1
    }

    /// Initialize the cells, with the player initial revealed cell.
    fn init(&mut self, rcol: usize, rrow: usize) {
        println!("Starting init");
        // This is very unefficient to do so, but anyway.
        let mut rng = rand::thread_rng();
        let mut placed = 0;
        while placed < self.bombs {
            let col = rng.gen_range(0..self.size[0]);
            let row = rng.gen_range(0..self.size[1]);
            // Place a bomb only if
            // 1) the cell was not revealed by the player
            // 2) the cell is not a neighbour of the one revealed by the player
            // 3) no previous bomb was there
            if !self.is_neighbour(rcol, rrow, col, row) {
                let cell = self.get_mut_cell(col, row);
                if let CellContent::Nothing(_) = cell.content {
                    cell.content = CellContent::Bomb;
                    placed += 1;
                }
            }
        }
        println!("Bombs placed");

        // And now compute the neighbors.
        for row in 0..self.size[1] {
            for col in 0..self.size[0] {
                let cell = self.get_cell(col, row);
                match cell.content {
                    CellContent::Nothing(_) => {
                        let new_val = self.count_neighbor_bombs(col, row);
                        let cell = self.get_mut_cell(col, row);
                        cell.content = CellContent::Nothing(new_val);
                    },
                    CellContent::Bomb => {},
                }
            }
        }

        // Now the game starts!
        self.state = GameState::Alive;
        println!("Init done!");
    }

    /// Update the state of the gameboard.
    fn update_state(&mut self, col: usize, row: usize) {
        // The state is only updatable when being alive.
        if let GameState::Alive = self.state {
            let cell = self.get_cell(col, row);

            // Did the player lost?
            if let PlayerCell::Revealed = cell.player {
                if let CellContent::Bomb = cell.content {
                    // Too bad!
                    self.state = GameState::Lost;
                    println!("Too bad, you lost!");
                    return;
                }
            }

            // Did the player won?
            // Actually, we can just look at player views, if we only have
            // Revealed and exactly `bombs` Flagged, the player wins.
            let mut flagged = 0;
            let mut over = true;
            for nrow in 0..self.size[1] {
                for ncol in 0..self.size[0] {
                    match self.get_cell(ncol, nrow).player {
                        PlayerCell::Flagged => {
                            flagged += 1;
                            if flagged > self.bombs {
                                // Too many flags, not won!
                                over = false;
                            }
                        },
                        PlayerCell::Revealed => {},
                        // If some are not Revealed nor Flagged, then the game
                        // is not over.
                        _ => over = false,
                    }
                }
            }

            self.flagged = flagged;
            if over && self.flagged == self.bombs {
                // If we arrive here, it means the player won!
                self.state = GameState::Won;
                println!("Hoora, you won!");
            }
        }

    }

    fn reveal_with_no_neighbors(&mut self, col: usize, row: usize) {
        for nrow in row.saturating_sub(1)..=min(row + 1, self.size[1] - 1) {
            for ncol in col.saturating_sub(1)..=min(col + 1, self.size[0] - 1) {
                // Only handle cells that are not revealed, otherwise we will
                // loop forever.
                if let PlayerCell::Revealed = self.get_cell(ncol, nrow).player {
                    continue;
                }
                self.get_mut_cell(ncol, nrow).player = PlayerCell::Revealed;
                match self.get_cell(ncol, nrow).content {
                    CellContent::Nothing(0) => self.reveal_with_no_neighbors(ncol, nrow),
                    _ => {},
                }
            }
        }
    }

    /// Sets the player input.
    pub fn set(&mut self, col: usize, row: usize, val: PlayerCell) {
        if let GameState::Initial = self.state {
            // If the game is in Initial state and the value is not a Revealed
            // one, do nothing.
            match val {
                PlayerCell::Revealed => {
                    // Record that we revealed a cell, and then determine the
                    // bomb positions.
                    let cell = self.get_mut_cell(col, row);
                    cell.player = PlayerCell::Revealed;
                    self.init(col, row);
                    // Only perform the optimization if the player has some luck.
                    match self.get_cell(col, row).content {
                        CellContent::Nothing(0) => self.reveal_with_no_neighbors(col, row),
                        _ => {},
                    }
                }
                _ => {}
            }
            return;
        }

        // We can only set something if we are in the Alive state.
        if let GameState::Alive = self.state {
            // If we try to place a flag while we are at the right number of
            // bombs, do nothing.
            if let PlayerCell::Flagged = val {
                if self.flagged >= self.bombs {
                    return;
                }
            }

            let cell = self.get_mut_cell(col, row);
            // If the cell is Revealed, nothing to do.
            if let PlayerCell::Revealed = cell.player {
                return;
            }

            // Ok, then something should probably be set.
            cell.player = val;

            // Add the optimization to reduce the number of clicks.
            if let PlayerCell::Revealed = val {
                match cell.content {
                    CellContent::Nothing(0) => self.reveal_with_no_neighbors(col, row),
                    _ => {},
                }
            }

            // Finally, check the game status looking at the last cell touched.
            self.update_state(col, row);
        }
    }

    fn get_neighbours(&self, col: usize, row: usize) -> (Option<(char, Color)>, Color) {
        // If we reveal the input, we should only have nothing
        // in the cell.
        let cell = self.get_cell(col, row);
        match cell.content {
            CellContent::Nothing(v) => match v {
                0 => (None, REV_BACKGROUND),
                1 => (Some(('1', ONE_COLOR)), REV_BACKGROUND),
                2 => (Some(('2', TWO_COLOR)), REV_BACKGROUND),
                3 => (Some(('3', THREE_COLOR)), REV_BACKGROUND),
                4 => (Some(('4', FOUR_COLOR)), REV_BACKGROUND),
                5 => (Some(('5', FIVE_COLOR)), REV_BACKGROUND),
                6 => (Some(('6', SIX_COLOR)), REV_BACKGROUND),
                7 => (Some(('7', SEVEN_COLOR)), REV_BACKGROUND),
                8 => (Some(('8', EIGTH_COLOR)), REV_BACKGROUND),
                // Not possible to have more than 8
                _ => panic!("more than 8 bombs???"),
            },
            _ => (None, REV_BACKGROUND),
        }
    }

    /// Gets the character with its own font and background color at cell location.
    /// TODO: pictures.
    pub fn char_and_colors(&self, col: usize, row: usize) -> (Option<(char, Color)>, Color) {
        let cell = self.get_cell(col, row);
        match self.state {
            GameState::Lost => {
                // If we lost, reveal the bomb positions.
                match cell.content {
                    CellContent::Nothing(_) => match cell.player {
                        PlayerCell::Revealed => self.get_neighbours(col, row),
                        PlayerCell::Flagged => (Some(('X', BLACK)), WRONG_BACKGROUND),
                        _ => (None, ND_BACKGROUND),
                    },
                    CellContent::Bomb => match cell.player {
                        PlayerCell::Revealed => (Some(('B', BLACK)), WRONG_BACKGROUND),
                        PlayerCell::Flagged => (Some(('F', BLACK)), FLAGGED_BACKGROUND),
                        _ => (Some(('B', BLACK)), BOMB_BACKGROUND),
                    }
                }
            },
            _ => {
                // In other states, show the player input.
                match cell.player {
                    PlayerCell::NotDetermined => (None, ND_BACKGROUND),
                    PlayerCell::Flagged => (Some(('F', BLACK)), FLAGGED_BACKGROUND),
                    PlayerCell::Question => (Some(('?', BLACK)), QUESTION_BACKGROUND),
                    PlayerCell::Revealed => {
                        self.get_neighbours(col, row)
                    }
                }
            }
        }
    }
}