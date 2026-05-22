use std::fmt;

use x11rb::connection::RequestConnection;
use x11rb::errors::{ConnectError, ConnectionError, ReplyError};
use x11rb::protocol::xtest::{ConnectionExt as _, X11_EXTENSION_NAME};
use x11rb::rust_connection::RustConnection;

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
        }
    }
}

impl std::error::Error for ConfigureX11Error {}

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
