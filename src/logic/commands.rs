use winit::{
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::{
    logic::{
        commands::Request::RenderLayerClone, engine::Engine, game_window::{GameWindow, InputHandler},
    }, render::render_layers::RenderLayer,
};

pub(crate) enum Command {
    CloseWindow(WindowId),
    ///exits the engine WITHOUT calling exit() on any input handler
    Exit,
    NewWindow(Box<dyn InputHandler>, WindowAttributes),

    AddRenderLayer(WindowId, Box<dyn RenderLayer>),
    RemoveRenderLayer(WindowId, usize),
    GetRenderLayerClone(WindowId, usize, WindowId), //second window id is target to deliver the clone
    ModifyRenderLayer(
        WindowId,
        usize,
        Box<dyn FnOnce(&mut Box<dyn RenderLayer>) + Send>,
    ),
}

pub enum Request {
    RenderLayerClone(Box<dyn RenderLayer>),
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
        input_handler: Box<dyn InputHandler>,
        window_attributes: WindowAttributes,
    ) {
        self.queue
            .push(Command::NewWindow(input_handler, window_attributes));
    }

    pub fn add_render_layer(&mut self, window_id: WindowId, layer: Box<dyn RenderLayer>) {
        self.queue.push(Command::AddRenderLayer(window_id, layer));
    }

    pub fn remove_render_layer(&mut self, window_id: WindowId, index: usize) {
        self.queue
            .push(Command::RemoveRenderLayer(window_id, index));
    }

    pub fn modify_render_layer<A>(&mut self, window_id: WindowId, index: usize, op: A)
    where
        A: FnOnce(&mut Box<dyn RenderLayer>) + Send + 'static,
    {
        self.queue
            .push(Command::ModifyRenderLayer(window_id, index, Box::new(op)));
    }

    pub fn get_render_layer_clone(&mut self, to_clone: WindowId, index_to_clone: usize, to_deliver: WindowId) {
        self.queue.push(Command::GetRenderLayerClone(to_clone, index_to_clone, to_deliver));
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
                let shared_render = window.shared_render_info.as_ref().unwrap().lock().unwrap();
                let mut shared_logic = window.shared_logic_info.as_ref().unwrap().lock().unwrap();
                shared_logic.should_despawn = true;
                shared_render.window_id != window_id
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
        Command::AddRenderLayer(window_id, render_layer) => {
            if let Some(game_window) = game.windows.iter_mut().find(|a| a.window_id == window_id) {
                game_window.scene_tree.root.push(render_layer);
            };
        }
        Command::RemoveRenderLayer(window_id, index) => {
            if let Some(game_window) = game.windows.iter_mut().find(|a| a.window_id == window_id) {
                game_window.scene_tree.root.remove(index);
            };
        }
        Command::ModifyRenderLayer(window_id, index, fn_once) => {
            if let Some(game_window) = game.windows.iter_mut().find(|a| a.window_id == window_id) {
                if let Some(layer) = game_window.scene_tree.root.get_mut(index) {
                    fn_once(layer);
                }
            };
        }
        Command::GetRenderLayerClone(read, index, target) => {
            if let Some(game_window) = game.windows.iter().find(|a| a.window_id == read) {
                if let Some(pointer) = game_window.scene_tree.root.get(index) {
                    let clone = pointer.clone();
                    if let Some(target_layer) = game.windows.iter_mut().find(|a| a.window_id == target) {
                       target_layer.shared_logic_info.as_ref().unwrap().lock().unwrap().requests.push(RenderLayerClone(clone)); 
                    }
                }
            };
        }
    }
}
