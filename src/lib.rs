use winit::{event_loop::EventLoop, window::WindowAttributes};

pub mod common;
pub mod logic;
pub mod render;

pub use wgpu::include_wgsl;
pub use winit;

use crate::logic::{game::Game, game_window::InputHandler};

pub fn start_engine(
    main_input_handler: Box<dyn InputHandler + Send>,
    window_attributes: WindowAttributes,
) {
    let event_loop = EventLoop::new().unwrap();

    let mut app = Game::new(main_input_handler, window_attributes);

    event_loop.run_app(&mut app).unwrap();
}
