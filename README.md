# keydo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Linux](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://www.kernel.org/)
[![macOS](https://img.shields.io/badge/platform-macOS-blue.svg)](https://www.apple.com/macos/)
[![Windows](https://img.shields.io/badge/platform-Windows-green.svg)](https://www.microsoft.com/windows/)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)

**keydo** is a powerful keyboard remapping daemon ported from [keyd](https://github.com/rvaiya/keyd), running on Linux, macOS, and Windows. It implements layers, chords, overloads, macros, and a full IPC protocol — and extends the original by adding native macOS support via `CGEventTap` and native Windows support via a low-level keyboard hook.

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
- **Native Windows Backend:** Uses a `WH_KEYBOARD_LL` hook for capture and `SendInput` for injection (no drivers required).

## Prerequisites

- **OS:** Linux, macOS 13.0 or later, or Windows 10/11.
- **Permissions (macOS):** `keydo` requires **Accessibility** permissions to capture and inject keyboard events via `CGEventTap`.
- **Permissions (Windows):** No special permissions are needed for most use cases. To remap input inside elevated (admin) windows, run the daemon from an elevated terminal.
- **Rust:** A modern Rust toolchain (Edition 2024). On Windows, install via [rustup](https://rustup.rs/) and ensure the **MSVC** toolchain is active (the default on Windows). The **Visual Studio Build Tools 2019** or later with the "Desktop development with C++" workload is required as the linker backend.

## Getting Started

### Installation

1. **Build and install the binary:**
   ```bash
   cargo install --path .
   ```
   The binary is placed in `~/.cargo/bin/keydo` (Linux/macOS) or `%USERPROFILE%\.cargo\bin\keydo.exe` (Windows). Ensure this directory is on your `PATH`.

2. **Platform-specific setup:**

#### Linux
- **System Path:** To make `keydo` available for system-wide commands (which require `sudo`), copy it to `/usr/local/bin`:
  ```bash
  sudo cp ~/.cargo/bin/keydo /usr/local/bin/
  ```
- **Service:** Register the daemon as a system-wide service (supports systemd and runit):
  ```bash
  sudo keydo install
  ```
- **Permissions:** `keydo` runs as a dedicated system user. The installer adds your user to the `keydo` group, but you must **log out and back in** for this to take effect. If setting up manually, ensure your user can access the IPC socket and `/dev/uinput` has a udev rule (see [Linux Permissions](#linux-permissions) below).

#### macOS
- **Service:** Register the daemon as a user-level service:
  ```bash
  keydo install
  ```
- **Permissions:** Go to **System Settings** → **Privacy & Security** → **Accessibility** and add the `keydo` binary (`~/.cargo/bin/keydo`).

#### Windows
- **Service:** Register the daemon to start at logon:
  ```bash
  keydo install
  ```
- **Elevation:** To remap input inside elevated (admin) windows, you must run the daemon from an elevated terminal. The IPC pipe is accessible from both elevated and non-elevated terminals.

### Configuration

`keydo` uses the same configuration language as `keyd`. By default, it looks for `.conf` files in the following locations:
- **Linux/macOS:** `~/.config/keydo/` (user) or `/etc/keyd/` (system)
- **Windows:** `%APPDATA%\keydo\` (user) or `C:\ProgramData\keyd\` (system)

> [!TIP]
> Check out the [keyd documentation](https://github.com/rvaiya/keyd/blob/master/docs/keyd.scdoc) for a full reference of the configuration syntax.

#### Basic Example

Place this file at `~/.config/keydo/default.conf` (Linux/macOS) or `%APPDATA%\keydo\default.conf` (Windows):

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
# Start the daemon manually
keydo daemon

# Run with a specific config file
keydo daemon --config path/to/config.conf

# Monitor key events in real-time
keydo monitor

# Validate your configuration files
keydo check path/to/config.conf

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

## Linux Permissions

When running as a service on Linux, `keydo` runs as a dedicated system user (`keydo`) with permission to read from `/dev/input/*` and write to `/dev/uinput`.

The installer (`sudo keydo install`) automatically:
1. Creates a `keydo` system user and group.
2. Adds the `keydo` user to the `input` and `uinput` groups.
3. Adds your current user (via `SUDO_USER`) to the `keydo` group so you can use the CLI.

If you are setting permissions manually:
1. **Ensure `/dev/uinput` has the correct group permissions:**
   Create `/etc/udev/rules.d/99-keydo.rules`:
   ```udev
   KERNEL=="uinput", GROUP="uinput", MODE="0660", OPTIONS+="static_node=uinput"
   ```
2. **Ensure your user can access the IPC socket:**
   ```bash
   sudo usermod -aG keydo $USER
   ```
3. **Apply changes:**
   ```bash
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```
   **Log out and back in** for group changes to take effect.

## Windows Notes

The Windows backend mirrors the macOS design: a single system-wide low-level keyboard hook (`WH_KEYBOARD_LL`) captures and swallows hardware key events, the remapping engine processes them, and the result is re-injected with `SendInput` (tagged so the hook ignores keydo's own output). Keys are translated by **scancode**, so remapping is keyboard-layout independent, exactly as on Linux. IPC between the CLI and the daemon uses the named pipe `\\.\pipe\keydo`.

Known limitations:

- **Elevated windows:** input destined for elevated (admin) applications is not visible to a non-elevated hook. Run `keydo daemon` from an elevated terminal if you need remapping inside admin apps.
- **Injected input is distinguishable:** some games and anti-cheat systems ignore or flag `SendInput`-injected events.
- **Console window:** `keydo daemon` runs in a console window, including when auto-started at logon.
- **Unicode macros:** the unicode composition sequences are macOS-specific and do not yet produce the right characters on Windows.
- **Per-device `[ids]` matching is unavailable:** the low-level hook cannot distinguish keyboards, so all input appears as a single device (same as macOS).

If remapping suddenly stops while typing under heavy system load, Windows may have silently removed the hook (it evicts hooks whose callbacks run too slowly); restart the daemon. The panic sequence (hold **backspace + enter + escape**) immediately terminates the daemon and restores normal input.

## Acknowledgments

This project is a port of [keyd](https://github.com/rvaiya/keyd) by [Raheman Vaiya](https://github.com/rvaiya). We are grateful for his work on the original architecture and configuration language.
