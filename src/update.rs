use crate::CONFIG;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

const STOP: AtomicBool = AtomicBool::new(false);

pub fn start() -> Result<(), ()> {
    let update_channel = CONFIG.lock().unwrap().get_update_channel().cloned();
    if let Some(_update_channel) = update_channel {
        let update_interval = CONFIG.lock().unwrap().update_interval.clone();

        thread::spawn(move || {
            while !STOP.load(Ordering::SeqCst) {
                todo!();
                thread::sleep(update_interval);
            }
        });

        STOP.store(false, Ordering::SeqCst);
    }

    Ok(())
}

pub fn stop() {
    STOP.store(true, Ordering::SeqCst);
}
