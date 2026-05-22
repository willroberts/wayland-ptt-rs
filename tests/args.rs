use wayland_ptt::args::{parse_args_from, usage, Config};

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn parses_defaults_with_device_path() {
    let parsed = parse_args_from(args(&["wayland-ptt", "/dev/input/by-id/test-kbd"])).unwrap();

    assert_eq!(
        parsed,
        Config {
            verbose: false,
            listen_key: "KEY_LEFTMETA".to_string(),
            send_key: "Super_L".to_string(),
            mouse_button: None,
            input_device_path: "/dev/input/by-id/test-kbd".to_string(),
        }
    );
}

#[test]
fn parses_all_supported_flags() {
    let parsed = parse_args_from(args(&[
        "wayland-ptt",
        "-v",
        "-l",
        "KEY_F13",
        "-s",
        "F24",
        "/dev/input/by-id/test-kbd",
    ]))
    .unwrap();

    assert_eq!(
        parsed,
        Config {
            verbose: true,
            listen_key: "KEY_F13".to_string(),
            send_key: "F24".to_string(),
            mouse_button: None,
            input_device_path: "/dev/input/by-id/test-kbd".to_string(),
        }
    );
}

#[test]
fn parses_mouse_target() {
    let parsed = parse_args_from(args(&[
        "wayland-ptt",
        "-s",
        "MOUSE5",
        "/dev/input/by-id/test-mouse",
    ]))
    .unwrap();

    assert_eq!(
        parsed,
        Config {
            verbose: false,
            listen_key: "KEY_LEFTMETA".to_string(),
            send_key: "MOUSE5".to_string(),
            mouse_button: Some(5),
            input_device_path: "/dev/input/by-id/test-mouse".to_string(),
        }
    );
}

#[test]
fn rejects_invalid_mouse_target() {
    let err = parse_args_from(args(&[
        "wayland-ptt",
        "-s",
        "MOUSEabc",
        "/dev/input/by-id/test-mouse",
    ]))
    .unwrap_err();

    assert_eq!(err, "Invalid mouse_button value: MOUSEabc");
}

#[test]
fn rejects_missing_device_path() {
    let err = parse_args_from(args(&["wayland-ptt", "-v"])).unwrap_err();

    assert_eq!(err, usage("wayland-ptt"));
}

#[test]
fn rejects_unknown_flag() {
    let err =
        parse_args_from(args(&["wayland-ptt", "-x", "/dev/input/by-id/test-kbd"])).unwrap_err();

    assert_eq!(err, usage("wayland-ptt"));
}

#[test]
fn rejects_multiple_device_paths() {
    let err = parse_args_from(args(&[
        "wayland-ptt",
        "/dev/input/by-id/one",
        "/dev/input/by-id/two",
    ]))
    .unwrap_err();

    assert_eq!(err, usage("wayland-ptt"));
}
