use std::process;

use wayland_ptt::args::parse_args;
use wayland_ptt::setup_evdev::{input_device_metadata, setup_evdev};

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };

    let setup = match setup_evdev(&config) {
        Ok(setup) => setup,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    for line in input_device_metadata(&setup.device) {
        eprintln!("{line}");
    }

    println!("{config:?}");
    println!("listen key code: {:?}", setup.listen_key_code);
}
