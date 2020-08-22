use crate::bar;
use rhai::{ImmutableString, Module};
use systemstat::{Platform, System, BatteryLife, Duration};
use log::error;

pub fn new() -> Module {
    let mut module = Module::new();

    module.set_fn_0("battery", || {
        let sys = System::new();
        Ok(match sys.battery_life() {
            Ok(x) => x,
            Err(x) => {
                error!("{}", x.to_string());
                BatteryLife {
                    remaining_time: Duration::new(0, 0),
                    remaining_capacity: 0.0
                }
            }
        })
    });

    module
}

