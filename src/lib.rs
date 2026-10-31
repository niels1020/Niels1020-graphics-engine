use log::info;
use winit::{event_loop::EventLoop, window::WindowAttributes};

pub mod common;
pub mod logic;
pub mod render;
pub mod utils;

pub use egui;
pub use image;
pub use log;
pub use nalgebra;
pub use wgpu::include_wgsl;
pub use winit;

use crate::logic::{engine::Engine, game_window::InputHandler};

/// # main entry point
/// this is the main entry point of the engine
/// # parameters
/// - main_input_hanlder: the input handler for the first window to be created
/// - window_attributes: the atributes of the first window (derived from winit see winit docs for usages)
/// # Example
///
/// ```no_run
/// use niels1020_graphics_engine::{start_engine, logic::{game_window::InputHandler, commands::GlobalCommands, game_window::GameInfo}, winit::{window, event::WindowEvent}};
///
/// pub struct Input{};
///
/// impl InputHandler for Input {
///     fn update(&mut self, _: GlobalCommands, _: &mut GameInfo, _: f64) { }
///     fn start(&mut self, c: GlobalCommands, _: &mut GameInfo) { c.exit() }
///     fn exit(&mut self, _: GlobalCommands, _: &mut GameInfo) {  }
///     fn receive_request(&mut self, _: GlobalCommands, _: &mut GameInfo, _: niels1020_graphics_engine::logic::commands::Request) {  }
///     fn gui(&mut self, _: GlobalCommands, _: &mut GameInfo, _: niels1020_graphics_engine::egui::Context) {  }
///     fn window_event(&mut self, _: GlobalCommands, _: &mut GameInfo, _: niels1020_graphics_engine::winit::event::WindowEvent, _: bool) { }
/// }
///
/// start_engine(Box::new(Input{}), window::WindowAttributes::default());
/// ```
pub fn start_engine(
    main_input_handler: Box<dyn InputHandler + Send>,
    window_attributes: WindowAttributes,
) {
    pretty_env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let mut app = Engine::new(main_input_handler, window_attributes);

    event_loop.run_app(&mut app).unwrap();
    info!("exit");
}
