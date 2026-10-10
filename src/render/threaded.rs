use std::thread;

use crate::{
    logic::{
        commands::{GlobalComands, Request},
        game_window::SceneTree,
        threaded::SharedRenderInfo,
    },
    render::renderer::Renderer,
};

pub(crate) fn start_render_thread(
    commands: GlobalComands,
    renderer: Renderer,
    shared: SharedRenderInfo,
) {
    thread::spawn(move || {
        let mut renderer = renderer;
        let mut scene_tree = SceneTree::new();

        let mut running = true;

        while running {
            match commands.get_render_commands(renderer.window_id) {
                Some(mut queue) => {
                    while !queue.is_empty() {
                        let cmd = queue.remove(0).unwrap();

                        match cmd {
                            crate::logic::commands::RenderCommand::AddRenderLayer(render_layer, id) => {
                                let name = render_layer.get_name();
                                scene_tree.root.push(render_layer);
                                commands.push_request(id, Request::LayerAddedAtIndex(name, scene_tree.root.len() - 1));
                            }
                            crate::logic::commands::RenderCommand::RemoveRenderLayer(index) => {
                                scene_tree.root.remove(index);
                            }
                            crate::logic::commands::RenderCommand::GetRenderLayerClone(
                                index,
                                window_id,
                            ) => {
                                if scene_tree.root.len() > index {
                                    commands.push_request(
                                        window_id,
                                        Request::RenderLayerClone(
                                            scene_tree.root.get(index).unwrap().clone(),
                                        ),
                                    )
                                }
                            }
                            crate::logic::commands::RenderCommand::ModifyRenderLayer(
                                index,
                                fn_once,
                            ) => {
                                if let Some(layer) = scene_tree.root.get_mut(index) {
                                    fn_once(layer)
                                }
                            }
                            crate::logic::commands::RenderCommand::Exit => running = false,
                            crate::logic::commands::RenderCommand::Resized(new_size) => {
                                renderer.resize(new_size.0, new_size.1)
                            }
                        }
                    }
                }
                None => {}
            }

            renderer.ui.output = shared.lock().unwrap().egui_output.take();
            renderer.render(&mut scene_tree);
        }
    });
}
