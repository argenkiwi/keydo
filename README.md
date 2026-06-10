# keydo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![macOS](https://img.shields.io/badge/platform-macOS-blue.svg)](https://www.apple.com/macos/)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)

**keydo** is a powerful keyboard remapping daemon for macOS, faithfully ported from [keyd](https://github.com/rvaiya/keyd). It brings the flexibility of Linux-style keyboard customization to macOS, implementing layers, chords, overloads, macros, and a full IPC protocol using native macOS APIs.

Unlike many macOS remappers that rely on simple key swaps, `keydo` captures input at a low level using `CGEventTap`, allowing for complex stateful transformations like multi-purpose keys (e.g., Caps Lock as Escape when tapped, Control when held).

## Key Features

- **Layer Support:** Create custom keyboard layers triggered by any key.
- **Overloads:** Assign different behaviors to a key when tapped vs. held.
- **Chords:** Trigger actions by pressing multiple keys simultaneously.
- **Macros:** Execute complex sequences of keys and text.
- **IPC Protocol:** Interact with the running daemon to reload configs, inject input, or monitor state.
- **Native macOS Backend:** Uses `CGEventTap` for capture and `CGEventPost` for injection (no kernel extensions required).

## Prerequisites

- **OS:** macOS 13.0 or later.
- **Permissions:** `keydo` requires **Accessibility** permissions to capture and inject keyboard events.
- **Rust:** A modern Rust toolchain (Edition 2024).

## Getting Started

### Installation

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Install the binary:**
   ```bash
   sudo cp target/release/keydo /usr/local/bin/keydo
   ```

3. **Grant Permissions:**
   Go to **System Settings** → **Privacy & Security** → **Accessibility** and add either your Terminal (for testing) or the `keydo` binary.

### Configuration

`keydo` uses the same configuration language as `keyd`. By default, it looks for `.conf` files in `/etc/keyd/`.

> [!TIP]
> Check out the [keyd documentation](https://github.com/rvaiya/keyd/blob/master/docs/keyd.scd) for a full reference of the configuration syntax.

#### Basic Example (`/etc/keyd/default.conf`)

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
# Start the daemon (reads /etc/keyd/*.conf)
sudo keydo daemon

# Run with a specific config file
sudo keydo daemon --config ~/.config/keydo/work.conf

# Monitor key events in real-time
sudo keydo monitor

# Validate your configuration files
keydo check /etc/keyd/default.conf

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

This project is a port of [keyd](https://github.com/rvaiya/keyd) by [Rahul Vaiya](https://github.com/rvaiya). We are grateful for his work on the original architecture and configuration language.
