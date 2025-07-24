CXX = g++
CXXFLAGS = -Os $(shell pkg-config --cflags libevdev)
LDFLAGS = $(shell pkg-config --libs libevdev) -lxdo

.PHONY: all clean install

all: push-to-talk

push-to-talk: push-to-talk.cpp
        $(CXX) $(CXXFLAGS) push-to-talk.cpp -o push-to-talk $(LDFLAGS)

clean:
        rm -f push-to-talk

install:
        install -m 755 push-to-talk /usr/bin
        install -m 644 push-to-talk.desktop /etc/xdg/autostart