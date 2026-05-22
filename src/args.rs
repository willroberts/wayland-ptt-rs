use std::env;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    // Set to 'true' to log configuration and matching libev events.
    pub verbose: bool,
    // The key to listen for in the event stream.
    pub listen_key: String,
    // The key to send to X11 when a matching event is detected.
    pub send_key: String,
    // The mouse button to send to X11 in place of send_key.
    pub mouse_button: Option<u32>,
    // The input device to watch with libev.
    pub input_device_path: String,
}

pub fn usage(program: &str) -> String {
    format!("Usage: {program} [-v] [-l listen_key] [-s send_key] /dev/input/by-id/<device-name>")
}

pub fn parse_args_from<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_else(|| "wayland-ptt".to_string());

    let mut verbose = false;
    let mut listen_key = "BTN_EXTRA".to_string();
    let mut send_key = "MOUSE9".to_string();
    let mut mouse_button = Some(9);
    let mut input_device_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-v" => verbose = true,
            "-l" => {
                listen_key = args.next().ok_or_else(|| usage(&program))?;
            }
            "-s" => {
                let value = args.next().ok_or_else(|| usage(&program))?;
                mouse_button = value
                    .strip_prefix("MOUSE")
                    .map(|suffix| suffix.parse::<u32>())
                    .transpose()
                    .map_err(|_| format!("Invalid mouse_button value: {value}"))?;
                if mouse_button == Some(0) {
                    return Err("Invalid mouse_button value: MOUSE0".to_string());
                }
                send_key = value;
            }
            _ if arg.starts_with('-') => return Err(usage(&program)),
            _ => {
                if input_device_path.is_some() {
                    return Err(usage(&program));
                }
                input_device_path = Some(arg);
            }
        }
    }

    let input_device_path = input_device_path.ok_or_else(|| usage(&program))?;

    Ok(Config {
        verbose,
        listen_key,
        send_key,
        mouse_button,
        input_device_path,
    })
}

pub fn parse_args() -> Result<Config, String> {
    parse_args_from(env::args())
}
