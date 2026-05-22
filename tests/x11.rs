use wayland_ptt::x11::{configure_x11_with_display_name, ConfigureX11Error};

#[test]
fn rejects_invalid_display_name() {
    match configure_x11_with_display_name(Some("definitely-invalid-display-name")) {
        Err(ConfigureX11Error::Connect {
            display_name: Some(display_name),
            ..
        }) => assert_eq!(display_name, "definitely-invalid-display-name"),
        Ok(_) => panic!("expected invalid display name to fail"),
        Err(other) => panic!("expected Connect error, got {other:?}"),
    }
}

#[test]
fn formats_default_display_connection_error() {
    let err = ConfigureX11Error::Connect {
        display_name: None,
        source: x11rb::errors::ConnectError::DisplayParsingError(
            x11rb::errors::DisplayParsingError::Unknown,
        ),
    };

    assert_eq!(
        err.to_string(),
        "Failed to connect to the default X11 display: Unknown error while parsing a $DISPLAY address"
    );
}

#[test]
fn formats_named_display_connection_error() {
    let err = ConfigureX11Error::Connect {
        display_name: Some(":9".to_string()),
        source: x11rb::errors::ConnectError::DisplayParsingError(
            x11rb::errors::DisplayParsingError::Unknown,
        ),
    };

    assert_eq!(
        err.to_string(),
        "Failed to connect to X11 display :9: Unknown error while parsing a $DISPLAY address"
    );
}
