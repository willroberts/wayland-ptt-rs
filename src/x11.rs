use std::fmt;

use x11rb::connection::{Connection, RequestConnection};
use x11rb::errors::{ConnectError, ConnectionError, ReplyError};
use x11rb::protocol::xproto::{
    BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xtest::{ConnectionExt as _, X11_EXTENSION_NAME};
use x11rb::rust_connection::RustConnection;
use xkbcommon_rs::keysym::keysym_from_name;

use crate::args::Config;
use crate::evdev::ListenKeyState;

const XTEST_MAJOR_VERSION: u8 = 2;
const XTEST_MINOR_VERSION: u16 = 2;

pub struct X11Config {
    pub connection: RustConnection,
    pub screen_num: usize,
    pub xtest_major_version: u8,
    pub xtest_minor_version: u16,
}

#[derive(Debug)]
pub enum ConfigureX11Error {
    Connect {
        display_name: Option<String>,
        source: ConnectError,
    },
    MissingXtestExtension,
    QueryXtestVersion {
        source: ReplyError,
    },
    InvalidSendKey {
        key: String,
    },
    UnmappedSendKey {
        key: String,
    },
    InvalidMouseButton {
        button: u32,
    },
    QueryKeyboardMapping {
        source: ReplyError,
    },
    SendInput {
        source: ReplyError,
    },
}

impl fmt::Display for ConfigureX11Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect {
                display_name,
                source,
            } => match display_name {
                Some(display_name) => write!(
                    f,
                    "Failed to connect to X11 display {display_name}: {source}"
                ),
                None => write!(f, "Failed to connect to the default X11 display: {source}"),
            },
            Self::MissingXtestExtension => {
                write!(f, "X11 server does not support the XTEST extension")
            }
            Self::QueryXtestVersion { source } => {
                write!(f, "Failed to query XTEST version: {source}")
            }
            Self::InvalidSendKey { key } => {
                write!(f, "Failed to resolve X11 send key: {key}")
            }
            Self::UnmappedSendKey { key } => {
                write!(f, "X11 server has no keycode mapped for send key: {key}")
            }
            Self::InvalidMouseButton { button } => {
                write!(f, "Invalid X11 mouse button: {button}")
            }
            Self::QueryKeyboardMapping { source } => {
                write!(f, "Failed to query X11 keyboard mapping: {source}")
            }
            Self::SendInput { source } => {
                write!(f, "Failed to send X11 input event: {source}")
            }
        }
    }
}

