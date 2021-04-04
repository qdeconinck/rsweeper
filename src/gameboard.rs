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
    /// The size of the gameboard.
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
const ND_BACKGROUND: Color = [1.0, 1.0, 1.0, 1.0];
const REV_BACKGROUND: Color = [0.7, 0.7, 0.7, 1.0];
const FLAGGED_BACKGROUND: Color = [1.0, 0.64, 0.0, 1.0];
const BLACK: Color = [0.0, 0.0, 0.1, 1.0];

const ONE_COLOR: Color = [0.0, 0.0, 1.0, 1.0];
const TWO_COLOR: Color = [0.0, 1.0, 0.0, 1.0];
const THREE_COLOR: Color = [1.0, 0.0, 0.0, 1.0];
const FOUR_COLOR: Color = [0.875, 0.6875, 1.0, 1.0];
const FIVE_COLOR: Color = [0.64, 0.16, 0.16, 1.0];
const SIX_COLOR: Color = [0.5, 1.0, 0.5, 1.0];
const SEVEN_COLOR: Color = [0.9, 0.8, 1.0, 1.0];
const EIGTH_COLOR: Color = [1.0, 0.6, 0.6, 1.0];

impl Gameboard {
    /// Creates a new game board.
    pub fn new(size: [usize; 2], bombs: usize) -> Self {
        assert!(size[0] * size[1] > bombs, "Too many bombs to be placed");
        Self {
            size,
            bombs,
            flagged: 0,
            state: GameState::Initial,
            cells: vec![vec![Cell::default(); size[1]]; size[0]],
        }
    }

    fn count_neighbor_bombs(&self, x: usize, y: usize) -> u8 {
        let mut res = 0;
        for ny in y.saturating_sub(1)..=min(y + 1, self.size[1] - 1) {
            for nx in x.saturating_sub(1)..=min(x + 1, self.size[0] - 1) {
                // We do not handle ourselve, but if we are a bomb this has
                // no much sense.
                match self.get_cell(nx, ny).content {
                    CellContent::Bomb => res += 1,
                    _ => {},
                }
            }
        }
        println!("res is {}", res);
        res
    }

    /// Gets a immutable reference to a Cell.
    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        & self.cells[y][x]
    }

    /// Gets a mutable reference to a Cell.
    pub fn get_mut_cell(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y][x]
    }

    /// Initialize the cells.
    fn init(&mut self) {
        println!("Starting init");
        // This is very unefficient to do so, but anyway.
        let mut rng = rand::thread_rng();
        let mut placed = 0;
        while placed < self.bombs {
            let x = rng.gen_range(0..self.size[0]);
            let y = rng.gen_range(0..self.size[1]);
            let cell = self.get_mut_cell(x, y);
            // Place a bomb only if
            // 1) the cell was not revealed by the player
            // 2) no previous bomb was there
            if let CellContent::Nothing(_) = cell.content {
                match cell.player {
                    PlayerCell::Revealed => {},
                    _ => {
                        cell.content = CellContent::Bomb;
                        placed += 1;
                    }
                }
            }
        }
        println!("Bomb placed");

        // And now compute the neighbors.
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let cell = self.get_cell(x, y);
                match cell.content {
                    CellContent::Nothing(_) => {
                        let new_val = self.count_neighbor_bombs(x, y);
                        let cell = self.get_mut_cell(x, y);
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
    fn update_state(&mut self, ind: [usize; 2]) {
        // The state is only updatable when being alive.
        if let GameState::Alive = self.state {
            let cell = self.get_cell(ind[0], ind[1]);

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
            for y in 0..self.size[1] {
                for x in 0..self.size[0] {
                    match self.get_cell(x, y).player {
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

    fn reveal_with_no_neighbors(&mut self, x: usize, y: usize) {
        for ny in y.saturating_sub(1)..=min(y + 1, self.size[1] - 1) {
            for nx in x.saturating_sub(1)..=min(x + 1, self.size[0] - 1) {
                // Only handle cells that are not revealed, otherwise we will
                // loop forever.
                if let PlayerCell::Revealed = self.get_cell(nx, ny).player {
                    continue;
                }
                self.get_mut_cell(nx, ny).player = PlayerCell::Revealed;
                match self.get_cell(nx, ny).content {
                    CellContent::Nothing(0) => self.reveal_with_no_neighbors(nx, ny),
                    _ => {},
                }
            }
        }
    }

    /// Sets the player input.
    pub fn set(&mut self, ind: [usize; 2], val: PlayerCell) {
        if let GameState::Initial = self.state {
            // If the game is in Initial state and the value is not a Revealed
            // one, do nothing.
            match val {
                PlayerCell::Revealed => {
                    // Record that we revealed a cell, and then determine the
                    // bomb positions.
                    let cell = self.get_mut_cell(ind[0], ind[1]);
                    cell.player = PlayerCell::Revealed;
                    self.init();
                    // Only perform the optimization if the player has some luck.
                    match self.get_cell(ind[0], ind[1]).content {
                        CellContent::Nothing(0) => self.reveal_with_no_neighbors(ind[0], ind[1]),
                        _ => {},
                    }
                }
                _ => {}
            }
            return;
        }

        // We can only set something if we are in the Alive state.
        if let GameState::Alive = self.state {
            let cell = self.get_mut_cell(ind[0], ind[1]);
            // If the cell is Revealed, nothing to do.
            if let PlayerCell::Revealed = cell.player {
                return;
            }

            // Ok, then something should probably be set.
            cell.player = val;

            // Add the optimization to reduce the number of clicks.
            if let PlayerCell::Revealed = val {
                match cell.content {
                    CellContent::Nothing(0) => self.reveal_with_no_neighbors(ind[0], ind[1]),
                    _ => {},
                }
            }

            // Finally, check the game status looking at the last cell touched.
            self.update_state(ind);
        }
    }

    /// Gets the character with its own font and background color at cell location.
    /// TODO: pictures.
    pub fn char_and_colors(&self, ind: [usize; 2]) -> (Option<(char, Color)>, Color) {
        let cell = self.get_cell(ind[0], ind[1]);
        match self.state {
            GameState::Lost => {
                // If we lost, reveal the bomb positions.
                match cell.content {
                    CellContent::Nothing(_) => return (None, ND_BACKGROUND),
                    CellContent::Bomb => (Some(('B', BLACK)), BOMB_BACKGROUND),
                }
            },
            _ => {
                // In other states, show the player input.
                match cell.player {
                    PlayerCell::NotDetermined => (None, ND_BACKGROUND),
                    PlayerCell::Flagged => (Some(('F', BLACK)), FLAGGED_BACKGROUND),
                    PlayerCell::Question => (Some(('?', BLACK)), ND_BACKGROUND),
                    PlayerCell::Revealed => {
                        // If we reveal the input, we should only have nothing
                        // in the cell.
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
                }
            }
        }
    }
}