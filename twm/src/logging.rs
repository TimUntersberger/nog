use flexi_logger::{opt_format, Age, Cleanup, Criterion, Duplicate, Logger, Naming};
use std::path::PathBuf;

#[cfg(debug_assertions)]
const DEBUG: &'static str = "twm=debug";

#[cfg(not(debug_assertions))]
const DEBUG: &'static str = "twm=debug";

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut path: PathBuf = ["./log"].iter().collect();

    #[cfg(not(debug_assertions))]
    {
        path = dirs::config_dir().expect("Failed to get config directory");

        path.push("nog");
        path.push("log");
    }

    #[allow(unused_mut)]
    let mut logger = Logger::with_env_or_str(DEBUG)
        .duplicate_to_stderr(Duplicate::All)
        .directory(path)
        .format(opt_format)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(6),
        );

    #[cfg(not(debug_assertions))]
    {
        logger = logger.log_to_file();
    }
        
    logger.start()
        .expect("Failed to initialize logger");

    Ok(())
}
