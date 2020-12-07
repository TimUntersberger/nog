/// Converts the given value into the inner value of the variant
#[macro_export]
macro_rules! cast {
    ($enum: expr, $variant: path) => {
        if let $variant(x) = $enum {
            x
        } else {
            unreachable!()
        }
    };
}

/// Converts the given value into a number
#[macro_export]
macro_rules! number {
    ($enum: expr) => {
        cast!($enum, Dynamic::Number)
    };
}

/// Converts the given value into an object
#[macro_export]
macro_rules! object {
    ($enum: expr) => {
        cast!($enum, Dynamic::Object)
    };
}

/// Converts the given value into a boolean
#[macro_export]
macro_rules! boolean {
    ($enum: expr) => {
        cast!($enum, Dynamic::Boolean)
    };
}

/// Converts the given value into a string
#[macro_export]
macro_rules! string {
    ($enum: expr) => {
        cast!($enum, Dynamic::String)
    };
}

/// Converts the given value into an array
#[macro_export]
macro_rules! array {
    ($enum: expr) => {
        cast!($enum, Dynamic::Array)
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
