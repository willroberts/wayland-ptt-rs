use std::fmt;
use std::io;
use std::str::FromStr;

use evdev::{Device, EventSummary, InputEvent, InputId, KeyCode};

use crate::args::Config;

pub struct EvdevConfig {
    pub device: Device,
    pub listen_key_code: KeyCode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ListenKeyState {
    Pressed,
    Released,
}

#[derive(Debug)]
pub enum ConfigureEvdevError {
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

impl fmt::Display for ConfigureEvdevError {
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

impl std::error::Error for ConfigureEvdevError {}

pub fn configure_evdev(config: &Config) -> Result<EvdevConfig, ConfigureEvdevError> {
    let device = open_device(&config.input_device_path)?;
    let listen_key_code = resolve_listen_key(&config.listen_key)?;

    ensure_device_supports_key(
        &device,
        listen_key_code,
        &config.input_device_path,
        &config.listen_key,
    )?;

    Ok(EvdevConfig {
        device,
        listen_key_code,
    })
}

pub fn input_device_metadata(device: &Device) -> [String; 2] {
    format_input_device_metadata(device.name(), device.input_id())
}

pub fn read_next_listen_key_state(evdev_config: &mut EvdevConfig) -> io::Result<ListenKeyState> {
    loop {
        for event in evdev_config.device.fetch_events()? {
            if let Some(state) = classify_listen_key_event(event, evdev_config.listen_key_code) {
                return Ok(state);
            }
        }
    }
}

pub fn classify_listen_key_event(
    event: InputEvent,
    listen_key_code: KeyCode,
) -> Option<ListenKeyState> {
    match event.destructure() {
        EventSummary::Key(_, code, 1) if code == listen_key_code => Some(ListenKeyState::Pressed),
        EventSummary::Key(_, code, 0) if code == listen_key_code => Some(ListenKeyState::Released),
        _ => None,
    }
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

fn open_device(path: &str) -> Result<Device, ConfigureEvdevError> {
    Device::open(path).map_err(|source| ConfigureEvdevError::OpenDevice {
        path: path.to_string(),
        source,
    })
}

fn resolve_listen_key(key: &str) -> Result<KeyCode, ConfigureEvdevError> {
    KeyCode::from_str(key).map_err(|_| ConfigureEvdevError::InvalidListenKey {
        key: key.to_string(),
    })
}

fn ensure_device_supports_key(
    device: &Device,
    listen_key_code: KeyCode,
    path: &str,
    key: &str,
) -> Result<(), ConfigureEvdevError> {
    let supports_key = device
        .supported_keys()
        .is_some_and(|keys| keys.contains(listen_key_code));

    if supports_key {
        Ok(())
    } else {
        Err(ConfigureEvdevError::UnsupportedListenKey {
            path: path.to_string(),
            key: key.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    // These tests run against our internal resolve_listen_key helper.
    // Otherwise they'd be located with the public API tests in tests/.

    use super::{
        classify_listen_key_event, resolve_listen_key, ConfigureEvdevError, ListenKeyState,
    };
    use evdev::{EventType, InputEvent, KeyCode};

    #[test]
    fn resolves_valid_listen_key_name() {
        let key = resolve_listen_key("KEY_LEFTMETA").unwrap();

        assert_eq!(key, KeyCode::KEY_LEFTMETA);
    }

    #[test]
    fn rejects_invalid_listen_key_name() {
        let err = resolve_listen_key("INVALID_KEY").unwrap_err();

        match err {
            ConfigureEvdevError::InvalidListenKey { key } => assert_eq!(key, "INVALID_KEY"),
            other => panic!("expected InvalidListenKey, got {other:?}"),
        }
    }

    #[test]
    fn classifies_matching_listen_key_press() {
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, 1);

        assert_eq!(
            classify_listen_key_event(event, KeyCode::KEY_LEFTMETA),
            Some(ListenKeyState::Pressed)
        );
    }

    #[test]
    fn classifies_matching_listen_key_release() {
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, 0);

        assert_eq!(
            classify_listen_key_event(event, KeyCode::KEY_LEFTMETA),
            Some(ListenKeyState::Released)
        );
    }

    #[test]
    fn ignores_matching_listen_key_autorepeat() {
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTMETA.0, 2);

        assert_eq!(classify_listen_key_event(event, KeyCode::KEY_LEFTMETA), None);
    }

    #[test]
    fn ignores_other_key_events() {
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.0, 1);

        assert_eq!(classify_listen_key_event(event, KeyCode::KEY_LEFTMETA), None);
    }

    #[test]
    fn ignores_non_key_events() {
        let event = InputEvent::new(EventType::RELATIVE.0, 0, 1);

        assert_eq!(classify_listen_key_event(event, KeyCode::KEY_LEFTMETA), None);
    }
}
