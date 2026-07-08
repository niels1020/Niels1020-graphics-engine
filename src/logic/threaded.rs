use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use winit::{event::WindowEvent, window::WindowId};

use crate::logic::{
    commands::Commands,
    game_window::{GameInfo, InputHandler},
};

pub(crate) struct SharedInfo {
    pub game_info: GameInfo,
    pub commands: Commands,
    pub events: Vec<(WindowEvent, WindowId)>,
    pub should_despawn: bool,
}

pub(crate) fn start_logic_thread(
    window_id: WindowId,
    input_handler: Box<dyn InputHandler + Send>,
) -> Arc<Mutex<SharedInfo>> {
    let shared_info = Arc::new(Mutex::new(SharedInfo {
        game_info: GameInfo::new(window_id),
        commands: Commands::new(),
        events: vec![],
        should_despawn: false,
    }));
    let shared_info_thread = shared_info.clone();

    thread::spawn(move || {
        let mut input_handler = input_handler;
        {
            let mut shared = shared_info_thread.lock().unwrap();
            let mut commands = Commands::new();
            input_handler.start(&mut commands, &mut shared.game_info);
            shared.commands.append(&mut commands);
        }

        let mut last_update = Instant::now();
        loop {
            {
                let mut shared = shared_info_thread.lock().unwrap();
                let mut commands = Commands::new();

                let now = Instant::now();

                let delta = (now - last_update).as_secs_f64();

                input_handler.update(&mut commands, &mut shared.game_info, delta);

                last_update = now;

                shared.commands.append(&mut commands);
            }

            //event handling
            {
                let mut shared = shared_info_thread.lock().unwrap();
                while !shared.events.is_empty() {
                    let mut commands = Commands::new();
                    let (event, id) = shared.events.remove(0);

                    if id == shared.game_info.window_id {
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

            {
                let mut shared = shared_info_thread.lock().unwrap();
                if shared.should_despawn {
                    let mut commands = Commands::new();
                    input_handler.exit(&mut shared.game_info);
                    shared.commands.append(&mut commands);
                }
            }
        }
    });

    shared_info
}
