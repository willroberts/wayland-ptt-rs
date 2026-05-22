.PHONY: build install

build:
	rm -f target/release/wayland-ptt
	cargo build --release

install:
	install -m 755 target/release/wayland-ptt /usr/bin
	install -m 644 wayland-ptt.desktop /etc/xdg/autostart
