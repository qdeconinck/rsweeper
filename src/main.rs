#![deny(missing_docs)]
//! A sweeper game made in Rust.

use glutin_window::GlutinWindow;
use graphics::clear;
use piston::{EventLoop, EventSettings, Events, RenderEvent, window::WindowSettings};
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};

fn main() {
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("RSweeper", [1024;2])
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: GlutinWindow = settings.build()
        .expect("could not create window");
    
    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let gameboard_size = [10, 10];
    let gameboard = Gameboard::new(gameboard_size, 10);
    let mut gameboard_controller = GameboardController::new(gameboard);
    let gameboard_view_settings = GameboardViewSettings::new();
    let gameboard_view = GameboardView::new(gameboard_view_settings);

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let ref mut glyphs = GlyphCache::new("assets/FiraSans-Bold.ttf", (), texture_settings)
        .expect("cannot load font");

    while let Some(e) = events.next(&mut window) {
        gameboard_controller.event(gameboard_view.settings.position,
            gameboard_view.settings.cell_size, &e);
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                clear([1.0; 4], g);
                gameboard_view.draw(&gameboard_controller, glyphs, &c, g);
            });
        }
    }
}

pub use crate::gameboard::Gameboard;
pub use crate::gameboard_controller::GameboardController;
pub use crate::gameboard_view::{GameboardView, GameboardViewSettings};

mod gameboard;
mod gameboard_controller;
mod gameboard_view;