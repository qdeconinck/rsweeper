#![deny(missing_docs)]
//! A sweeper game made in Rust.

use glium;
use conrod_core::{Borderable, Labelable, widget::matrix::Elements, widget_ids};

use glium::Surface;
use image;

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    struct Ids {
        master,

        header,
        body,
        footer,

        grid,

        counter,
        cell[],
        cell_label[],
        cell_img[],
    }
}

struct ImageIds {
    blank: conrod_core::image::Id,
    flag: conrod_core::image::Id,
}


fn set_widgets(ref mut ui: conrod_core::UiCell, ids: &mut Ids, img_ids: &mut ImageIds, gc: &mut GameboardController) {
    use conrod_core::{color, widget, Sizeable, Positionable, Widget, Colorable};

    // Construct our main `Canvas` tree.
    widget::Canvas::new()
        .flow_down(&[
            (
                ids.header,
                widget::Canvas::new().color(color::BLUE).length(100.0).pad_bottom(20.0),
            ),
            (
                ids.body,
                widget::Canvas::new().color(color::WHITE).pad_bottom(20.0),
            ),
            (
                ids.footer,
                widget::Canvas::new()
                    .color(color::BLUE)
                    .length(50.0),
                    //.scroll_kids_vertically(),
            ),
        ])
        .set(ids.master, ui);

    // Draw bomb counters.
    let str = match gc.gameboard.state {
        crate::GameState::Lost => format!("BOOM!"),
        crate::GameState::Won => format!("You won!"),
        _ => format!("Left: {}", gc.gameboard.bombs - gc.gameboard.flagged),
    };

    widget::Text::new(&str).middle_of(ids.header).font_size(36).set(ids.counter, ui);

    let grid_wh = ui.wh_of(ids.body).unwrap();
    let grid_size = gc.gameboard.size;
    let square_cell_size = (grid_wh[0] / (grid_size[1] as f64)).min(grid_wh[1] / (grid_size[0] as f64));
    let mut elements = widget::Matrix::new(grid_size[1], grid_size[0])
        .w_h(square_cell_size * (grid_size[1] as f64), square_cell_size * (grid_size[0] as f64))
        .mid_top_of(ids.body)
        .set(ids.grid, ui);
    let mut elements: Elements = elements;
    if ids.cell_label.len() != grid_size[0] * grid_size[1] {
        ids.cell_label.resize(grid_size[0] * grid_size[1], &mut ui.widget_id_generator());
        ids.cell_img.resize(grid_size[0] * grid_size[1], &mut ui.widget_id_generator());
    }

    let label_size = square_cell_size / 2.0;
    let label_size: u32 = if label_size < 12.0 {
        12
    } else if label_size > 48.0 {
        48
    } else {
        label_size as u32
    };

    use gameboard::PlayerCell;
    while let Some(elem) = elements.next(ui) {
        let (r, c) = (elem.row, elem.col);
        let n = c + (r * grid_size[1]);
        let cell = gc.gameboard.get_cell(c, r);

        let enabled = match cell.get_player_cell() {
            PlayerCell::Revealed => false,
            _ => true && !matches!(gc.gameboard.state, GameState::Won | GameState::Lost),
        };

        let (ch, color) = gc.gameboard.char_and_colors(c, r);

        let color = color::rgba(color[0], color[1], color[2], color[3]);
        let mut ch_str = String::new();

        let button = widget::Button::new().color(color).enabled(enabled);
        let mut ch_str = String::new();
        let button = match ch {
            Some((ch, ch_col)) => {
                ch_str = ch.to_string();
                let ch_color = color::rgba(ch_col[0], ch_col[1], ch_col[2], ch_col[3]);
                button.label(&ch_str).label_color(ch_color).label_font_size(label_size)
            },
            None => button,
        };

        // Show a nice flag if needed.
        if let PlayerCell::Flagged = cell.get_player_cell() {
            widget::Image::new(img_ids.flag)
                .w_h(elem.w, elem.h)
                .middle_of(ids.grid)
                .x_position(button.get_x_position(ui))
                .y_position(button.get_y_position(ui))
                //.middle_of(elem.widget_id)
                .set(ids.cell_img[n], ui);
        }

        for event in elem.set(button, ui) {
            gc.event(c, r, &event);
        }
    }
}

