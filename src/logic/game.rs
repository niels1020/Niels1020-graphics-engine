use winit::{
    application::ApplicationHandler, event_loop::ActiveEventLoop, window::WindowAttributes,
};

use crate::logic::{
    commands::{Commands, run_command},
    game_window::{GameWindow, InputHandler},
};

pub struct Game {
    pub(crate) windows: Vec<GameWindow>,
    pub(crate) commands: Commands,
}

impl Game {
    pub fn new(
        main_input_handler: Box<dyn InputHandler + Send>,
        window_attributes: WindowAttributes,
    ) -> Self {
        Self {
            windows: vec![GameWindow::new(main_input_handler, window_attributes)],
            commands: Commands::new(),
        }
    }

    fn run_commands(&mut self, event_loop: &ActiveEventLoop) {
        while !self.commands.queue.is_empty() {
            let command = self.commands.queue.remove(0);
            run_command(event_loop, self, command);
        }
    }
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let len = self.windows.len();
        self.windows
            .get_mut(len - 1)
            .unwrap()
            .start(&mut self.commands, event_loop);
        self.run_commands(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        for window in self.windows.iter_mut() {
            window.window_event(&mut self.commands, window_id, event.clone());
        }
        self.run_commands(event_loop);
    }
}
