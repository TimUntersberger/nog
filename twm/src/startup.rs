use crate::system::api;
use log::info;

#[allow(unreachable_code, unused_variables)]
pub fn set_launch_on_startup(enabled: bool) {
    #[cfg(debug_assertions)] // don't override the startup exe when in debug mode
    return;

    if enabled {
        api::add_launch_on_startup();
        info!("Enabled launch on startup in registry");
    } else {
        api::remove_launch_on_startup();
        info!("Disabled launch on startup in registry");
    }
}
