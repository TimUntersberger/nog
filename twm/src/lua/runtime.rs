use std::{fmt::Debug, path::PathBuf, sync::Arc};

use mlua::Lua;
use parking_lot::Mutex;

//TODO: Fix unwraps

#[derive(Clone)]
pub struct LuaRuntime(pub Arc<Mutex<Lua>>);

impl LuaRuntime {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Lua::new())))
    }

    pub fn with_lua(&self, f: impl Fn(&mut Lua) -> mlua::Result<()>) -> mlua::Result<()> {
        f(&mut self.0.lock())
    }

    pub fn run_str(&self, name: &str, s: &str) {
        let guard = self.0.lock();
        if let Err(e) = guard.load(s).set_name(name).unwrap().exec() {
            println!("[ERROR]: {}", e);
        }
    }

    pub fn run_file<P: Into<PathBuf>>(&self, p: P) {
        let content = std::fs::read_to_string(p.into()).unwrap();
        self.run_str("init.lua", &content);
    }
}

impl Debug for LuaRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
