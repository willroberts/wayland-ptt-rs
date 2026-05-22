use std::fmt;
use std::str::FromStr;

use evdev::{Device, InputId, KeyCode};

use crate::args::Config;

pub struct EvdevSetup {
    pub device: Device,
    pub listen_key_code: KeyCode,
}

#[derive(Debug)]
pub enum SetupEvdevError {
    OpenDevice {
        path: String,
        source: std::io::Error,
    },
    InvalidListenKey {
        key: String,
    },
    UnsupportedListenKey {
        path: String,
        key: String,
    },
}

impl fmt::Display for SetupEvdevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenDevice { path, source } => {
                write!(f, "Failed to open device {path}: {source}")
            }
            Self::InvalidListenKey { key } => {
                write!(f, "Key code not found: {key}")
            }
            Self::UnsupportedListenKey { path, key } => {
                write!(
                    f,
                    "Input device {path} is not capable of sending listen key {key}"
                )
            }
        }
    }
}

impl std::error::Error for SetupEvdevError {}

pub fn setup_evdev(config: &Config) -> Result<EvdevSetup, SetupEvdevError> {
    let device = open_device(&config.input_device_path)?;
    let listen_key_code = resolve_listen_key(&config.listen_key)?;

    ensure_device_supports_key(&device, listen_key_code, &config.input_device_path, &config.listen_key)?;

    Ok(EvdevSetup {
        device,
        listen_key_code,
    })
}

pub fn input_device_metadata(device: &Device) -> [String; 2] {
    format_input_device_metadata(device.name(), device.input_id())
}

// This is extracted into its own function for testability.
pub fn format_input_device_metadata(name: Option<&str>, input_id: InputId) -> [String; 2] {
    [
        format!("Input device name: \"{}\"", name.unwrap_or("unknown")),
        format!(
            "Input device ID: bus {:#x} vendor {:#x} product {:#x}",
            input_id.bus_type().0,
            input_id.vendor(),
            input_id.product()
        ),
    ]
}

fn open_device(path: &str) -> Result<Device, SetupEvdevError> {
    Device::open(path).map_err(|source| SetupEvdevError::OpenDevice {
        path: path.to_string(),
        source,
    })
}

fn resolve_listen_key(key: &str) -> Result<KeyCode, SetupEvdevError> {
    KeyCode::from_str(key).map_err(|_| SetupEvdevError::InvalidListenKey {
        key: key.to_string(),
    })
}

fn ensure_device_supports_key(
    device: &Device,
    listen_key_code: KeyCode,
    path: &str,
    key: &str,
) -> Result<(), SetupEvdevError> {
    let supports_key = device
        .supported_keys()
        .is_some_and(|keys| keys.contains(listen_key_code));

    if supports_key {
        Ok(())
    } else {
        Err(SetupEvdevError::UnsupportedListenKey {
            path: path.to_string(),
            key: key.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    // These tests run against our internal resolve_listen_key helper.
    // Otherwise they'd be located with the public API tests in tests/.

    use super::{resolve_listen_key, SetupEvdevError};
    use evdev::KeyCode;

    #[test]
    fn resolves_valid_listen_key_name() {
        let key = resolve_listen_key("KEY_LEFTMETA").unwrap();

        assert_eq!(key, KeyCode::KEY_LEFTMETA);
    }

    #[test]
    fn rejects_invalid_listen_key_name() {
        let err = resolve_listen_key("INVALID_KEY").unwrap_err();

        match err {
            SetupEvdevError::InvalidListenKey { key } => assert_eq!(key, "INVALID_KEY"),
            other => panic!("expected InvalidListenKey, got {other:?}"),
        }
    }
}
