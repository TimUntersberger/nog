use log::{debug, info};
use winapi::shared::minwindef::HKEY;
use winapi::um::winnt::KEY_SET_VALUE;
use winapi::um::winnt::REG_OPTION_NON_VOLATILE;
use winapi::um::winnt::REG_SZ;
use winapi::um::winreg::RegCreateKeyExW;
use winapi::um::winreg::RegDeleteKeyValueW;
use winapi::um::winreg::RegSetValueExW;
use winapi::um::winreg::HKEY_CURRENT_USER;
use crate::util;

#[allow(unreachable_code, unused_variables)]
pub fn set_launch_on_startup(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)] // don't override the startup exe when in debug mode
    return Ok(());
    if let Some(mut target_path) = dirs::config_dir() {
        target_path.push("nog");
        target_path.push("nog.exe");

        if let Ok(source_path) = std::env::current_exe() {
            if source_path != target_path && enabled {
                debug!("Exe doesn't exist yet");
                std::fs::copy(source_path, &target_path)?;
            }

            let app_path: Vec<u16> = target_path
                .to_str()
                .unwrap_or_default()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut key_name: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut value_name = util::to_widestring("nog");

            unsafe {
                let mut key: HKEY = std::mem::zeroed();

                if enabled {
                    if RegCreateKeyExW(
                        HKEY_CURRENT_USER,
                        key_name.as_mut_ptr(),
                        0,
                        std::ptr::null_mut(),
                        REG_OPTION_NON_VOLATILE,
                        KEY_SET_VALUE,
                        std::ptr::null_mut(),
                        &mut key,
                        std::ptr::null_mut(),
                    ) == 0
                    {
                        RegSetValueExW(
                            key,
                            value_name.as_mut_ptr(),
                            0,
                            REG_SZ,
                            app_path.as_ptr() as _,
                            app_path.len() as u32 * 2,
                        );

                        info!("Enabled launch on startup in registry");
                    };
                } else {
                    RegDeleteKeyValueW(
                        HKEY_CURRENT_USER,
                        key_name.as_mut_ptr(),
                        value_name.as_mut_ptr(),
                    );
                    info!("Disabled launch on startup in registry");
                }
            }
        }
    }

    Ok(())
}
