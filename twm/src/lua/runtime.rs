use std::{fmt::Debug, path::PathBuf, sync::Arc};

use mlua::Lua;
use parking_lot::Mutex;

//TODO: Fix unwraps

#[derive(Clone)]
pub struct LuaRuntime(pub Arc<Mutex<Lua>>);

fn get_err_msg(e: &mlua::Error) -> String {
    match e {
        mlua::Error::CallbackError { traceback, cause } => {
            format!("{} \n{}", get_err_msg(&cause), traceback)
        }
        mlua::Error::RuntimeError(x) => {
            format!("{}", x)
        }
        e => {
            format!("{}", e)
        }
    }
}
impl LuaRuntime {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Lua::new())))
    }

    pub fn with_lua(&self, f: impl Fn(&mut Lua) -> mlua::Result<()>) -> mlua::Result<()> {
        f(&mut self.0.lock())
    }

    pub fn run_str(&self, name: &str, s: &str) {
        let guard = self.0.lock();
        let chunk = guard.load(s).set_name(name).unwrap();
        if let Err(e) = chunk.exec() {
            println!("[ERROR]: {}", get_err_msg(&e));
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
