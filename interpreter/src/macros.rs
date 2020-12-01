/// Converts the given value into the inner value of the variant
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
macro_rules! number {
    ($enum: expr) => {
        cast!($enum, Dynamic::Number)
    };
}

/// Converts the given value into an array
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
