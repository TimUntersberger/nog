use flexi_logger::{Cleanup, Criterion, Duplicate, Logger, Naming, opt_format, Age};
use lazy_static::lazy_static;

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref LOG_FILE: String = String::from("output.log");
}

#[cfg(not(debug_assertions))]
lazy_static! {
    pub static ref LOG_FILE: String = {
        let mut path = dirs::config_dir().unwrap();
        path.push("wwm");
        path.push("output.log");
        path.into_os_string().into_string().unwrap()
    };
}

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    Logger::with_env_or_str("debug,wwm::app_bar=error")
        .log_to_file()
        .duplicate_to_stderr(Duplicate::All)
        .directory("./log")
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
