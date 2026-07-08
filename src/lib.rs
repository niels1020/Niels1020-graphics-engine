use winit::event_loop::EventLoop;

pub mod common;
pub mod logic;
pub mod render;

pub use wgpu::include_wgsl;
pub use winit;

use crate::logic::{game::Game, game_window::InputHandler};

pub fn start_engine(main_input_handler: Box<dyn InputHandler + Send>) {
    let event_loop = EventLoop::new().unwrap();

    let mut app = Game::new(main_input_handler);

    event_loop.run_app(&mut app).unwrap();
}
