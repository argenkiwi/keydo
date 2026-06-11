# keydo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Linux](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://www.kernel.org/)
[![macOS](https://img.shields.io/badge/platform-macOS-blue.svg)](https://www.apple.com/macos/)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)

**keydo** is a powerful keyboard remapping daemon ported from [keyd](https://github.com/rvaiya/keyd), running on both Linux and macOS. It implements layers, chords, overloads, macros, and a full IPC protocol — and extends the original by adding native macOS support via `CGEventTap`.

Unlike many remappers that rely on simple key swaps, `keydo` captures input at a low level, allowing for complex stateful transformations like multi-purpose keys (e.g., Caps Lock as Escape when tapped, Control when held).

The name **keydo** carries a triple meaning:
- **keyd oxidised:** A tribute to its roots in `keyd`, now reimagined in Rust.
- **key do:** A direct command, ordering your keys to perform exactly as you wish.
- **The Way of the Key:** Inspired by the Japanese *dō* (道), signifying the path or discipline of mastering your input.

## Key Features

- **Layer Support:** Create custom keyboard layers triggered by any key.
- **Overloads:** Assign different behaviors to a key when tapped vs. held.
- **Chords:** Trigger actions by pressing multiple keys simultaneously.
- **Macros:** Execute complex sequences of keys and text.
- **IPC Protocol:** Interact with the running daemon to reload configs, inject input, or monitor state.
- **Native macOS Backend:** Uses `CGEventTap` for capture and `CGEventPost` for injection (no kernel extensions required).

## Prerequisites

- **OS:** Linux, or macOS 13.0 or later.
- **Permissions (macOS):** `keydo` requires **Accessibility** permissions to capture and inject keyboard events via `CGEventTap`.
- **Rust:** A modern Rust toolchain (Edition 2024).

## Getting Started

### Installation

1. **Install the binary:**
   ```bash
   cargo install --path .
   ```

2. **Grant Permissions (macOS):**
   Go to **System Settings** → **Privacy & Security** → **Accessibility** and add the `keydo` binary (`~/.cargo/bin/keydo`).

3. **Register as a background service:**
   ```bash
   keydo install
   ```
   This writes and loads the appropriate service descriptor for your platform:
   - **macOS:** a `LaunchAgent` plist in `~/Library/LaunchAgents/`
   - **Linux (systemd):** a unit file in `/etc/systemd/system/` (requires root)
   - **Linux (runit):** a run script in `/etc/sv/keydo/` with a symlink in `/var/service/` (requires root)

   On Linux the init system is auto-detected. To specify it explicitly:
   ```bash
   sudo keydo install --init systemd
   sudo keydo install --init runit
   ```

   To remove the service:
   ```bash
   keydo uninstall          # macOS
   sudo keydo uninstall     # Linux
   ```

### Configuration

`keydo` uses the same configuration language as `keyd`. By default, it looks for `.conf` files in `~/.config/keydo/`, falling back to `/etc/keyd/` if the former does not exist.

> [!TIP]
> Check out the [keyd documentation](https://github.com/rvaiya/keyd/blob/master/docs/keyd.scdoc) for a full reference of the configuration syntax.

#### Basic Example (`~/.config/keydo/default.conf` or `/etc/keyd/default.conf`)

```ini
[ids]
*

[main]
# Maps capslock to escape when tapped and the 'nav' layer when held.
capslock = overload(nav, esc)

[nav]
h = left
j = down
k = up
l = right
```

## Usage

`keydo` provides a versatile CLI for managing the daemon and interacting with your keyboard.

```bash
# Start the daemon manually (reads ~/.config/keydo/*.conf or /etc/keyd/*.conf)
keydo daemon

# Run with a specific config file
keydo daemon --config ~/.config/keydo/work.conf

# Monitor key events in real-time
sudo keydo monitor

# Validate your configuration files
keydo check ~/.config/keydo/default.conf

# Reload configurations without restarting the daemon
keydo reload

# List all valid key names for use in configs
keydo list-keys
```

### Advanced Commands

- **Inject Text:** `keydo input "Hello, World!"`
- **Execute Macro:** `keydo do "C-c C-v"`
- **Live Binding:** `keydo bind "main.j=down"`
- **Listen for state:** `keydo listen` (streams layer changes)

## Acknowledgments

This project is a port of [keyd](https://github.com/rvaiya/keyd) by [Raheman Vaiya](https://github.com/rvaiya). We are grateful for his work on the original architecture and configuration language.
