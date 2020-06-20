macro_rules! if_hex {
    ($config:ident, $target:ident, $value:ident, $key:ident) => {
        if ($target == stringify!($key)) {
            $config.$key = i32::from_str_radix(
                $value
                    .as_str()
                    .ok_or(format!("{} has to be a string", stringify!($key)))?,
                16,
            )?;
        }
    };
}
macro_rules! if_bool {
    ($config:ident, $target:ident, $value:ident, $key:ident) => {
        if ($target == stringify!($key)) {
            $config.$key = $value
                .as_bool()
                .ok_or(format!("{} has to be a bool", stringify!($key)))?;
        }
    };
}

macro_rules! ensure_str {
    ($name:tt, $hash:ident, $key:ident) => {
        $hash[stringify!($key)]
            .as_str()
            .ok_or(format!("a {} has to have a '{}' property of type string", $name, stringify!($key)))?;
    };
}

macro_rules! ensure_i32 {
    ($name:tt, $hash:ident, $key:ident) => {
        $hash[stringify!($key)]
            .as_i64()
            .ok_or(format!("a {} has to have a '{}' property of type int", $name, stringify!($key)))? as i32;
    };
}