# wayland-ptt-rs
Enables push-to-talk (PTT) in X11 apps running under XWayland.

Wayland restricts input events to only the currently-active window, so this tool helps route events to inactive windows using XWayland as an X11 compatibility layer (e.g. Discord).

Based on the [original C++ version](https://github.com/Rush/wayland-push-to-talk-fix), with some language-based differences like using a pure-Rust `evdev` implementation instead of `libevdev`, and `x11rb` instead of `libxdo`.

## How It Works
- Parses user configuration for which keys to receive and send
- Listens to input events via `evdev`
- If an event matches the `listen_key`, send input to X11 via `x11rb`

## Usage
```
wayland-ptt [-v] [-l listen_key] [-s send_key] /dev/input/by-id/<device-name>
```

The quickest way to find your listen key's keycode is to run the tool with `-v` against your input device. The tool will print observed input events from that device, including the keycodes. This works for keyboard and mouse events.

If `-l` is omitted, it defaults to `BTN_EXTRA`. If `-s` is omitted, it defaults to `MOUSE9`. These correspond to the "forward" side button of the mouse.

## Installation
Edit `wayland-ptt.desktop` and replace `/dev/input/by-id/<device-id>` with your desired device path, then install:
```
cargo build --release
sudo make install
```
To access input devices without superuser privileges, add your user to the `input` group:
```
sudo usermod -aG input <user>
```
Confirm things are working with `ps | grep wayland-ptt` after logging in.

## Reference
- [X11 KeySym Codes](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h) (ignore leading `XKB_KEY_`)
- [Finding X11 Mouse Button IDs](https://gitlab.freedesktop.org/xorg/app/xev/)

## License

MIT
