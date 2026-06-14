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

1. **Install the binary:**
   ```bash
   cargo install --path .
   ```
   The binary is placed in `~/.cargo/bin/keydo` (Linux/macOS) or `%USERPROFILE%\.cargo\bin\keydo.exe` (Windows). Make sure this directory is on your `PATH`.

2. **Grant Permissions (macOS only):**
   Go to **System Settings** → **Privacy & Security** → **Accessibility** and add the `keydo` binary (`~/.cargo/bin/keydo`).

3. **Register as a background service:**
   ```bash
   keydo install
   ```
   This writes and loads the appropriate service descriptor for your platform:
   - **macOS:** a `LaunchAgent` plist in `~/Library/LaunchAgents/`
   - **Linux (systemd):** a unit file in `/etc/systemd/system/` (requires root)
   - **Linux (runit):** a run script in `/etc/sv/keydo/` with a symlink in `/var/service/` (requires root)
   - **Windows:** an `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registry value that starts `keydo daemon` at logon (no admin rights required; low-level hooks cannot run from a session-0 service)

   On Linux the init system is auto-detected. To specify it explicitly:
   ```bash
   sudo keydo install --init systemd
   sudo keydo install --init runit
   ```

   To remove the service:
   ```bash
   keydo uninstall          # macOS / Windows
   sudo keydo uninstall     # Linux
   ```

### Windows Quick-Start

1. Install [rustup](https://rustup.rs/) and, when prompted, choose the default **MSVC** toolchain.
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select **Desktop development with C++**.
3. Build and install keydo:
   ```powershell
   cargo install --path .
   ```
4. Place your config file at `%APPDATA%\keydo\default.conf` (create the directory if it does not exist).
5. Register the auto-start entry:
   ```powershell
   keydo install
   ```
6. Start the daemon immediately (without rebooting):
   ```powershell
   keydo daemon
   ```
   A console window will appear while the daemon is running. Subsequent logins will start it automatically via the registry Run key.

> [!NOTE]
> To remap keys inside applications running as Administrator, launch the daemon from an elevated PowerShell prompt (`Run as Administrator`). The IPC pipe (`\\.\pipe\keydo`) is accessible from both elevated and non-elevated terminals.

### Configuration

`keydo` uses the same configuration language as `keyd`. By default, it looks for `.conf` files in `~/.config/keydo/`, falling back to `/etc/keyd/` if the former does not exist. On Windows the directories are `%APPDATA%\keydo\` and `C:\ProgramData\keyd\`.

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
