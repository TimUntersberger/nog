use crate::{event::Event, AppState};
use log::{debug, error};
use notify::watcher;
use notify::DebouncedEvent;
use notify::RecursiveMode;
use notify::Watcher;
use parking_lot::Mutex;
use std::{
    sync::{mpsc::channel, Arc},
    thread,
};

pub fn start(state: Arc<Mutex<AppState>>) {
    let state = state.clone();
    thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, std::time::Duration::from_millis(10))
            .expect("Failed to spawn file watcher");

        let mut path = dirs::config_dir().expect("Failed to get config dir");

        path.push("nog");

        debug!("Watching {:?} recursively for file changes", &path);

        watcher
            .watch(path, RecursiveMode::Recursive)
            .expect("Failed to watch config directory");

        loop {
            match rx.recv() {
                Ok(ev) => match ev {
                    DebouncedEvent::Write(path) => {
                        if path.extension().unwrap() == "ns" {
                            debug!("Nogscript file {:?} changed! Reloading config", &path);
                            state
                                .lock()
                                .event_channel
                                .sender
                                .clone()
                                .send(Event::ReloadConfig)
                                .expect("Failed to send ReloadConfig event");
                        }
                    }
                    _ => {}
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });
}
