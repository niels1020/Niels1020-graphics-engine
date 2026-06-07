use winit::event_loop::EventLoop;

pub mod logic;
pub mod render;

pub mod common;

pub use winit;

use crate::logic::{app::Game, game_window::InputHandler};

pub fn start_engine<T: InputHandler>(main_input_handler: T) {
    let event_loop = EventLoop::new().unwrap();

    let mut app = Game::new(main_input_handler);

    event_loop.run_app(&mut app).unwrap();
}
