#include <stdio.h>
#include <fcntl.h>
#include <libevdev/libevdev.h>
extern "C" {
  #include <xdo.h> // Needed for simulating keyboard/mouse actions via X11
}
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>

int main(int argc, char **argv)
{
  struct libevdev *dev = NULL;   // libevdev device structure
  xdo_t *xdo;                    // xdo object for sending key/mouse events

  bool verbose = false;         // verbose output flag
  const char* keycode = "KEY_LEFTMETA"; // default keycode to listen for
  const char *keyname = "Super_L";      // default key to send
  int button = 0;               

  // Parse command-line options
  int opt;
  while ((opt = getopt(argc, argv, "vk:n:")) != -1) {
    switch (opt) {
      case 'v':
        verbose = true;
        break;
      case 'k': // Set keycode to listen for (Linux input event code)
        keycode = optarg;
        break;
      case 'n': // Set keyname (X11 keysym) or mouse button (e.g., MOUSE1)
        if (optarg && strlen(optarg) >= 5 && !strncmp(optarg, "MOUSE", 5)) {
          // Extract button number from "MOUSE<n>"
          button = strtol((optarg + 5), NULL, 10);

          if (errno) {
            perror("strtol");
            exit(EXIT_FAILURE);
          }
        }
        // If not mouse, treat as keysym name
        keyname = optarg;
        break;
      default:
        fprintf(stderr, "Usage: %s [-v] [-k keycode] [-n keyname] /dev/input/by-id/<device-name>\n", argv[0]);
        exit(EXIT_FAILURE);
    }
  }

  // Make sure a device path was given
  if (optind >= argc) {
    fprintf(stderr, "Usage: %s [-v] [-k keycode] [-n keyname] /dev/input/by-id/<device-name>\n", argv[0]);
    exit(EXIT_FAILURE);
  }

  // Open input device (e.g., /dev/input/by-id/...)
  int fd = open(argv[optind], O_RDONLY);
  if (fd < 0) {
    perror("Failed to open device");
    if (getuid() != 0)
      fprintf(stderr, "Fix permissions to %s or run as root\n", argv[1]);
    exit(1);
  }

  // Initialize libevdev from the file descriptor
  int rc = libevdev_new_from_fd(fd, &dev);
  if (rc < 0)
  {
    fprintf(stderr, "Failed to init libevdev (%s)\n", strerror(-rc));
    exit(1);
  }

  // Print device information
  fprintf(stderr, "Input device name: \"%s\"\n", libevdev_get_name(dev));
  fprintf(stderr, "Input device ID: bus %#x vendor %#x product %#x\n",
          libevdev_get_id_bustype(dev),
          libevdev_get_id_vendor(dev),
          libevdev_get_id_product(dev));

  // Translate string keycode (e.g., "KEY_LEFTMETA") to internal code
  int ev_keycode = libevdev_event_code_from_name(EV_KEY, keycode);
  if (ev_keycode < 0) {
    fprintf(stderr, "Key code not found\n");
    fprintf(stderr, "see https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h\n");
    exit(1);
  }

  // Verify that the input device can generate this key
  if (!libevdev_has_event_code(dev, EV_KEY, ev_keycode)) {
    fprintf(stderr, "This device is not capable of sending this key code\n");
    exit(1);
  }

  // Initialize xdo (used to send synthetic key/mouse events via X11)
  xdo = xdo_new(NULL);
  if (xdo == NULL) {
    fprintf(stderr, "Failed to initialize xdo lib\n");
    exit(1);
  }

  if (verbose) {
    fprintf(stderr, "Listening for code %s, sending %s\n", libevdev_event_code_get_name(EV_KEY, ev_keycode), keyname);
  }

  // Main event loop: wait for key press/release
  do {
    struct input_event ev;

    // Read the next input event
    rc = libevdev_next_event(dev, LIBEVDEV_READ_FLAG_NORMAL, &ev);
    if (rc != LIBEVDEV_READ_STATUS_SUCCESS)
      continue;

    // If event matches the one we're listening for, and is not auto-repeat
    if (ev.type == EV_KEY && ev.code == ev_keycode && ev.value != 2) {
      if (verbose)
        fprintf(stderr, "key %s\n", ev.value ? "up" : "down");

      if (ev.value == 1) { // Key press
        if (!button)
          xdo_send_keysequence_window_down(xdo, CURRENTWINDOW, keyname, 0);
        else
          xdo_mouse_down(xdo, CURRENTWINDOW, button);
      } else {             // Key release
        if (!button)
          xdo_send_keysequence_window_up(xdo, CURRENTWINDOW, keyname, 0);
        else
          xdo_mouse_up(xdo, CURRENTWINDOW, button);
      }
    }
  } while (rc == LIBEVDEV_READ_STATUS_SYNC || rc == LIBEVDEV_READ_STATUS_SUCCESS || rc == -EAGAIN);

  // Cleanup
  xdo_free(xdo);
  libevdev_free(dev);
  close(fd);

  return 0;
}
