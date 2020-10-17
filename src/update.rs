#![allow(unreachable_code)]
#![allow(dead_code)]

use crate::AppState;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

const STOP: AtomicBool = AtomicBool::new(false);

pub fn start(state: &AppState) -> Result<(), ()> {
    if let Some(_update_channel) = state.config.get_update_channel() {
        let update_interval = state.config.update_interval;

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
