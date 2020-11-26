use std::{fs,path::PathBuf};
use log::{info};

// TODO: need to hide tiles after loading them so they aren't visible if user is on an empty workspace (like workspace 1 when nog loads)

const TEMPLATE: &'static str = "\n\n\n\n\n\n\n\n\n\n"; // TODO: check for OS compatability? 
pub struct Store { }

impl Store {
    fn get_path() -> PathBuf {
        #[allow(unused_mut)]
        let mut path: PathBuf = dirs::config_dir().expect("Failed to get config directory");
        path.push("nog");
        path.push("grid.nog");

        path
    }
    pub fn save(id: i32, grid: String) -> std::io::Result<()> {
        info!("Saving {} {}", id, grid);
        let file = match fs::read_to_string(Store::get_path()) {
                       Ok(f) => f,
                       Err(_) => TEMPLATE.into()
                   };
        let file: String = file.split("\n")
                               .into_iter()
                               .enumerate()
                               .map(|(i, value)| if i + 1 == (id as usize) { &grid } else { value })
                               .collect::<Vec::<_>>()
                               .join("\n");

        fs::write(Store::get_path(), file)?;

        Ok(())
    }
    pub fn load() -> Vec<String> {
        match fs::read_to_string(Store::get_path()) {
            Ok(f) => f,
            Err(_) => TEMPLATE.into()
        }.split("\n")
         .map(|x| x.to_string())
         .collect()
    }
}
