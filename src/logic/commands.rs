use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use winit::{
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::{
    logic::{
        commands::LogicCommand::CloseWindow,
        engine::Engine,
        game_window::{GameWindow, InputHandler},
    },
    render::render_layers::RenderLayer,
};

pub(crate) enum LogicCommand {
    CloseWindow(WindowId),
    ///exits the engine WITHOUT calling exit() on any input handler
    Exit,
    NewWindow(Box<dyn InputHandler>, WindowAttributes),
}
pub(crate) enum RenderCommand {
    AddRenderLayer(Box<dyn RenderLayer>, WindowId),
    RemoveRenderLayer(usize),
    GetRenderLayerClone(usize, WindowId), //second window id is target to deliver the clone
    ModifyRenderLayer(usize, Box<dyn FnOnce(&mut Box<dyn RenderLayer>) + Send>),
    Exit,
    Resized((u32, u32)),
}

pub enum Request {
    RenderLayerClone(Box<dyn RenderLayer>),
    LayerAddedAtIndex(String, usize)
}

struct Commands {
    logic_queue: VecDeque<LogicCommand>,
    render_queues: HashMap<WindowId, VecDeque<RenderCommand>>,
    requests: HashMap<WindowId, VecDeque<Request>>,
}

pub struct GlobalComands(Arc<Mutex<Commands>>);

impl GlobalComands {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Commands::new())))
    }
}

impl Clone for GlobalComands {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Commands {
    fn new() -> Self {
        Self {
            logic_queue: VecDeque::new(),
            render_queues: HashMap::new(),
            requests: HashMap::new(),
        }
    }

    fn render_len(&self, window_id: WindowId) -> usize {
        match self.render_queues.get(&window_id) {
            Some(queue) => queue.len(),
            None => 0,
        }
    }

    fn close_window(&mut self, id: WindowId) {
        match self.render_queues.get_mut(&id) {
            Some(queue) => queue.push_back(RenderCommand::Exit),
            None => {
                let mut vecd = VecDeque::new();
                vecd.push_back(RenderCommand::Exit);
                self.render_queues.insert(id, vecd);
            }
        }
        self.logic_queue.push_back(CloseWindow(id));
    }

    fn exit(&mut self) {
        self.logic_queue.push_back(LogicCommand::Exit);
    }

    fn new_window(
        &mut self,
        input_handler: Box<dyn InputHandler>,
        window_attributes: WindowAttributes,
    ) {
        self.logic_queue
            .push_back(LogicCommand::NewWindow(input_handler, window_attributes));
    }

    fn add_render_layer(&mut self, window_id: WindowId, layer: Box<dyn RenderLayer>) {
        match self.render_queues.get_mut(&window_id) {
            Some(queue) => queue.push_back(RenderCommand::AddRenderLayer(layer, window_id)),
            None => {
                let mut vecd = VecDeque::new();
                vecd.push_back(RenderCommand::AddRenderLayer(layer, window_id));
                self.render_queues.insert(window_id, vecd);
            }
        }
    }

    fn remove_render_layer(&mut self, window_id: WindowId, index: usize) {
        match self.render_queues.get_mut(&window_id) {
            Some(queue) => queue.push_back(RenderCommand::RemoveRenderLayer(index)),
            None => {
                let mut vecd = VecDeque::new();
                vecd.push_back(RenderCommand::RemoveRenderLayer(index));
                self.render_queues.insert(window_id, vecd);
            }
        }
    }

    fn modify_render_layer<A>(&mut self, window_id: WindowId, index: usize, op: A)
    where
        A: FnOnce(&mut Box<dyn RenderLayer>) + Send + 'static,
    {
        match self.render_queues.get_mut(&window_id) {
            Some(queue) => queue.push_back(RenderCommand::ModifyRenderLayer(index, Box::new(op))),
            None => {
                let mut vecd = VecDeque::new();
                vecd.push_back(RenderCommand::ModifyRenderLayer(index, Box::new(op)));
                self.render_queues.insert(window_id, vecd);
            }
        }
    }

    fn get_render_layer_clone(
        &mut self,
        to_clone: WindowId,
        index_to_clone: usize,
        to_deliver: WindowId,
    ) {
        match self.render_queues.get_mut(&to_clone) {
            Some(queue) => queue.push_back(RenderCommand::GetRenderLayerClone(
                index_to_clone,
                to_deliver,
            )),
            None => {
                let mut vecd = VecDeque::new();
                vecd.push_back(RenderCommand::GetRenderLayerClone(
                    index_to_clone,
                    to_deliver,
                ));
                self.render_queues.insert(to_clone, vecd);
            }
        }
    }

