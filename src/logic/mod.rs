pub mod app;
pub mod game_window;

use winit::window::WindowId;

pub type Commands = Vec<Command>;

pub enum Command {
    CloseWindow(WindowId),
    Exit,
}