impl std::error::Error for ConfigureX11Error {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum X11Target {
    Key { keycode: u8 },
    MouseButton { button: u8 },
}

pub fn configure_x11() -> Result<X11Config, ConfigureX11Error> {
    configure_x11_with_display_name(None)
}

pub fn configure_x11_with_display_name(
    display_name: Option<&str>,
) -> Result<X11Config, ConfigureX11Error> {
    let (connection, screen_num) =
        x11rb::connect(display_name).map_err(|source| ConfigureX11Error::Connect {
            display_name: display_name.map(ToOwned::to_owned),
            source,
        })?;

    ensure_xtest_extension(&connection)?;

    let version = connection
        .xtest_get_version(XTEST_MAJOR_VERSION, XTEST_MINOR_VERSION)
        .map_err(connection_error_to_configure_x11_error)?
        .reply()
        .map_err(|source| ConfigureX11Error::QueryXtestVersion { source })?;

    Ok(X11Config {
        connection,
        screen_num,
        xtest_major_version: version.major_version,
        xtest_minor_version: version.minor_version,
    })
}

pub fn configure_x11_target(
    x11_config: &X11Config,
    config: &Config,
) -> Result<X11Target, ConfigureX11Error> {
    if let Some(button) = config.mouse_button {
        let button =
            u8::try_from(button).map_err(|_| ConfigureX11Error::InvalidMouseButton { button })?;
        return Ok(X11Target::MouseButton { button });
    }

    let keysym = resolve_send_keysym(&config.send_key)?;
    let keycode = find_keycode_for_keysym(x11_config, keysym, &config.send_key)?;

    Ok(X11Target::Key { keycode })
}

pub fn send_target_state(
    x11_config: &X11Config,
    target: X11Target,
    state: ListenKeyState,
) -> Result<(), ConfigureX11Error> {
    let (event_type, detail) = event_type_and_detail(target, state);

    let cookie = x11_config
        .connection
        .xtest_fake_input(event_type, detail, x11rb::CURRENT_TIME, x11rb::NONE, 0, 0, 0)
        .map_err(connection_error_to_send_input_error)?;
    cookie
        .check()
        .map_err(|source| ConfigureX11Error::SendInput { source })?;
    x11_config
        .connection
        .flush()
        .map_err(connection_error_to_send_input_error)?;

    Ok(())
}

fn ensure_xtest_extension(connection: &RustConnection) -> Result<(), ConfigureX11Error> {
    let extension = connection
        .extension_information(X11_EXTENSION_NAME)
        .map_err(connection_error_to_configure_x11_error)?;

    if extension.is_some() {
        Ok(())
    } else {
        Err(ConfigureX11Error::MissingXtestExtension)
    }
}

fn connection_error_to_configure_x11_error(source: ConnectionError) -> ConfigureX11Error {
    match source {
        ConnectionError::UnsupportedExtension => ConfigureX11Error::MissingXtestExtension,
        source => ConfigureX11Error::QueryXtestVersion {
            source: ReplyError::ConnectionError(source),
        },
    }
}

fn connection_error_to_send_input_error(source: ConnectionError) -> ConfigureX11Error {
    ConfigureX11Error::SendInput {
        source: ReplyError::ConnectionError(source),
    }
}

fn resolve_send_keysym(key: &str) -> Result<u32, ConfigureX11Error> {
    keysym_from_name(key, 0).map(u32::from).ok_or_else(|| ConfigureX11Error::InvalidSendKey {
        key: key.to_string(),
    })
}

fn find_keycode_for_keysym(
    x11_config: &X11Config,
    keysym: u32,
    key: &str,
) -> Result<u8, ConfigureX11Error> {
    let setup = x11_config.connection.setup();
    let first_keycode = setup.min_keycode;
    let keycode_count = setup.max_keycode - setup.min_keycode + 1;
    let mapping = x11_config
        .connection
        .get_keyboard_mapping(first_keycode, keycode_count)
        .map_err(|source| ConfigureX11Error::QueryKeyboardMapping {
            source: ReplyError::ConnectionError(source),
        })?
        .reply()
        .map_err(|source| ConfigureX11Error::QueryKeyboardMapping { source })?;

    let keysyms_per_keycode = usize::from(mapping.keysyms_per_keycode);
    for (index, keysyms) in mapping.keysyms.chunks(keysyms_per_keycode).enumerate() {
        if keysyms.contains(&keysym) {
            return Ok(first_keycode + index as u8);
        }
    }

    Err(ConfigureX11Error::UnmappedSendKey {
        key: key.to_string(),
    })
}

fn event_type_and_detail(target: X11Target, state: ListenKeyState) -> (u8, u8) {
    match (target, state) {
        (X11Target::Key { keycode }, ListenKeyState::Pressed) => (KEY_PRESS_EVENT, keycode),
        (X11Target::Key { keycode }, ListenKeyState::Released) => (KEY_RELEASE_EVENT, keycode),
        (X11Target::MouseButton { button }, ListenKeyState::Pressed) => {
            (BUTTON_PRESS_EVENT, button)
        }
        (X11Target::MouseButton { button }, ListenKeyState::Released) => {
            (BUTTON_RELEASE_EVENT, button)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        event_type_and_detail, resolve_send_keysym, ConfigureX11Error, X11Target,
    };
    use crate::evdev::ListenKeyState;
    use x11rb::protocol::xproto::{
        BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
    };

    #[test]
    fn resolves_valid_send_key_name() {
        // Uses the left SUPER key as a test key.
        let keysym = resolve_send_keysym("Super_L").unwrap();
        assert_ne!(keysym, 0);
    }

    #[test]
    fn rejects_invalid_send_key_name() {
        let err = resolve_send_keysym("NOT_A_REAL_X11_KEY").unwrap_err();

        match err {
            ConfigureX11Error::InvalidSendKey { key } => {
                assert_eq!(key, "NOT_A_REAL_X11_KEY")
            }
            other => panic!("expected InvalidSendKey, got {other:?}"),
        }
    }

    #[test]
    fn maps_key_press_event_type_and_detail() {
        assert_eq!(
            event_type_and_detail(
                X11Target::Key { keycode: 42 },
                ListenKeyState::Pressed
            ),
            (KEY_PRESS_EVENT, 42)
        );
    }

    #[test]
    fn maps_key_release_event_type_and_detail() {
        assert_eq!(
            event_type_and_detail(
                X11Target::Key { keycode: 42 },
                ListenKeyState::Released
            ),
            (KEY_RELEASE_EVENT, 42)
        );
    }

    #[test]
    fn maps_mouse_press_event_type_and_detail() {
        assert_eq!(
            event_type_and_detail(
                X11Target::MouseButton { button: 5 },
                ListenKeyState::Pressed
            ),
            (BUTTON_PRESS_EVENT, 5)
        );
    }

    #[test]
    fn maps_mouse_release_event_type_and_detail() {
        assert_eq!(
            event_type_and_detail(
                X11Target::MouseButton { button: 5 },
                ListenKeyState::Released
            ),
            (BUTTON_RELEASE_EVENT, 5)
        );
    }
}
