use winit::{event_loop::ActiveEventLoop, window::WindowId};

use crate::logic::{
    app::Game,
    game_window::{GameWindow, InputHandler},
};

pub(crate) enum Command {
    CloseWindow(WindowId),
    Exit,
    NewWindow(Box<dyn InputHandler>),
}

pub struct Commands {
    pub(crate) queue: Vec<Command>,
}

impl Commands {
    pub fn close_window(&mut self, id: WindowId) {
        self.queue.push(Command::CloseWindow(id));
    }

    pub fn exit(&mut self) {
        self.queue.push(Command::Exit);
    }

    pub fn new_window(&mut self, input_handler: Box<dyn InputHandler>) {
        self.queue.push(Command::NewWindow(input_handler));
    }

    pub fn new() -> Self {
        Self { queue: vec![] }
    }
}

pub(crate) fn run_command(
    event_loop: &ActiveEventLoop,
    game: &mut Game,
    command: Command,
) {
    match command {
        Command::CloseWindow(window_id) => {
            game.windows.retain(|window| {
                if let Some(id) = window.game_info.window_id {
                    id != window_id
                } else {
                    true
                }
            });
            if game.windows.is_empty() {
                println!("No windows open: Exiting");
                event_loop.exit();
            }
        }
        Command::Exit => event_loop.exit(),
        Command::NewWindow(input_handler) => {
            game.windows.push(GameWindow::new(input_handler));
            let len = game.windows.len();
            game.windows
                .get_mut(len - 1)
                .unwrap()
                .start(&mut game.commands, event_loop);
        }
    }
}
