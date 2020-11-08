use std::{collections::HashMap, sync::Arc};

use parking_lot::{Mutex, RwLock};
use rhai::{EvalAltResult, Module, ModuleResolver, Scope, AST};

use crate::AppState;

pub struct PluginModuleResolver {
    state_arc: Arc<Mutex<AppState>>,
    cache: RwLock<HashMap<String, AST>>,
}

impl PluginModuleResolver {
    pub fn new(state_arc: Arc<Mutex<AppState>>) -> Self {
        Self {
            state_arc,
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl ModuleResolver for PluginModuleResolver {
    fn resolve(
        &self,
        engine: &rhai::Engine,
        path: &str,
        pos: rhai::Position,
    ) -> Result<rhai::Module, Box<rhai::EvalAltResult>> {
        for plugin in &self.state_arc.lock().plugin_manager.plugins {
            let mut file_path = plugin.path.clone();
            file_path.push("plugin");
            file_path.push(path);
            file_path.set_extension("nog");

            if file_path.exists() {
                if let Some(ast) = self.cache.read().get(file_path.to_str().unwrap()) {
                    return Ok(
                        Module::eval_ast_as_new(Scope::new(), ast, engine).map_err(
                            |err| {
                                Box::new(EvalAltResult::ErrorInModule(
                                    file_path.to_str().unwrap().to_string(),
                                    err,
                                    pos,
                                ))
                            },
                        )?,
                    );
                } else {
                    let ast = engine.compile_file(file_path.clone()).map_err(|err| {
                        Box::new(EvalAltResult::ErrorInModule(
                            file_path.to_str().unwrap().to_string(),
                            err,
                            pos,
                        ))
                    })?;
                    let mut module = Module::eval_ast_as_new(Scope::new(), &ast, engine)
                        .map_err(|err| {
                            Box::new(EvalAltResult::ErrorInModule(
                                file_path.to_str().unwrap().to_string(),
                                err,
                                pos,
                            ))
                        })?;

                    // TODO: This line causes a deadlock, but I don't know why?
                    // self.cache.write().insert(file_path.to_str().unwrap().to_string(), ast);

                    return Ok(module);
                }
            }
        }

        Err(Box::new(EvalAltResult::ErrorModuleNotFound(
            path.to_string(),
            pos,
        )))
    }
}
