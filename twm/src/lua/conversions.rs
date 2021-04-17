use mlua::{FromLua, Value, Error as LuaError};

use crate::{direction::Direction, bar::component::ComponentText, system::SystemError};
use crate::split_direction::SplitDirection;
use std::str::FromStr;

impl From<SystemError> for LuaError {
    fn from(e: SystemError) -> Self {
        LuaError::RuntimeError(e.to_string())
    }
}

impl FromLua<'_> for Direction {
    fn from_lua(lua_value: mlua::Value<'_>, lua: &'_ mlua::Lua) -> mlua::Result<Self> {
        let raw_direction = String::from_lua(lua_value, lua)?;

        Ok(Direction::from_str(&raw_direction).unwrap_or(Direction::Right))
    }
}

impl FromLua<'_> for SplitDirection {
    fn from_lua(lua_value: mlua::Value<'_>, lua: &'_ mlua::Lua) -> mlua::Result<Self> {
        let raw_direction = String::from_lua(lua_value, lua)?;

        Ok(SplitDirection::from_str(&raw_direction).unwrap_or(SplitDirection::Horizontal))
    }
}

impl FromLua<'_> for ComponentText {
    fn from_lua(lua_value: mlua::Value<'_>, lua: &'_ mlua::Lua) -> mlua::Result<Self> {
        let text = match lua_value {
            Value::Nil => ComponentText::new().with_display_text("nil".into()),
            Value::Boolean(x) => ComponentText::new().with_display_text(x.to_string()),
            Value::Integer(x) => ComponentText::new().with_display_text(x.to_string()),
            Value::Number(x) => ComponentText::new().with_display_text(x.to_string()),
            Value::String(x) => ComponentText::new().with_display_text(x.to_str().unwrap().to_string()),
            Value::Table(tbl) => {
                let mut comp = ComponentText::new();
                for res in tbl.pairs::<String, Value>() {
                    if let Ok((key, val)) = res {
                        match key.as_ref() {
                            "text" => {
                                comp = comp.with_display_text(String::from_lua(val, lua)?);
                            },
                            "value" => {
                                comp = comp.with_value(i32::from_lua(val, lua)?);
                            },
                            "fg" => {
                                comp = comp.with_foreground_color(i32::from_lua(val, lua)?);
                            },
                            "bg" => {
                                comp = comp.with_background_color(i32::from_lua(val, lua)?);
                            },
                            _ => {}
                        }
                    }
                }
                comp
            },
            _ => ComponentText::new()
        };

        Ok(text)
    }
}
