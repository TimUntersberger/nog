use std::{fmt::Debug, path::PathBuf, sync::Arc};

use mlua::{Function, Lua, Table};
use parking_lot::Mutex;

const CALLBACK_TBL_NAME: &'static str = "__callbacks";

//TODO: Fix unwraps

#[derive(Clone)]
pub struct LuaRuntime(pub Arc<Mutex<Lua>>);

pub fn get_err_msg(e: &mlua::Error) -> String {
    match e {
        mlua::Error::CallbackError { traceback, cause } => {
            format!("{} \n{}", get_err_msg(&cause), traceback)
        }
        mlua::Error::RuntimeError(x) => format!("{}", x),
        e => format!("{}", e),
    }
}
impl LuaRuntime {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Lua::new())))
    }

    pub fn with_lua<R>(&self, f: impl Fn(&mut Lua) -> mlua::Result<R>) -> mlua::Result<R> {
        f(&mut self.0.lock())
    }

    pub fn add_callback(lua: &Lua, cb: Function) -> mlua::Result<usize> {
        let cb_tbl = lua
            .globals()
            .get::<_, Table>("nog")?
            .get::<_, Table>("__callbacks")?;

        let id = cb_tbl.raw_len() + 1;
        cb_tbl.set(id, cb)?;

        Ok(id as usize)
    }

    pub fn get_callback(lua: &Lua, id: usize) -> mlua::Result<Function>
    {
        lua
            .globals()
            .get::<_, Table>("nog")?
            .get::<_, Table>(CALLBACK_TBL_NAME)?
            .get::<_, _>(id)
    }

    pub fn print_callbacks(&self) -> mlua::Result<()> {
        let rt = self.0.lock();
        let cbs_tbl = rt
            .globals()
            .get::<_, Table>("nog")?
            .get::<_, Table>(CALLBACK_TBL_NAME)?;

        println!("callbacks");
        for res in cbs_tbl.pairs::<i32, Function>() {
            if let Ok((key, value)) = res {
                println!("{} = {:#?}", key, value);
            }
        }

        Ok(())
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
