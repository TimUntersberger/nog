use notify::DebouncedEvent;
use notify::watcher;
use notify::Watcher;
use log::debug;
use notify::RecursiveMode;
use std::sync::mpsc::channel;

pub fn start() {
    std::thread::spawn(|| {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, std::time::Duration::from_millis(10)).expect("Failed to spawn file watcher");

        let mut path = dirs::config_dir().expect("Failed to get config dir");

        path.push("wwm");
        path.push("config.yaml");

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .expect("Failed to watch config directory");

        loop {
            match rx.recv() {
                Ok(ev) => match ev {
                    DebouncedEvent::Write(_) => {
                        debug!("detected config change");
                    },
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}
