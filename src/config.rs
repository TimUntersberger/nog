use std::io::{Error, ErrorKind};

pub struct Config {
    pub remove_title_bar: bool
}

impl Config {
    pub fn new() -> Self {
        Self {
            remove_title_bar: true
        }
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>>{
    let mut pathbuf = match dirs::config_dir() {
        Some(path) => path,
        None => std::path::PathBuf::new()
    };

    pathbuf.push("wwm");

    if !pathbuf.exists() {
        std::fs::create_dir(pathbuf.clone());
    }

    pathbuf.push("config.yaml");

    if !pathbuf.exists() {
        std::fs::File::create(pathbuf.clone());
    }

    let path = match pathbuf.to_str() {
        Some(string) => string,
        None => ""
    };

    println!("Config path: {}", path);

    let file_content = std::fs::read_to_string(path).unwrap();
    let yaml = &yaml_rust::YamlLoader::load_from_str(&file_content).unwrap()[0];
    let mut config = Config::new();

    if let yaml_rust::yaml::Yaml::Hash(hash) = yaml {
        for entry in hash.iter() {
            let (key, value) = entry;
            let config_key = key.as_str().ok_or("Invalid config key")?;

            match config_key {
                "remove_title_bar" => {
                    config.remove_title_bar = value.as_bool().ok_or("remove_title_bar has to a bool")?;
                },
                s => {
                    return Err(Box::new(Error::new(ErrorKind::InvalidInput, "unknown option ".to_string() + s)));
                }
            }
        }
    }
    Ok(config)
}