    fn resize(&mut self, window_id: WindowId, new_size: (u32, u32)) {
        match self.render_queues.get_mut(&window_id) {
            Some(queue) => queue.push_back(RenderCommand::Resized(new_size)),
            None => {
                let mut q = VecDeque::new();
                q.push_back(RenderCommand::Resized(new_size));
                self.render_queues.insert(window_id, q);
            }
        }
    }

    fn get_requests(&mut self, window_id: WindowId) -> Option<VecDeque<Request>> {
        match self.requests.get_mut(&window_id) {
            Some(queue) => Some(queue.drain(..).collect()),
            None => None,
        }
    }

    fn get_render_commands(&mut self, window_id: WindowId) -> Option<VecDeque<RenderCommand>> {
        match self.render_queues.get_mut(&window_id) {
            Some(queue) => Some(queue.drain(..).collect()),
            None => None,
        }
    }

    fn get_logic_commands(&mut self) -> VecDeque<LogicCommand> {
        self.logic_queue.drain(..).collect()
    }

    fn push_request(&mut self, window_id: WindowId, request: Request) {
        match self.requests.get_mut(&window_id) {
            Some(queue) => queue.push_back(request),
            None => {
                let mut queue = VecDeque::new();
                queue.push_back(request);
                self.requests.insert(window_id, queue);
            }
        }
    }
}

macro_rules! forward_methods {
    (
        $(
            $method:ident (
                $($arg:ident : $arg_ty:ty),* $(,)?
            ) -> $ret:ty;
        )*
    ) => {
        impl GlobalComands {
            $(
                pub fn $method(&self, $($arg: $arg_ty),*) -> $ret {
                    self.0.lock().unwrap().$method($($arg),*)
                }
            )*
        }
    };
}

macro_rules! forward_methods_crate {
    (
        $(
            $method:ident (
                $($arg:ident : $arg_ty:ty),* $(,)?
            ) -> $ret:ty;
        )*
    ) => {
        impl GlobalComands {
            $(
                pub(crate) fn $method(&self, $($arg: $arg_ty),*) -> $ret {
                    self.0.lock().unwrap().$method($($arg),*)
                }
            )*
        }
    };
}

forward_methods! {
    render_len(id: WindowId) -> usize;
    close_window(id: WindowId) -> ();
    exit() -> ();
    new_window(input_handler: Box<dyn InputHandler>, window_atributes: WindowAttributes) -> ();
    add_render_layer(target: WindowId, layer: Box<dyn RenderLayer>) -> ();
    remove_render_layer(target: WindowId, index: usize) -> ();
    get_render_layer_clone(to_clone: WindowId, index_to_clone: usize, to_deliver: WindowId)-> ();
    resize(window_id: WindowId, new_size: (u32, u32)) -> ();
}

forward_methods_crate! {
    get_requests(window_id: WindowId) -> Option<VecDeque<Request>>;
    get_render_commands(window_id: WindowId) -> Option<VecDeque<RenderCommand>>;
    get_logic_commands() -> VecDeque<LogicCommand>;
    push_request(window_id: WindowId, request: Request) -> ();
}

//i don't have macro for this
impl GlobalComands {
    pub fn modify_render_layer<A>(&self, target: WindowId, index: usize, op: A)
    where
        A: FnOnce(&mut Box<dyn RenderLayer>) + Send + 'static,
    {
        self.0
            .lock()
            .unwrap()
            .modify_render_layer(target, index, op);
    }
}

pub(crate) fn run_logic_command(
    event_loop: &ActiveEventLoop,
    game: &mut Engine,
    command: LogicCommand,
) {
    match command {
        LogicCommand::CloseWindow(window_id) => {
            game.windows.retain(|window| {
                let mut shared_logic = window.shared_logic_info.as_ref().unwrap().lock().unwrap();
                shared_logic.should_despawn = true;
                window.window_id != window_id
            });
            if game.windows.is_empty() {
                println!("No windows open: Exiting");
                event_loop.exit();
            }
        }
        LogicCommand::Exit => event_loop.exit(),
        LogicCommand::NewWindow(input_handler, window_atributes) => {
            game.windows
                .push(GameWindow::new(input_handler, window_atributes));
            let len = game.windows.len();
            game.windows
                .get_mut(len - 1)
                .unwrap()
                .start(game.commands.clone(), event_loop);
        }
    }
}
