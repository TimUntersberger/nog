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
    let mut builder = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {:5} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(LOG_FILE.as_str()).unwrap())
        .chain(std::io::stdout());

    #[cfg(debug_assertions)]
    {
        builder = builder
            .level_for("hyper", log::LevelFilter::Info)
            .level_for("wwm::app_bar", log::LevelFilter::Error);
    }

    builder.apply().unwrap();

    Ok(())
}
