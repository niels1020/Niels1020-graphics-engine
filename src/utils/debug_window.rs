use std::collections::HashMap;
use egui::Window;
use winit::{
    event::WindowEvent::CloseRequested,
    window::{WindowAttributes, WindowId},
};

use crate::logic::{commands::Request, game_window::InputHandler};

pub fn get_new_debug_window(to_debug: WindowId) -> (Box<dyn InputHandler>, WindowAttributes) {
    return (
        Box::new(DebugWindow {
            to_debug,
            debug_info: HashMap::new(),
            requested: false,
        }),
        WindowAttributes::default(),
    );
}

struct DebugWindow {
    to_debug: WindowId,
    debug_info: HashMap<usize, (String, Vec<String>)>,
    requested: bool,
}

impl InputHandler for DebugWindow {
    fn window_event(
        &mut self,
        commands: crate::logic::commands::GlobalComands,
        game_info: &mut crate::logic::game_window::GameInfo,
        event: winit::event::WindowEvent,
        _consumed: bool,
    ) {
        match event {
            CloseRequested => {
                commands.close_window(game_info.window_id);
            }
            _ => {}
        }
    }

    fn update(
        &mut self,
        commands: crate::logic::commands::GlobalComands,
        game_info: &mut crate::logic::game_window::GameInfo,
        _delta: f64,
    ) {
        if !self.requested {
            self.requested = true;
            commands.get_layer_count(self.to_debug, game_info.window_id);
        }
    }

    fn start(
        &mut self,
        _commands: crate::logic::commands::GlobalComands,
        _game_info: &mut crate::logic::game_window::GameInfo,
    ) {
    }

    fn exit(
        &mut self,
        _commands: crate::logic::commands::GlobalComands,
        _game_info: &mut crate::logic::game_window::GameInfo,
    ) {
    }

    fn receive_request(
        &mut self,
        commands: crate::logic::commands::GlobalComands,
        game_info: &mut crate::logic::game_window::GameInfo,
        request: crate::logic::commands::Request,
    ) {
        match request {
            Request::LayerCount(len, target) => {
                if target == self.to_debug {
                    for i in 0..len {
                        commands.get_render_layer_clone(self.to_debug, i, game_info.window_id);
                    }
                }
            }
            Request::RenderLayerClone(layer, index, from) => {
                if from == self.to_debug {
                    self.debug_info.insert(index, layer.get_debug_info());
                }
            }
            _ => {}
        }
    }

    fn gui(
        &mut self,
        _commands: crate::logic::commands::GlobalComands,
        _game_info: &mut crate::logic::game_window::GameInfo,
        ctx: egui::Context,
    ) {
        Window::new("layer debug").show(&ctx, |ui| {
            ui.label("Hello, world!");
            ui.label("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
            ui.label("The quick brown fox jumps over the lazy dog.");

            for (_, (info, children)) in self.debug_info.iter() {
                ui.collapsing(info.as_str(), |ui| {
                    for info in children.iter() {
                        ui.label(info.as_str());
                    }
                });
            }
            /*for (_, (info, children_info)) in self.debug_info.iter_mut() {
                CollapsingHeader::new(info.as_str()).show(ui, |ui| {
                    for info in children_info {
                        ui.label("wtf");
                        ui.label(info.as_str());
                    }
                });
            }*/
        });
    }
}
