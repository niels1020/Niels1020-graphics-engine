use winit::{
    application::ApplicationHandler, event_loop::ActiveEventLoop, window::WindowAttributes,
};

use crate::logic::{
    commands::{GlobalComands, run_logic_command}, game_window::{GameWindow, InputHandler},
};

pub struct Engine {
    pub(crate) windows: Vec<GameWindow>,
    pub(crate) commands: GlobalComands,
}

impl Engine {
    pub fn new(
        main_input_handler: Box<dyn InputHandler + Send>,
        window_attributes: WindowAttributes,
    ) -> Self {
        Self {
            windows: vec![GameWindow::new(main_input_handler, window_attributes)],
            commands: GlobalComands::new(),
        }
    }

    
    pub fn run_commands(&mut self, event_loop: &ActiveEventLoop) {
        let mut commands = self.commands.get_logic_commands();
        while !commands.is_empty() {
            let cmd = commands.remove(0).unwrap();
            run_logic_command(event_loop, self, cmd);
        }
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let len = self.windows.len();
        self.windows
            .get_mut(len - 1)
            .unwrap()
            .start(self.commands.clone(), event_loop);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        for window in self.windows.iter_mut() {
            window.window_event(window_id, event.clone());
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        for window in self.windows.iter_mut() {
            window.device_event(event.clone(), device_id);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.run_commands(event_loop);
    }
}
