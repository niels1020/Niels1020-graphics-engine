use std::{
    collections::VecDeque, sync::{Arc, Mutex}, thread::{self, sleep}, time::{Duration, Instant},
};

use egui::{Context, FullOutput};
use egui_winit::State;
use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    window::{Window, WindowId},
};

use crate::logic::{
    commands::GlobalComands,
    game_window::{GameInfo, InputHandler},
};

///put data here for loggic thread to proccess
pub(crate) struct LogicInfo {
    pub window_events: VecDeque<(WindowEvent, WindowId)>,
    pub device_events: VecDeque<(DeviceEvent, DeviceId)>,
    pub should_despawn: bool,
}
pub(crate) type SharedLogicInfo = Arc<Mutex<LogicInfo>>;

///some data gets copied to the render thread later
pub(crate) struct LocalInfo {
    pub game_info: GameInfo,
    pub egui_ctx: Context,
    pub egui_state: State,
}

///data gets copied here from localinfo at the end of a logic iteration
pub(crate) struct RenderInfo {
    pub egui_output: Option<FullOutput>,
}
pub(crate) type SharedRenderInfo = Arc<Mutex<RenderInfo>>;

pub(crate) fn start_logic_thread(
    commands: GlobalComands,
    window: Arc<Window>,
    input_handler: Box<dyn InputHandler + Send>,
    gui_ctx: Context,
) -> (SharedLogicInfo, SharedRenderInfo) {
    let shared_render_info = Arc::new(Mutex::new(RenderInfo {
        egui_output: None,
    }));
    let shared_render_info_thread = shared_render_info.clone();

    let shared_logic_info = Arc::new(Mutex::new(LogicInfo {
        window_events: VecDeque::new(),
        device_events: VecDeque::new(),
        should_despawn: false,
    }));
    let shared_logic_info_thread = shared_logic_info.clone();

    thread::spawn(move || {
        let mut input_handler = input_handler;

        let mut local_info = {
            LocalInfo {
                egui_state: State::new(
                    gui_ctx.clone(),
                    egui::ViewportId::ROOT,
                    &window.clone(),
                    Some(window.scale_factor() as f32),
                    None,
                    None,
                ),
                game_info: GameInfo::new(window),
                egui_ctx: gui_ctx,
            }
        };

        input_handler.start(commands.clone(), &mut local_info.game_info);

        let mut last_sleep = 20;

        let mut last_update = Instant::now();
        'game: loop {
            //render and update
            {
                let now = Instant::now();

                let delta = (now - last_update).as_secs_f64();

                input_handler.update(commands.clone(), &mut local_info.game_info, delta);

                last_update = now;
            }

            //event handling
            {
                let (window_events, device_events, requests) = {
                    let mut shared = shared_logic_info_thread.lock().unwrap();
                    (
                        shared
                            .window_events
                            .drain(..)
                            .collect::<VecDeque<(WindowEvent, WindowId)>>(),
                        shared
                            .device_events
                            .drain(..)
                            .collect::<VecDeque<(DeviceEvent, DeviceId)>>(),
                        commands
                            .get_requests(local_info.game_info.window_id)
                            .unwrap_or_default(),
                    )
                };

                //window event handling
                for (event, id) in window_events {

                    match event.clone() {
                        WindowEvent::Resized(new_size) => commands.resize(local_info.game_info.window_id, new_size.into()),
                        _ => {}
                    }                    

                    let mut consumed = false;
                    if id == local_info.game_info.window.id() {
                        //handle egui event
                        consumed = local_info
                            .egui_state
                            .on_window_event(&local_info.game_info.window, &event.clone())
                            .consumed;

                        input_handler.window_event(
                            commands.clone(),
                            &mut local_info.game_info,
                            event,
                            consumed,
                        );
                    } else {
                        input_handler.other_window_event(
                            commands.clone(),
                            &mut local_info.game_info,
                            id,
                            event,
                            consumed,
                        );
                    }
                }

                //device event handling
                for (event, id) in device_events {
                    input_handler.device_event(
                        commands.clone(),
                        &mut local_info.game_info,
                        event,
                        id,
                    );
                }

                //delivering requests
                for request in requests {
                    input_handler.receive_request(
                        commands.clone(),
                        &mut local_info.game_info,
                        request,
                    );
                }
            }

            //do gui
            {
                let raw_input = local_info
                    .egui_state
                    .take_egui_input(&local_info.game_info.window);

                local_info.egui_ctx.begin_pass(raw_input);

                input_handler.gui(
                    commands.clone(),
                    &mut local_info.game_info,
                    local_info.egui_ctx.clone(),
                );

                let mut output = local_info.egui_ctx.end_pass();

                local_info.egui_state.handle_platform_output(
                    &local_info.game_info.window,
                    output.platform_output.clone(),
                );

                let mut shared = shared_render_info_thread.lock().unwrap();

                if shared.egui_output.is_some() {
                    let moved = shared.egui_output.take().unwrap();
                    output.textures_delta.append(moved.textures_delta);
                }

                shared.egui_output = Some(output);
            }

            //check if it should despawn
            {
                if shared_logic_info_thread.lock().unwrap().should_despawn {
                    input_handler.exit(commands.clone(), &mut local_info.game_info);

                    break 'game;
                }
            }


            //dont overwork other threads
            let mut slept = false;
            while commands.render_len(local_info.game_info.window_id) > local_info.game_info.max_queue_size {
                sleep(Duration::from_millis(last_sleep));
                last_sleep = last_sleep * 2;
                slept = true;
            }
            if !slept {
                last_sleep = last_sleep / 2;
            }
        }
    });

    (shared_logic_info, shared_render_info)
}
