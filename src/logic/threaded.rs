use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent}, window::{Window, WindowId},
};

use crate::logic::{
    commands::Commands,
    game_window::{GameInfo, InputHandler},
};

pub(crate) struct SharedInfo {
    pub game_info: GameInfo,
    pub commands: Commands,
    pub window_events: Vec<(WindowEvent, WindowId)>,
    pub device_events: Vec<(DeviceEvent, DeviceId)>,
    pub should_despawn: bool,
    pub refresh_rate: u64,
}

pub(crate) fn start_logic_thread(
    window: Arc<Window>,
    input_handler: Box<dyn InputHandler + Send>,
) -> Arc<Mutex<SharedInfo>> {
    let shared_info = Arc::new(Mutex::new(SharedInfo {
        game_info: GameInfo::new(window),
        commands: Commands::new(),
        window_events: vec![],
        device_events: vec![],
        should_despawn: false,
        refresh_rate: 144,
    }));
    let shared_info_thread = shared_info.clone();

    thread::spawn(move || {
        let mut input_handler = input_handler;
        {
            let mut shared = shared_info_thread.lock().unwrap();
            let mut commands = Commands::new();
            input_handler.start(&mut commands, &mut shared.game_info);
            shared.commands.append(&mut commands);

            shared.game_info.window.request_redraw();
        }

        let mut last_update = Instant::now();
        loop {
            //frame update
            {
                let mut shared = shared_info_thread.lock().unwrap();
                let mut commands = Commands::new();

                let now = Instant::now();

                let delta = (now - last_update).as_secs_f64();

                input_handler.update(&mut commands, &mut shared.game_info, delta);

                last_update = now;

                shared.commands.append(&mut commands);
            }

            //window event handling
            {
                let mut shared = shared_info_thread.lock().unwrap();
                while !shared.window_events.is_empty() {
                    let mut commands = Commands::new();
                    let (event, id) = shared.window_events.remove(0);

                    if id == shared.game_info.window.id() {
                        input_handler.window_event(&mut commands, &mut shared.game_info, event);
                    } else {
                        input_handler.other_window_event(
                            &mut commands,
                            &mut shared.game_info,
                            id,
                            event,
                        );
                    }

                    shared.commands.append(&mut commands);
                }
            }

            //device event handling
            {
                let mut shared = shared_info_thread.lock().unwrap();
                while !shared.device_events.is_empty() {
                    let mut commands = Commands::new();
                    let (event, id) = shared.device_events.remove(0);

                    input_handler.device_event(&mut commands, &mut shared.game_info, event, id);

                    shared.commands.append(&mut commands);
                }
            }

            //check if it should despawn
            {
                let mut shared = shared_info_thread.lock().unwrap();
                if shared.should_despawn {
                    let mut commands = Commands::new();
                    input_handler.exit(&mut shared.game_info);
                    shared.commands.append(&mut commands);
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
    });

    shared_info
}
