use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use evdev::{BusType, InputId};
use wayland_ptt::args::Config;
use wayland_ptt::evdev::{
    configure_evdev, format_input_device_metadata, ConfigureEvdevError,
};

fn make_config(path: String) -> Config {
    Config {
        verbose: false,
        listen_key: "MOUSE5".to_string(),
        send_key: "MOUSE5".to_string(),
        mouse_button: Some(5),
        input_device_path: path,
    }
}

#[test]
fn rejects_missing_input_device_path() {
    let config = make_config("/invalid/device".to_string());
    match configure_evdev(&config) {
        Err(ConfigureEvdevError::OpenDevice { path, .. }) => {
            assert_eq!(path, "/invalid/device");
        }
        Ok(_) => panic!("expected OpenDevice"),
        Err(other) => panic!("expected OpenDevice, got {other:?}"),
    }
}

#[test]
fn rejects_non_evdev_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("wayland-ptt-test-{unique}.txt"));
    fs::write(&path, b"not an evdev device").unwrap();

    let config = make_config(path.display().to_string());
    let result = configure_evdev(&config);

    fs::remove_file(&path).unwrap();

    match result {
        Err(ConfigureEvdevError::OpenDevice { path, .. }) => {
            assert!(path.contains("wayland-ptt-test-"));
        }
        Ok(_) => panic!("expected OpenDevice"),
        Err(other) => panic!("expected OpenDevice, got {other:?}"),
    }
}

#[test]
fn rejects_invalid_listen_key_on_missing_device() {
    let mut config = make_config("/invalid/device".to_string());
    config.listen_key = "INVALID_KEY".to_string();

    match configure_evdev(&config) {
        Err(ConfigureEvdevError::OpenDevice { .. }) => {}
        Ok(_) => panic!("expected an error"),
        Err(other) => panic!("expected OpenDevice, got {other:?}"),
    }
}

#[test]
fn formats_input_device_metadata_like_cpp_output() {
    let lines = format_input_device_metadata(
        Some("Test Keyboard"),
        InputId::new(BusType::BUS_USB, 0x1234, 0x5678, 0x1111),
    );

    assert_eq!(lines[0], "Input device name: \"Test Keyboard\"");
    assert_eq!(lines[1], "Input device ID: bus 0x3 vendor 0x1234 product 0x5678");
}

#[test]
fn formats_unknown_input_device_name() {
    let lines = format_input_device_metadata(
        None,
        InputId::new(BusType::BUS_BLUETOOTH, 0xabcd, 0xef01, 0x2222),
    );

    assert_eq!(lines[0], "Input device name: \"unknown\"");
    assert_eq!(lines[1], "Input device ID: bus 0x5 vendor 0xabcd product 0xef01");
}
