use flexi_logger::{opt_format, Age, Cleanup, Criterion, Duplicate, Logger, Naming};
use std::path::PathBuf;

#[cfg(debug_assertions)]
static DEBUG: &'static str = "debug,wwm::app_bar=debug";

#[cfg(not(debug_assertions))]
static DEBUG: &'static str = "debug";

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let mut path: PathBuf = ["./log"].iter().collect();

    #[cfg(not(debug_assertions))]
    {
        path = dirs::config_dir().expect("Failed to get config directory");

        path.push("wwm");
        path.push("log");
    }

    Logger::with_env_or_str(DEBUG)
        .log_to_file()
        .duplicate_to_stderr(Duplicate::All)
        .directory(path)
        .format(opt_format)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(6),
        )
        .start()
        .expect("Failed to initialize logger");

    Ok(())
}
