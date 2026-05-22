use std::process;

use wayland_ptt::args::parse_args;
use wayland_ptt::setup_evdev::setup_evdev;

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

    println!("{config:?}");
    println!(
        "Input device name: {:?}, input device ID: {:?}, listen key code: {:?}",
        setup.device.name(),
        setup.device.input_id(),
        setup.listen_key_code
    );
}
