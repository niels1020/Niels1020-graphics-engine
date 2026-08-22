use winit::{
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::logic::{
    engine::Engine,
    game_window::{GameWindow, InputHandler},
};

pub(crate) enum Command {
    CloseWindow(WindowId),
    Exit,
    NewWindow(Box<dyn InputHandler + Send>, WindowAttributes),
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

    pub fn new_window(
        &mut self,
        input_handler: Box<dyn InputHandler + Send>,
        window_attributes: WindowAttributes,
    ) {
        self.queue
            .push(Command::NewWindow(input_handler, window_attributes));
    }

    //leaves other empty
    pub fn append(&mut self, other: &mut Self) {
        self.queue.append(&mut other.queue);
    }

    pub fn new() -> Self {
        Self { queue: vec![] }
    }
}

pub(crate) fn run_command(event_loop: &ActiveEventLoop, game: &mut Engine, command: Command) {
    match command {
        Command::CloseWindow(window_id) => {
            game.windows.retain(|window| {
                let mut shared = window.shared_info.as_ref().unwrap().lock().unwrap();
                shared.should_despawn = true;
                shared.game_info.window_id != window_id
            });
            if game.windows.is_empty() {
                println!("No windows open: Exiting");
                event_loop.exit();
            }
        }
        Command::Exit => event_loop.exit(),
        Command::NewWindow(input_handler, window_atributes) => {
            game.windows
                .push(GameWindow::new(input_handler, window_atributes));
            let len = game.windows.len();
            game.windows
                .get_mut(len - 1)
                .unwrap()
                .start(&mut game.commands, event_loop);
        }
    }
}
