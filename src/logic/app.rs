use winit::application::ApplicationHandler;

use crate::logic::{Command, game_window::{GameWindow, InputHandler}};

pub struct Game<T: InputHandler> {
    windows: Vec<GameWindow<T>>,
}

impl<T: InputHandler> Game<T> {
    pub fn new(main_input_handler: T) -> Self {
        Self {
            windows: vec![GameWindow::new(main_input_handler)],
        }
    }
}

impl<T: InputHandler> ApplicationHandler for Game<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let len = self.windows.len();
        self.windows
            .get_mut(len - 1)
            .unwrap()
            .render_info_init(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut commands = vec![];
        for window in self.windows.iter_mut() {
            commands.extend(window.window_event(window_id, event.clone()));
        }

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
            }
        }
    }
}
