use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    window::{Window, WindowId},
};

use crate::logic::{
    commands::Commands,
    game_window::{GameInfo, InputHandler, SceneTree},
};

///put data here for loggic thread to proccess
pub(crate) struct LogicInfo {
    pub window_events: Vec<(WindowEvent, WindowId)>,
    pub device_events: Vec<(DeviceEvent, DeviceId)>,
    pub should_despawn: bool,
}
pub(crate) type SharedLogicInfo = Arc<Mutex<LogicInfo>>;

///some data gets copied to the render thread later
pub(crate) struct LocalInfo {
    pub game_info: GameInfo,
    pub commands: Commands,
}

///data gets copied here from localinfo at the end of a logic iteration
pub(crate) struct RenderInfo {
    pub scene_tree: SceneTree,
    pub commands: Commands,
    pub refresh_rate: usize,
    pub window_id: WindowId,
}
pub(crate) type SharedRenderInfo = Arc<Mutex<RenderInfo>>;

pub(crate) fn start_logic_thread(
    window: Arc<Window>,
    input_handler: Box<dyn InputHandler + Send>,
) -> (SharedLogicInfo, SharedRenderInfo) {
    let shared_render_info = Arc::new(Mutex::new(RenderInfo {
        scene_tree: SceneTree::new(),
        commands: Commands::new(),
        refresh_rate: 0,
        window_id: window.clone().id(),
    }));
    let shared_render_info_thread = shared_render_info.clone();

    let shared_logic_info = Arc::new(Mutex::new(LogicInfo {
        window_events: vec![],
        device_events: vec![],
        should_despawn: false,
    }));
    let shared_logic_info_thread = shared_logic_info.clone();

    thread::spawn(move || {
        let mut input_handler = input_handler;

        let mut local_info = {
            LocalInfo {
                game_info: GameInfo::new(window),
                commands: Commands::new(),
            }
        };

        input_handler.start(&mut local_info.commands, &mut local_info.game_info);

        let mut last_update = Instant::now();
        let mut last_redraw = Instant::now();
        'game: loop {
            //render update
            {
                let now = Instant::now();
                let delta = (now - last_update).as_secs_f64();

                input_handler.update(&mut local_info.commands, &mut local_info.game_info, delta);
                last_update = now;

                let should_redraw = (now - last_redraw)
                    >= Duration::from_secs_f64(1.0 / local_info.game_info.refresh_rate as f64);
                if should_redraw {
                    last_redraw = now;
                    local_info.game_info.window.request_redraw();
                }
            }

            //event handling
            {
                let (window_events, device_events) = {
                    let mut shared = shared_logic_info_thread.lock().unwrap();
                    (
                        shared
                            .window_events
                            .drain(..)
                            .collect::<Vec<(WindowEvent, WindowId)>>(),
                        shared
                            .device_events
                            .drain(..)
                            .collect::<Vec<(DeviceEvent, DeviceId)>>(),
                    )
                };

                //window event handling
                for (event, id) in window_events {
                    if id == local_info.game_info.window.id() {
                        input_handler.window_event(
                            &mut local_info.commands,
                            &mut local_info.game_info,
                            event,
                        );
                    } else {
                        input_handler.other_window_event(
                            &mut local_info.commands,
                            &mut local_info.game_info,
                            id,
                            event,
                        );
                    }
                }

                //device event handling
                for (event, id) in device_events {

                    input_handler.device_event(
                        &mut local_info.commands,
                        &mut local_info.game_info,
                        event,
                        id,
                    );
                }
            }

            //check if it should despawn
            {
                if shared_logic_info_thread.lock().unwrap().should_despawn {
                    input_handler.exit(&mut local_info.commands, &mut local_info.game_info);

                    let mut shared = shared_render_info_thread.lock().unwrap();
                    shared.commands.append(&mut local_info.commands);

                    break 'game;
                }
            }

            //clone scene tree and commands to render thread
            {
                let mut shared = shared_render_info_thread.lock().unwrap();
                shared.scene_tree = local_info.game_info.tree.clone();
                shared.commands.append(&mut local_info.commands);
                shared.refresh_rate = local_info.game_info.refresh_rate;
            }
        }
    });

    (shared_logic_info, shared_render_info)
}
