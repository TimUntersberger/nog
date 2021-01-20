use interpreter::Interpreter;
use std::env;
use std::ffi::OsStr;
use std::iter;
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args();
    args.next();
    if let Some(file_path) = args.next() {
        let abs_path = env::current_dir()
            .map(|cwd| {
                cwd.iter()
                    .chain(iter::once(OsStr::new(&file_path)))
                    .collect::<PathBuf>()
            })
            .unwrap();
        let mut i = Interpreter::new();
        i.debug = true;
        if let Err(msg) = i.execute_file(abs_path) {
            println!("ERROR: {}", msg);
        }
    }
}
