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
If `-l` and `-s` are omitted, both default to the mouse forward button.

## Reference
- [Keyboard Input Event Codes](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h)
- [Finding Mouse Input Event Codes](https://cgit.freedesktop.org/evtest/)
- [X11 KeySym Codes](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h) (ignore leading `XKB_KEY_`)
- [Finding X11 Mouse Button IDs](https://gitlab.freedesktop.org/xorg/app/xev/)

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

## License

MIT
