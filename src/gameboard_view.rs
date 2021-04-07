//! Gameboard view.

use graphics::{CharacterCache, Context, Graphics, Image, Line, Rectangle, Text, Transformed, types::Color};

use crate::GameboardController;

/// Stores gameboard view settings.
pub struct GameboardViewSettings {
    /// Position of the gameboard from left-top corner.
    pub gameboard_position: [f64; 2],
    /// Position of the left bombs counter from left-top corner.
    pub bombs_left_position: [f64; 2],
    /// Size of gameboard along horizontal and vertical edge.
    // pub size: [f64; 2],
    /// Size of a single cell along horizontal and vertical edges.
    pub cell_size: [f64; 2],
    /// Background color.
    pub background_color: Color,
    /// Border color.
    pub border_color: Color,
    /// Edge color around the whole board.
    pub board_edge_color: Color,
    /// Edge color between the 3x3 section.
    pub section_edge_color: Color,
    /// Edge color between cells.
    pub cell_edge_color: Color,
    /// Edge radius around the whole board.
    pub board_edge_radius: f64,
    /// Edge radius between the 3x3 sections.
    pub section_edge_radius: f64,
    /// Edge radius between cells.
    pub cell_edge_radius: f64,
    /// Selected cell background color.
    pub selected_cell_background_color: Color,
    /// Text color.
    pub text_color: Color,
}

impl GameboardViewSettings {
    /// Creates new gameboard view settings.
    pub fn new(gameboard_size: [usize; 2]) -> Self {
        let cell_size = [30.0; 2];
        let bombs_left_x =  (gameboard_size[0] as f64 * cell_size[0]) - 150.0;
        Self {
            gameboard_position: [10.0, 100.0],
            bombs_left_position: [bombs_left_x, 60.0],
            cell_size: [30.0, 30.0],
            background_color: [0.8, 0.8, 1.0, 1.0],
            border_color: [0.0, 0.0, 0.2, 1.0],
            board_edge_color: [0.0, 0.0, 0.2, 1.0],
            section_edge_color: [0.0, 0.0, 0.2, 1.0],
            cell_edge_color: [0.0, 0.0, 0.2, 1.0],
            board_edge_radius: 3.0,
            section_edge_radius: 2.0,
            cell_edge_radius: 1.0,
            selected_cell_background_color: [0.9, 0.9, 1.0, 1.0],
            text_color: [0.0, 0.0, 0.1, 1.0],
        }
    }
}

/// Stores visual informatin about a gameboard.
pub struct GameboardView {
    /// Stores gameboard view settings.
    pub settings: GameboardViewSettings,
}

impl GameboardView {
    /// Creates a new gameboard view.
    pub fn new(settings: GameboardViewSettings) -> Self {
        Self {
            settings,
        }
    }

    /// Draw the gameboard.
    pub fn draw<G: Graphics, C>(
        &self,
        controller: &GameboardController,
        glyphs: &mut C,
        c: &Context,
        g: &mut G,
    )
    where
        C: CharacterCache<Texture=G::Texture>,
    {
        let ref settings = self.settings;
        let ref gameboard = controller.gameboard;
        let gameboard_size = [
            settings.cell_size[0] * (gameboard.size[0] as f64),
            settings.cell_size[1] * (gameboard.size[1] as f64),
        ];
        let board_rect = [
            settings.gameboard_position[0], settings.gameboard_position[1],
            gameboard_size[0], gameboard_size[1],
        ];

        let text = Text::new(30);
        let bombs_counter_rect = [
            settings.bombs_left_position[0], settings.bombs_left_position[1],
            140.0, 140.0,
        ];

        // Draw bomb counters.
        let str = match gameboard.state {
            crate::GameState::Lost => format!("BOOM!"),
            crate::GameState::Won => format!("You won!"),
            _ => format!("Left: {}", gameboard.bombs - gameboard.flagged),
        };

        let _ = text.draw(&str, glyphs, &c.draw_state, c.transform.trans(bombs_counter_rect[0], bombs_counter_rect[1]), g);
        // Draw board background.
        Rectangle::new(settings.background_color)
            .draw(board_rect, &c.draw_state, c.transform, g);

        // Declare the format for cell and section lines.
        let cell_edge = Line::new(settings.cell_edge_color, settings.cell_edge_radius);
        let flag_pole = Line::new([0.65, 0.16, 0.16, 1.00], 2.0);
        let flag = Line::new([1.00, 0.00, 0.00, 1.00], 1.0);

        let x_size = gameboard_size[0] / (gameboard.size[0] as f64);
        let y_size = gameboard_size[1] / (gameboard.size[1] as f64);
        for cell_y in 0..gameboard.size[1] {
            for cell_x in 0..gameboard.size[0] {
                let (ch, bg_color) = gameboard.char_and_colors([cell_x, cell_y]);

                let x = settings.gameboard_position[0] + (cell_x as f64) * x_size;
                let y = settings.gameboard_position[1] + (cell_y as f64) * y_size;
                let x2 = x + x_size;
                let y2 = y + y_size;

                let vline = [x, y, x, y2];
                let hline = [x, y, x2, y];

                // Draw background
                let cell_rect = [
                    x, y,
                    x_size, y_size,
                ];
                Rectangle::new(bg_color)
                    .draw(cell_rect, &c.draw_state, c.transform, g);

                // Draw lines
                cell_edge.draw(vline, &c.draw_state, c.transform, g);
                cell_edge.draw(hline, &c.draw_state, c.transform, g);

                // If there is a char, draw it.
                if let Some((ch, ch_color)) = ch {
                    let text_image = Image::new_color(ch_color);
                    let pos = [
                        x,
                        y,
                    ];
                    if let Ok(character) = glyphs.character(26, ch) {
                        match ch {
                            'F' => {
                                // Draw a nice flag.
                                let f_x = pos[0] + 6.0;
                                let f_y = pos[1] + 2.0;
                                let pole_pos = [f_x, f_y, f_x, f_y + settings.cell_size[1] - 4.0];
                                flag_pole.draw(pole_pos, &c.draw_state, c.transform, g);
                                // The flag itself.
                                let base_y = f_y + settings.cell_size[1] / 3.0;
                                let max_i = 2 * ((settings.cell_size[1] / 3.0) as i64) - 3;
                                for i in (1..max_i).rev() {
                                    let flag_line = [f_x + 2.0 + ((max_i - i) as f64), base_y + (i as f64) / 2.0, f_x + 2.0 + ((max_i - i) as f64), base_y - (i as f64) / 2.0];
                                    flag.draw(flag_line, &c.draw_state, c.transform, g);
                                }
                            },
                            _ => {
                                let ch_x = pos[0] + (settings.cell_size[0] - character.atlas_size[0]) / 2.0;
                                let ch_y = pos[1] + (settings.cell_size[1] - character.atlas_size[1]) / 2.0;
                                let text_image = text_image.src_rect([
                                    character.atlas_offset[0],
                                    character.atlas_offset[1],
                                    character.atlas_size[0],
                                    character.atlas_size[1],
                                ]);
                                text_image.draw(character.texture,
                                                &c.draw_state,
                                                c.transform.trans(ch_x, ch_y),
                                                g);
                                    }
                        }
                    }
                }
            }
        }

        // Draw board edge.
        Rectangle::new_border(settings.board_edge_color, settings.board_edge_radius)
            .draw(board_rect, &c.draw_state, c.transform, g);
    }
}