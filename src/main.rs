use std::process;

use wayland_ptt::args::parse_args;
use wayland_ptt::evdev::{configure_evdev, input_device_metadata, read_next_listen_key_state, ListenKeyState};
use wayland_ptt::x11::{configure_x11, configure_x11_target, send_target_state};

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };

    let mut evdev_config = match configure_evdev(&config) {
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

    let x11_target = match configure_x11_target(&x11_config, &config) {
        Ok(target) => target,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    for line in input_device_metadata(&evdev_config.device) {
        if config.verbose {
            eprintln!("{line}");
        }
    }

    if config.verbose {
        eprintln!(
            "Listening for code {:?}, sending {}",
            evdev_config.listen_key_code, config.send_key
        );
    }

    loop {
        let state = match read_next_listen_key_state(&mut evdev_config, config.verbose) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("Failed to read evdev input: {err}");
                process::exit(1);
            }
        };

        if config.verbose && state == ListenKeyState::Pressed {
            eprintln!("Target key pressed, sending {}", config.send_key);
        }

        if let Err(err) = send_target_state(&x11_config, x11_target, state) {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}
