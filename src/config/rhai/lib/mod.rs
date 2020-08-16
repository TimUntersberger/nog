use rhai::{AST, Engine};

mod popup;

pub fn init(engine: &mut Engine){
    popup::init(engine);
}