use log::{error, info};
use std::{fs, path::PathBuf};

const TEMPLATE: &'static str = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n"; // TODO: check for OS compatability?
const PINNED_OFFSET: usize = 10;
const PINNED_VISIBLE: &'static str = "v";
const PINNED_INVISIBLE: &'static str = "n";

pub struct Store {}
pub struct StoredData {
    pub grids: Vec<String>,
    pub pinned_windows: Vec<(bool, Vec<i32>)>, // (isVisible, Vec of windowIDs)
}

impl Store {
    fn get_path() -> PathBuf {
        #[allow(unused_mut)]
        let mut path: PathBuf = ["./log"].iter().collect();
        #[cfg(not(debug_assertions))]
        {
            path = dirs::config_dir().expect("Failed to get config directory");

            path.push("nog");
        }

        path.push("workspaces.grid");
        path
    }
    pub fn save(id: i32, grid: String) {
        info!("Saving {} {}", id, grid);
        Store::write_to_file((id - 1) as usize, grid);
    }

    ///   v|1234|5678
    ///   ^  ^    ^
    ///   |  |    |
    ///   |  |    -- 2nd pinned window
    ///   |  -- 1st pinned window
    ///   -- indicates whether pinned windows are visible (v) or not (n)
    pub fn save_pinned(index: usize, pinned: Vec<&i32>, is_visible: bool) {
        let visible = if is_visible {
            PINNED_VISIBLE
        } else {
            PINNED_INVISIBLE
        };
        let pinned = pinned
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("|");
        let pinned = format!("{}|{}", visible, pinned);
        info!("Saving Pinned IDs {}", pinned);
        Store::write_to_file(index + PINNED_OFFSET, pinned);
    }
    pub fn load() -> StoredData {
        let data = match fs::read_to_string(Store::get_path()) {
            Ok(f) => f,
            Err(_) => TEMPLATE.into(),
        }
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

        let (grids, other_data) = data.split_at(10);

        let mut pinned_windows: Vec<(bool, Vec<i32>)> = Vec::new();
        // at this point, other_data should contain 11 rows left if it's a valid format
        // these last 11 rows each represent a set of pinned windows.
        // 0 is global, 1-10 are for each workspace
        if other_data.len() > 11 {
            let (saved_pinned, _) = other_data.split_at(11);

            saved_pinned.iter().for_each(|x| {
                let split_pinned = x.split("|").collect::<Vec<_>>();
                let (visibility, pinned_ids) = split_pinned.split_at(1);
                let visibility = visibility.get(0).unwrap_or(&PINNED_INVISIBLE);
                let windows = pinned_ids
                    .into_iter()
                    .map(|x| x.parse::<i32>())
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                    .collect();
                pinned_windows.push((*visibility == PINNED_VISIBLE, windows));
            });
        }

        StoredData {
            grids: Vec::from(grids),
            pinned_windows,
        }
    }
    fn write_to_file(index: usize, new_value: String) {
        let file = match fs::read_to_string(Store::get_path()) {
            Ok(f) => f,
            Err(_) => TEMPLATE.into(),
        };
        let file = file.split("\n").collect::<Vec<_>>();
        let file: String = TEMPLATE // start with template so we always have the right # of rows
            .split("\n")
            .into_iter()
            .enumerate()
            .map(|(i, empty_string)| {
                if i == index {
                    &new_value
                } else {
                    // if we don't already have a value on this row
                    // then populate it with an empty
                    // string from the template
                    if let Some(old_value) = file.get(i) {
                        old_value
                    } else {
                        empty_string
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        match fs::write(Store::get_path(), file) {
            Err(e) => error!("Error storing grid {:?}", e),
            _ => (),
        }
    }
}