enum Request<'a, 'b: 'a> {
    Event {
        event: &'a glium::glutin::event::Event<'b, ()>,
        should_update_ui: &'a mut bool,
        should_exit: &'a mut bool,
    },
    SetUi {
        needs_redraw: &'a mut bool,
    },
    Redraw,
}

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod_core::render::Primitives` to the
/// screen.
///
/// This function simplifies some of the boilerplate involved in limiting the redraw rate in the
/// glutin+glium event loop.
fn run_loop<F>(display: glium::Display, event_loop: glium::glutin::event_loop::EventLoop<()>, mut callback: F) -> !
where
    F: 'static + FnMut(Request, &glium::Display),
{
    let sixteen_ms = std::time::Duration::from_millis(16);
    let mut next_update = None;
    let mut ui_update_needed = false;
    event_loop.run(move |event, _, control_flow| {
        {
            let mut should_update_ui = false;
            let mut should_exit = false;
            callback(
                Request::Event {
                    event: &event,
                    should_update_ui: &mut should_update_ui,
                    should_exit: &mut should_exit,
                },
                &display,
            );
            ui_update_needed |= should_update_ui;
            if should_exit {
                *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                return;
            }
        }

        // We don't want to draw any faster than 60 FPS, so set the UI only on every 16ms, unless:
        // - this is the very first event, or
        // - we didn't request update on the last event and new events have arrived since then.
        let should_set_ui_on_main_events_cleared = next_update.is_none() && ui_update_needed;
        match (&event, should_set_ui_on_main_events_cleared) {
            (glium::glutin::event::Event::NewEvents(glium::glutin::event::StartCause::Init { .. }), _)
            | (glium::glutin::event::Event::NewEvents(glium::glutin::event::StartCause::ResumeTimeReached { .. }), _)
            | (glium::glutin::event::Event::MainEventsCleared, true) => {
                next_update = Some(std::time::Instant::now() + sixteen_ms);
                ui_update_needed = false;

                let mut needs_redraw = false;
                callback(
                    Request::SetUi {
                        needs_redraw: &mut needs_redraw,
                    },
                    &display,
                );
                if needs_redraw {
                    display.gl_window().window().request_redraw();
                } else {
                    // We don't need to redraw anymore until more events arrives.
                    next_update = None;
                }
            }
            _ => {}
        }
        if let Some(next_update) = next_update {
            *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_update);
        } else {
            *control_flow = glium::glutin::event_loop::ControlFlow::Wait;
        }

        // Request redraw if needed.
        match &event {
            glium::glutin::event::Event::RedrawRequested(_) => {
                callback(Request::Redraw, &display);
            }
            _ => {}
        }
    })
}

// Conversion functions for converting between types from glium's version of `winit` and
// `conrod_core`.
conrod_winit::v023_conversion_fns!();

// Load an image from our assets folder as a texture we can draw to the screen.
fn load_image<P>(display: &glium::Display, path: P) -> glium::texture::SrgbTexture2d
where
    P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &rgba_image.into_raw(),
        image_dimensions,
    );
    let texture = glium::texture::SrgbTexture2d::new(display, raw_image).unwrap();
    texture
}


fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 1024;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("RSweeper")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // Construct our `UI`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("FiraSans-Bold.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let mut image_map = conrod_core::image::Map::new();
    let blank_image = load_image(&display, assets.join("blank.png"));
    let flag_image = load_image(&display, assets.join("flag-icon.png"));
    let mut image_ids = ImageIds {
        blank: image_map.insert(blank_image),
        flag: image_map.insert(flag_image),
    };

    // Instantiate the generated list of widget identifiers.
    let mut ids = Ids::new(ui.widget_id_generator());

    let gameboard = Gameboard::new(20, 20, 80);
    let mut gameboard_controller = GameboardController::new(gameboard);
    let gameboard_view_settings = GameboardViewSettings::new(gameboard_controller.gameboard.size);
    let gameboard_view = GameboardView::new(gameboard_view_settings);

    // Poll events from the window.
    run_loop(display, event_loop, move |request, display| {
        match request {
            Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = convert_event(&event, &display.gl_window().window()) {
                    // gameboard_controller.event(gameboard_view.settings.gameboard_position,
                    //     gameboard_view.settings.cell_size, &event);
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            Request::SetUi { needs_redraw } => {
                // Instantiate all widgets in the GUI.
                set_widgets(ui.set_widgets(), &mut ids, &mut image_ids, &mut gameboard_controller);

                *needs_redraw = ui.has_changed();
            }
            Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    })
}

pub use crate::gameboard::{Gameboard, GameState};
pub use crate::gameboard_controller::GameboardController;
pub use crate::gameboard_view::{GameboardView, GameboardViewSettings};

mod gameboard;
mod gameboard_controller;
mod gameboard_view;