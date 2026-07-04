<div align="center">

# keydo

*A powerful, cross-platform keyboard remapping daemon ported from [keyd](https://github.com/rvaiya/keyd), written in Rust.*

[![Release](https://github.com/argenkiwi/keydo/workflows/Release/badge.svg)](https://github.com/argenkiwi/keydo/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)](#)

⭐ If you find this project useful, please star it on GitHub!

[Features](#features) • [Prerequisites](#prerequisites) • [Installation](#installation) • [Configuration](#configuration) • [Usage](#usage) • [Platform Specifics](#platform-specifics) • [Acknowledgments](#acknowledgments)

</div>

---

**keydo** captures keyboard input at a low level to perform complex stateful transformations, such as multi-purpose keys (e.g., Caps Lock as Escape when tapped, Control when held), custom layers, and shortcuts. 

Unlike simple key-swappers, `keydo` runs as a system daemon and interfaces directly with input systems across Linux, macOS, and Windows.

> [!NOTE]
> **What does keydo mean?**
> - **keyd oxidised:** A tribute to its roots in `keyd`, reimagined in Rust.
> - **key do:** A direct command, ordering your keys to perform exactly as you wish.
> - **The Way of the Key:** Inspired by the Japanese *dō* (道), signifying the path or discipline of mastering your input.

---

## Features

- 🎯 **Layer Support** – Create custom keyboard layers triggered by any key.
- ⚡ **Overloads** – Assign different behaviors to a key when tapped vs. held.
- 🤝 **Chords** – Trigger actions by pressing multiple keys simultaneously.
- 📜 **Macros** – Execute complex sequences of keys and text.
- 🔌 **IPC Protocol** – Interact with the running daemon to reload configurations, inject input, or monitor state.
- 💻 **Cross-Platform Backends**:
  - **Linux**: Integrates natively with `uinput` and `/dev/input`.
  - **macOS**: Uses `CGEventTap` for capture and `CGEventPost` for injection (no kernel extensions needed).
  - **Windows**: Uses a `WH_KEYBOARD_LL` hook for capture and `SendInput` for injection (no drivers needed).

---

## Prerequisites

- **OS Support**: Linux, macOS 13.0 or later, or Windows 10/11.
- **Rust Toolchain**: Rust 2024 Edition.
  - **Windows**: Build requires the **MSVC** toolchain (default) and **Visual Studio Build Tools 2019** (or later) with the "Desktop development with C++" workload installed.
- **Permissions**:
  - **macOS**: Accessibility permissions are required to capture and inject keys.
  - **Windows**: Requires an elevated terminal/daemon to remap input inside admin/elevated windows.
  - **Linux**: Root access is required to access input devices and the system IPC socket.

---

## Installation

### 1. Build the Binary
Compile and install the binary using Cargo:
```bash
cargo install --path .
```
Ensure that `~/.cargo/bin` (Linux/macOS) or `%USERPROFILE%\.cargo\bin` (Windows) is added to your system `PATH`.

### 2. Register the Service
Configure `keydo` to run automatically as a system or user service:

#### Linux
1. Copy the binary to your system path:
   ```bash
   sudo cp ~/.cargo/bin/keydo /usr/local/bin/
   ```
2. Automatically register and start the daemon (supports `systemd` and `runit`):
   ```bash
   sudo /usr/local/bin/keydo install
   ```
3. Add your user to the `keydo` group to use the CLI without root (requires logging out and back in to apply):
   ```bash
   sudo usermod -aG keydo $USER
   ```

#### macOS
Register the daemon as a user-level launchd service:
```bash
keydo install
```
> [!IMPORTANT]
> You must grant `keydo` Accessibility permissions. Go to **System Settings** → **Privacy & Security** → **Accessibility** and add the `keydo` binary (`~/.cargo/bin/keydo`).

#### Windows
Register the daemon to start automatically at logon:
```cmd
keydo install
```

---

## Configuration

`keydo` uses the same configuration language as `keyd`. By default, it reads configuration files from:
- **Linux**: `/etc/keyd/` (system-wide configuration files only)
- **macOS**: `~/.config/keydo/` (user-local configurations)
- **Windows**: `%APPDATA%\keydo\` (user-local) or `C:\ProgramData\keyd\` (system-wide)

> [!TIP]
> Refer to the official [keyd configuration documentation](https://github.com/rvaiya/keyd/blob/master/docs/keyd.scdoc) for a full reference of the syntax.

### Basic Example
Create a configuration file (e.g., `default.conf`) in your platform's configuration directory:

```ini
[ids]
*

[main]
# Maps capslock to escape when tapped, and the 'nav' layer when held
capslock = overload(nav, esc)

[nav]
h = left
j = down
k = up
l = right
```

---

## Usage

Start the daemon manually or use the CLI to interact with a running instance.

### Daemon Commands
```bash
# Start the daemon in the foreground
keydo daemon

# Run the daemon with a specific configuration file
keydo daemon --config path/to/config.conf

# Check configuration files for syntax errors
keydo check path/to/config.conf

# Reload configurations without restarting the daemon
keydo reload
```

### Monitoring & Debugging
```bash
# Monitor key events in real-time
keydo monitor

# Stream live layer state changes
keydo listen

# List all valid key names for configuration files
keydo list-keys
```

Both `monitor` and `listen` prefix every line with an absolute Unix epoch-millisecond
timestamp, so output captured from both commands at the same time can be
cross-correlated by timestamp (e.g. to see which key press caused a layer change).
Both also accept a `-j`/`--json` flag that emits one JSON object per line instead of
plain text — handy for piping logs to a script or a coding agent.

`keydo monitor` (plain text, tab-separated):
```
1751654321045	Apple Internal Keyboard	0003:05ac:0256	a down
```
Pass `-t`/`--timestamp` to additionally print the relative delta (in ms) since the
previous event, inserted as the second column:
```
1751654321045	+12 ms	Apple Internal Keyboard	0003:05ac:0256	a down
```
`keydo monitor --json`:
```json
{"seq":0,"ts_ms":1751654321045,"delta_ms":null,"device_name":"Apple Internal Keyboard","device_id":"0003:05ac:0256","keycode":30,"key":"a","event":"key","direction":"down"}
```

`keydo listen` (plain text): still uses the `+`/`-`/`/` sigil to mean
layer-activated / layer-deactivated / layout-switched, now with a leading timestamp:
```
1751654321045	+nav
1751654321980	/dvorak
```
`keydo listen --json` turns the sigil into named `event`/`action` fields:
```json
{"seq":0,"ts_ms":1751654321045,"event":"layer","layer":"nav","action":"activate"}
{"seq":1,"ts_ms":1751654321980,"event":"layout","layer":"dvorak","action":"switch"}
```

### Input & Macro Injection
```bash
# Inject raw text
keydo input "Hello, World!"

# Execute a macro sequence (e.g., Control+C, then Control+V)
keydo do "C-c C-v"

# Bind a key temporarily
keydo bind "main.j=down"
```

---

## Platform Specifics

### Linux Permissions
When running as a service, the `keydo` daemon runs as `root` to interface with `/dev/input` and `/dev/uinput`. The IPC socket `/run/keydo/socket` is owned by the `keydo` system group with `0660` permissions.

If setting permissions manually without the installer:
1. Create the socket directory with setgid group permissions:
   ```bash
   sudo mkdir -p /run/keydo
   sudo groupadd -f keydo
   sudo chown root:keydo /run/keydo
   sudo chmod 2750 /run/keydo
   ```
2. Add your user to the group:
   ```bash
   sudo usermod -aG keydo $USER
   ```
3. Log out and log back in to apply the group changes.

### Windows Details
On Windows, a system-wide low-level keyboard hook (`WH_KEYBOARD_LL`) captures events and `SendInput` injects them. Input is translated by scancode, making remappings layout-independent. IPC uses the named pipe `\\.\pipe\keydo`.

> [!WARNING]
> **Windows Limitations & Safety Hooks**
> - **Elevated Windows**: Hook input is not captured inside admin/elevated programs unless the daemon itself runs in an elevated command prompt.
> - **Anti-Cheat & Games**: Some games/anti-cheat systems block or flag `SendInput`-injected events.
> - **Console Window**: The daemon currently runs in a visible console window when started.
> - **Unicode Macros**: Unicode composition sequences are currently macOS-only.
> - **No Device Matching**: All keyboards appear as a single device; per-device matching via `[ids]` is not supported.
> - **Emergency Panic Sequence**: If input locks up or remapping stops under heavy load, press and hold **Backspace + Enter + Escape** to instantly terminate the daemon and restore default keyboard behavior.

---

## Acknowledgments

This project is a port of [keyd](https://github.com/rvaiya/keyd) created by [Raheman Vaiya](https://github.com/rvaiya). We are incredibly grateful for his work on the design and configuration syntax.
