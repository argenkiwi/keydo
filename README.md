# keydo

A keyboard remapping daemon for macOS, ported from [keyd](https://github.com/rvaiya/keyd).

keydo implements the same configuration language and daemon architecture as keyd — layers,
chords, overloads, macros, and IPC — using the macOS CGEventTap API for keyboard capture
and injection instead of Linux evdev.

## Status

Early development / personal use. Compatibility with the full keyd configuration surface
area is in progress.

## Requirements

- macOS (tested on macOS 13+)
- Rust toolchain (edition 2024, stable)
- Accessibility permission granted to the terminal or binary
  (System Settings → Privacy & Security → Accessibility)

## Build

```sh
cargo build --release
```

The compiled binary is at `target/release/keydo`.

## Install

```sh
sudo cp target/release/keydo /usr/local/bin/keydo
```

## Usage

Configuration files follow the keyd `.conf` format and are read from `/etc/keyd/`.

```sh
# Start the daemon (reads /etc/keyd/*.conf)
sudo keydo daemon

# Start with a specific config file
sudo keydo daemon --config ~/.config/keydo/default.conf

# Validate config files for errors
keydo check /path/to/config.conf

# List all valid key names
keydo list-keys

# Monitor key events (requires Accessibility permission)
sudo keydo monitor

# Signal running daemon to reload configs
keydo reload
```

## Configuration

keydo uses the same configuration syntax as keyd. See the
[keyd documentation](https://github.com/rvaiya/keyd/blob/master/docs/keyd.scd) for the
full reference.

Example `/etc/keyd/default.conf`:

```ini
[ids]
*

[main]
capslock = overload(nav, esc)

[nav]
h = left
j = down
k = up
l = right
```

## Acknowledgements

keydo is based on [keyd](https://github.com/rvaiya/keyd) by
[Rahul Vaiya](https://github.com/rvaiya). The configuration language, layer/chord/macro
architecture, and IPC protocol are derived from keyd. This project adapts that work to
macOS using native CGEventTap and CGEventPost APIs.

## License

MIT — see [LICENSE](LICENSE) for the full text, which includes the original copyright
notice from the keyd project.
