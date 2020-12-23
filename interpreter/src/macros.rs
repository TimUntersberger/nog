/// Converts the given value into the inner value of the variant
#[macro_export]
macro_rules! cast {
    ($enum: expr, $variant: path, $expected: tt) => {
        if let $variant(x) = $enum {
            Ok(x)
        } else {
            Err(RuntimeError::UnexpectedType {
                expected: $expected.into(),
                actual: $enum.type_name(),
            })
        }
    };
}

/// Converts the given value into a number
#[macro_export]
macro_rules! number {
    ($enum: expr) => {
        cast!($enum, Dynamic::Number, "Number")
    };
}

/// Converts the given value into an object
#[macro_export]
macro_rules! object {
    ($enum: expr) => {
        cast!($enum, Dynamic::Object, "Object")
    };
}

/// Converts the given value into a rust value
#[macro_export]
macro_rules! rust_value {
    ($enum: expr) => {
        cast!($enum, Dynamic::RustValue, "RustValue")
    };
}

/// Converts the given value into a boolean
#[macro_export]
macro_rules! boolean {
    ($enum: expr) => {
        cast!($enum, Dynamic::Boolean, "Boolean")
    };
}

/// Converts the given value into a string
#[macro_export]
macro_rules! string {
    ($enum: expr) => {
        cast!($enum, Dynamic::String, "String")
    };
}

/// Converts the given value into an array
#[macro_export]
macro_rules! array {
    ($enum: expr) => {
        cast!($enum, Dynamic::Array, "Array")
    };
}

macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
             )*
                _map
        }
    };
}
