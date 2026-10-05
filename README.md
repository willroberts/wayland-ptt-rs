# wayland-ptt-rs
Enables push-to-talk (PTT) in X11 apps running under XWayland.

Wayland restricts input events to only the currently-active window, so this tool helps route events to inactive windows using XWayland as an X11 compatibility layer (e.g. Discord).

Based on the [original C++ version](https://github.com/Rush/wayland-push-to-talk-fix), with some language-based differences like using a pure-Rust `evdev` implementation instead of `libevdev`, and `x11rb` instead of `libxdo`.

## How It Works
- Parses user configuration for which keys to receive and send
- Listens to input events via `evdev`
- If an event matches the `listen_key`, find Discord's X11 window and send the matching press or release directly with X11 `SendEvent`

## Usage
```
wayland-ptt [-v] [--xtest] [-l listen_key] [-s send_key] /dev/input/by-id/<device-name>
```

The quickest way to find your listen key's keycode is to run the tool with `-v` against your input device. The tool will print observed input events from that device, including the keycodes. This works for keyboard and mouse events.

If `-l` is omitted, it defaults to `BTN_EXTRA`. If `-s` is omitted, it defaults to `MOUSE9`. These correspond to the "forward" side button of the mouse.

The direct Discord window backend is the default. `--xtest` selects the previous XTEST backend for comparison. Discord must expose a window on the same X11 display as this program.

To try unfocused PTT, bind Discord PTT to Mouse 9 or F13, then run (use `-s F13` for the F13 case):

```
wayland-ptt -v /dev/input/by-id/<device-name>
```

Focus a native Wayland application and hold/release the physical PTT button. Verbose output shows the detected window ID, event, press/release, and whether X11 processed the `SendEvent` request. It cannot confirm delivery or handling in Discord. Synthetic X11 events carry the `send_event` flag; Discord has been observed to ignore this experiment while unfocused. Confirm behavior with Discord's PTT indicator or audio. If it ignores the event, this direct-window approach cannot provide working background PTT by itself.

If the request completes but PTT stays inactive, inspect the logged window with `xprop -id <window-id> WM_CLASS _NET_WM_NAME WM_NAME`, then repeat while Discord is focused. If the window is Discord and the focused test works, the remaining failure is specific to background input handling. If neither test works, Discord may be ignoring synthetic events entirely.

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
