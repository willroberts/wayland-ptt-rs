use std::process;

use wayland_ptt::args::parse_args;

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };

    println!("{config:?}");
}
