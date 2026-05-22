use std::process;

use wayland_ptt::args::parse_args;
use wayland_ptt::evdev::{configure_evdev, input_device_metadata};
use wayland_ptt::x11::configure_x11;

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };

    let evdev_config = match configure_evdev(&config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    let x11_config = match configure_x11() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    for line in input_device_metadata(&evdev_config.device) {
        eprintln!("{line}");
    }

    println!("{config:?}");
    println!("listen key code: {:?}", evdev_config.listen_key_code);
    println!(
        "XTEST version: {}.{}",
        x11_config.xtest_major_version, x11_config.xtest_minor_version
    );
}
