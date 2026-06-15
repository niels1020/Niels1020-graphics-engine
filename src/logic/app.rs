use winit::{application::ApplicationHandler, event_loop::ActiveEventLoop};

use crate::{
    common::{Command, Commands},
    logic::game_window::{GameWindow, InputHandler},
};

pub struct Game {
    windows: Vec<GameWindow>,
}

impl Game {
    pub fn new(main_input_handler: Box<dyn InputHandler>) -> Self {
        Self {
            windows: vec![GameWindow::new(main_input_handler)],
        }
    }

    fn run_commands(&mut self, event_loop: &ActiveEventLoop, commands: Commands) {
        for command in commands {
            match command {
                Command::CloseWindow(window_id) => {
                    self.windows.retain(|window| {
                        if let Some(id) = window.game_info.window_id {
                            id != window_id
                        } else {
                            true
                        }
                    });
                    if self.windows.is_empty() {
                        println!("No windows open: Exiting");
                        event_loop.exit();
                    }
                }
                Command::Exit => event_loop.exit(),
                Command::NewWindow(input_handler) => {
                    self.windows.push(GameWindow::new(input_handler));
                    let len = self.windows.len();
                    let commands = self.windows.get_mut(len - 1).unwrap().start(event_loop);
                    self.run_commands(event_loop, commands);
                }
            }
        }
    }
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let len = self.windows.len();
        let commands = self.windows.get_mut(len - 1).unwrap().start(event_loop);
        self.run_commands(event_loop, commands);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut commands = vec![];
        for window in self.windows.iter_mut() {
            commands.extend(window.window_event(window_id, event.clone()));
        }
        self.run_commands(event_loop, commands);
    }
}
