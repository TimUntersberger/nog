macro_rules! if_str {
    ($config:ident, $target:ident, $value:ident, $key:ident) => {
        if ($target == stringify!($key)) {
            $config.$key = $value
                .as_str()
                .ok_or(format!("{} has to be a string", stringify!($key)))?
                .to_string();
        }
    };
}
macro_rules! convert_color_format {
    ($ident:expr) => {
        $ident = RGB(
            GetBValue($ident as u32),
            GetGValue($ident as u32),
            GetRValue($ident as u32),
        ) as i32;
    };
}
macro_rules! if_regex {
    ($config:ident, $target:ident, $value:ident, $key:ident) => {
        if ($target == stringify!($key)) {
            $config.$key = Regex::new(
                $value
                    .as_str()
                    .ok_or(format!("{} has to be a string", stringify!($key)))?,
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

// macro_rules! if_specific_rule {
//     ($config:ident, $target:ident, $value:ident, $key:ident) => {
//         if $target == stringify!($key) {
//             let mut specific_rule = SpecificRule::default();
//
//             let hash = $value.as_hash().ok_or(format!("{} has to be a hash", stringify!($key)))?;
//
//             for entry in hash.iter() {
//                 let (key, value) = entry;
//                 let hash_key = key.as_str().ok_or("Invalid config key")?;
//
//                 if_i32!(specific_rule, hash_key, value, x);
//                 if_i32!(specific_rule, hash_key, value, y);
//                 if_i32!(specific_rule, hash_key, value, width);
//                 if_i32!(specific_rule, hash_key, value, height);
//             }
//
//             $config.$key = specific_rule;
//         }
//     };
// }

macro_rules! if_i32 {
    ($config:ident, $target:ident, $value:ident, $key:ident) => {
        if ($target == stringify!($key)) {
            $config.$key = $value
                .as_i64()
                .ok_or(format!("{} has to be an integer", stringify!($key)))?
                as i32;
        }
    };
}

macro_rules! ensure_str {
    ($name:tt, $hash:ident, $key:ident) => {
        $hash[stringify!($key)].as_str().ok_or(format!(
            "a {} has to have a '{}' property of type string",
            $name,
            stringify!($key)
        ))?;
    };
}

macro_rules! ensure_i32 {
    ($name:tt, $hash:ident, $key:ident) => {
        $hash[stringify!($key)].as_i64().ok_or(format!(
            "a {} has to have a '{}' property of type int",
            $name,
            stringify!($key)
        ))? as i32;
    };
}
