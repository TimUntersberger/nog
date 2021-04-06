use mlua::FromLua;

use crate::direction::Direction;
use crate::split_direction::SplitDirection;
use std::str::FromStr;

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
