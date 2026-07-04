pub mod error;
pub mod keys;
pub mod macro_types;
pub mod config;
pub mod config_validate;
pub mod ini;
pub mod unicode;
pub mod macro_parse;
pub mod config_parse;
pub mod config_impl;
pub mod vkbd;
pub mod device;
pub mod keyboard_types;
pub mod keyboard_impl;
pub mod daemon;
pub mod ipc;
pub mod install;
#[cfg(target_os = "macos")]
pub mod macos_input;
// Translation tables compile (and are tested) on every platform; the hook FFI
// inside is cfg(windows)-gated.
#[cfg_attr(not(windows), allow(dead_code))]
pub mod windows_input;
#[cfg(test)]
pub mod tests;
#[cfg(test)]
pub mod test_io;

use clap::{Parser, Subcommand};
use crate::daemon::Daemon;
use crate::device::{Device, DeviceEventType};
use crate::ipc::{IpcMessage, IpcMessageType};
use crate::keys::KEYCODE_TABLE;
use std::io::{self, BufRead, Read, Write};
use std::process;

// ── CLI definition ─────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    about   = "A key remapping daemon.",
    long_about = None,
    // With no subcommand, run the daemon (matches C behaviour).
    arg_required_else_help = false,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the keyd daemon (default when no subcommand is given)
    Daemon {
        /// Load a single config file instead of scanning default directories (/etc/keyd/ on Linux, ~/.config/keydo/ on macOS, %APPDATA%\keydo\ on Windows)
        #[arg(short, long)]
        config: Option<String>,
    },

    /// List all valid key names
    #[command(name = "list-keys")]
    ListKeys,

    /// Print key events in real time (requires root)
    Monitor {
        /// Print time in milliseconds since the previous event
        #[arg(short = 't', long)]
        timestamp: bool,

        /// Emit one JSON object per line instead of plain text
        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Check config files for errors
    Check {
        /// Files to check (all .conf files in default directories if omitted)
        files: Vec<String>,
    },

    /// Signal the running daemon to reload its configs
    Reload,

    /// Add bindings to all loaded configs at runtime
    Bind {
        /// Binding expressions e.g. main.a=b
        bindings: Vec<String>,
    },

    /// Execute a macro expression via the daemon
    #[command(name = "do")]
    DoMacro {
        /// Inter-key delay in microseconds
        #[arg(short = 't', long)]
        timeout: Option<u32>,
        /// Macro expression (read from stdin if omitted)
        #[arg(trailing_var_arg = true)]
        expr: Vec<String>,
    },

    /// Type text via the virtual keyboard
    Input {
        /// Inter-key delay in microseconds
        #[arg(short = 't', long)]
        timeout: Option<u32>,
        /// Text to type (read from stdin if omitted)
        #[arg(trailing_var_arg = true)]
        text: Vec<String>,
    },

    /// Stream layer changes from the running daemon
    Listen {
        /// Emit one JSON object per line instead of plain text
        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Register keydo as a persistent background service
    Install {
        /// Init system to use: auto, systemd, or runit (Linux only; auto-detected if omitted)
        #[cfg_attr(not(target_os = "linux"), arg(hide = true))]
        #[arg(long, value_enum, default_value = "auto")]
        init: install::InitSystem,
    },

    /// Remove the keydo background service
    Uninstall {
        /// Init system to use: auto, systemd, or runit (Linux only; auto-detected if omitted)
        #[cfg_attr(not(target_os = "linux"), arg(hide = true))]
        #[arg(long, value_enum, default_value = "auto")]
        init: install::InitSystem,
    },
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Returns the default configuration directory.
/// macOS: ~/.config/keydo/.
/// Linux: /etc/keyd/.
/// Windows: %APPDATA%\keydo\ first, then C:\ProgramData\keyd\.
#[cfg(target_os = "macos")]
fn get_config_dir() -> String {
    let home = std::env::var_os("HOME").unwrap_or_else(|| std::ffi::OsString::from("/"));
    std::path::PathBuf::from(home).join(".config/keydo").to_string_lossy().into_owned()
}

#[cfg(target_os = "linux")]
fn get_config_dir() -> String {
    "/etc/keyd/".to_string()
}

#[cfg(windows)]
fn get_config_dir() -> String {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let path = std::path::PathBuf::from(appdata).join("keydo");
        if path.is_dir() {
            return path.to_string_lossy().into_owned();
        }
    }
    r"C:\ProgramData\keyd".to_string()
}

/// Read a text payload: from `args` (space-joined) or stdin if args is empty.
fn read_payload(args: &[String]) -> Vec<u8> {
    if args.is_empty() {
        let mut buf = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buf) {
            eprintln!("WARNING: failed to read stdin: {e}");
        }
        buf
    } else {
        args.join(" ").into_bytes()
    }
}

/// Send one IPC message and exit non-zero on failure.
fn ipc_exec(msg_type: IpcMessageType, data: &[u8], timeout: u32) {
    use crate::error::KeydoError;
    match ipc::ipc_send_recv(msg_type, data, timeout) {
        Ok(_) => {}
        Err(KeydoError::IpcRemoteFailure(msg)) => {
            if msg.is_empty() {
                eprintln!("ERROR: daemon returned failure");
            } else {
                eprintln!("ERROR: {msg}");
            }
            process::exit(1);
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            process::exit(1);
        }
    }
}

// ── Logging output formatting (monitor / listen) ────────────────────────────

/// Minimal JSON string escaping sufficient for the flat, string-only fields
/// used in `--json` monitor/listen output (no full JSON value model needed).
fn json_escape(s: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); }
            c => out.push(c),
        }
    }
    out
}

/// Milliseconds since the Unix epoch (wall-clock, correlatable across processes).
fn epoch_time_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// One `monitor` key event, gathered once per event and handed to whichever
/// formatter (`format_monitor_plain`/`format_monitor_json`) the `--json` flag selects.
struct MonitorEvent<'a> {
    ts_ms: u128,
    delta_ms: Option<i64>,
    device_name: &'a str,
    device_id: &'a str,
    keycode: u8,
    key: &'a str,
    dir: &'a str,
}

fn format_monitor_plain(ev: &MonitorEvent) -> String {
    let MonitorEvent { ts_ms, delta_ms, device_name, device_id, key, dir, .. } = ev;
    match delta_ms {
        Some(d) => format!("{ts_ms}\t+{d} ms\t{device_name}\t{device_id}\t{key} {dir}"),
        None => format!("{ts_ms}\t{device_name}\t{device_id}\t{key} {dir}"),
    }
}

fn format_monitor_json(seq: u64, ev: &MonitorEvent) -> String {
    let MonitorEvent { ts_ms, delta_ms, device_name, device_id, keycode, key, dir } = ev;
    let delta_field = delta_ms.map_or("null".to_string(), |d| d.to_string());
    format!(
        r#"{{"seq":{seq},"ts_ms":{ts_ms},"delta_ms":{delta_field},"device_name":"{}","device_id":"{}","keycode":{keycode},"key":"{}","event":"key","direction":"{dir}"}}"#,
        json_escape(device_name),
        json_escape(device_id),
        json_escape(key),
    )
}

/// Parses one `"<epoch_ms>\t<sigil><name>"` line from the daemon's `LayerListen`
/// wire format and renders it as either plain text or a JSON line. Returns
/// `None` for malformed input (the line is dropped rather than crashing the
/// client on a corrupt/partial line).
fn format_listen_line(wire_line: &str, json: bool, seq: u64) -> Option<String> {
    let (ts, rest) = wire_line.split_once('\t')?;
    let mut chars = rest.chars();
    let sigil = chars.next()?;
    let name = chars.as_str();
    if name.is_empty() {
        return None;
    }
    let (event, action) = match sigil {
        '/' => ("layout", "switch"),
        '+' => ("layer", "activate"),
        '-' => ("layer", "deactivate"),
        _ => return None,
    };
    Some(if json {
        format!(
            r#"{{"seq":{seq},"ts_ms":{ts},"event":"{event}","layer":"{}","action":"{action}"}}"#,
            json_escape(name)
        )
    } else {
        format!("{ts}\t{sigil}{name}")
    })
}

// ── main ───────────────────────────────────────────────────────────────────

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        // ── Daemon (default / explicit) ────────────────────────────────────
        None | Some(Commands::Daemon { .. }) => {
            let single_config = if let Some(Commands::Daemon { config }) = cli.command {
                config
            } else {
                None
            };

            let mut daemon = Daemon::new().unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });

            if let Some(path) = single_config {
                daemon.load_config(&path).unwrap_or_else(|e| {
                    eprintln!("ERROR: {e}");
                    process::exit(1);
                });
            } else {
                let dir = get_config_dir();
                let n = daemon.load_configs_from_dir(&dir);
                if n == 0 {
                    eprintln!("WARNING: no .conf files found in {dir}");
                }
            }

            eprintln!("Starting keyd daemon...");
            daemon.run().unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });
        }

        // ── list-keys ──────────────────────────────────────────────────────
        Some(Commands::ListKeys) => {
            for ent in &KEYCODE_TABLE {
                if let Some(name) = ent.name { println!("{name}"); }
                if let Some(alt) = ent.alt_name.filter(|s| !s.is_empty()) {
                    println!("{alt}");
                }
                if let Some(sh) = ent.shifted_name { println!("{sh}"); }
            }
        }

        // ── monitor ────────────────────────────────────────────────────────
        Some(Commands::Monitor { timestamp, json }) => {
            let mut devices = Device::scan();
            if devices.is_empty() {
                eprintln!("No input devices found (try running as root).");
                process::exit(1);
            }
            let start = std::time::Instant::now();
            let mut last_ms: i64 = 0;
            let mut seq: u64 = 0;

            loop {
                for dev in &mut devices {
                    while let Some(ev) = dev.read_event() {
                        if ev.event_type == DeviceEventType::Key {
                            let now = start.elapsed().as_millis() as i64;
                            let name = KEYCODE_TABLE[ev.code as usize].name.unwrap_or("UNKNOWN");
                            let dir = if ev.pressed != 0 { "down" } else { "up" };
                            let delta_ms = if timestamp && last_ms != 0 { Some(now - last_ms) } else { None };
                            let mon_ev = MonitorEvent {
                                ts_ms: epoch_time_ms(),
                                delta_ms,
                                device_name: &dev.name,
                                device_id: &dev.id,
                                keycode: ev.code,
                                key: name,
                                dir,
                            };

                            let line = if json {
                                format_monitor_json(seq, &mon_ev)
                            } else {
                                format_monitor_plain(&mon_ev)
                            };
                            println!("{line}");

                            last_ms = now;
                            seq += 1;
                            io::stdout().flush().ok();
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        // ── check ──────────────────────────────────────────────────────────
        Some(Commands::Check { files }) => {
            let paths: Vec<String> = if files.is_empty() {
                let dir = get_config_dir();
                let mut v: Vec<_> = std::fs::read_dir(&dir)
                    .ok().into_iter().flatten().flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "conf"))
                    .filter_map(|p| p.to_str().map(str::to_owned))
                    .collect();
                v.sort();
                v
            } else {
                files
            };

            let mut all_ok = true;
            for path in &paths {
                eprintln!("Parsing {path}");
                if let Err(e) = crate::config_impl::config_parse(path) {
                    eprintln!("  FAILED: {e}");
                    all_ok = false;
                }
            }

            if all_ok {
                eprintln!("No errors found.");
            }
            process::exit(i32::from(!all_ok));
        }

        // ── reload ─────────────────────────────────────────────────────────
        Some(Commands::Reload) => {
            ipc_exec(IpcMessageType::Reload, &[], 0);
            println!("Success");
        }

        // ── bind ───────────────────────────────────────────────────────────
        Some(Commands::Bind { bindings }) => {
            if bindings.is_empty() {
                eprintln!("Usage: keyd bind <binding> [<binding> ...]");
                process::exit(1);
            }
            for binding in &bindings {
                ipc_exec(IpcMessageType::Bind, binding.as_bytes(), 0);
            }
            println!("Success");
        }

        // ── do ─────────────────────────────────────────────────────────────
        Some(Commands::DoMacro { timeout, expr }) => {
            let payload = read_payload(&expr);
            // Strip trailing newlines (matches C behaviour).
            let payload = payload.iter().rposition(|&b| b != b'\n')
                .map_or(payload.as_slice(), |i| &payload[..=i]);
            ipc_exec(IpcMessageType::Macro, payload, timeout.unwrap_or(0));
        }

        // ── input ──────────────────────────────────────────────────────────
        Some(Commands::Input { timeout, text }) => {
            let payload = read_payload(&text);
            ipc_exec(IpcMessageType::Input, &payload, timeout.unwrap_or(0));
        }

        // ── install ────────────────────────────────────────────────────────
        Some(Commands::Install { init }) => {
            install::install(init).unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });
        }

        // ── uninstall ──────────────────────────────────────────────────────
        Some(Commands::Uninstall { init }) => {
            install::uninstall(init).unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });
        }

        // ── listen ─────────────────────────────────────────────────────────
        Some(Commands::Listen { json }) => {
            let mut stream = ipc::ipc_connect().unwrap_or_else(|e| {
                eprintln!("ERROR: Failed to connect to daemon: {e}");
                process::exit(1);
            });

            let msg = IpcMessage::new(IpcMessageType::LayerListen, 0);
            msg.write_to(&mut stream).unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });

            let mut reader = io::BufReader::new(stream);
            let mut seq: u64 = 0;
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end_matches('\n');
                        if trimmed.is_empty() { continue; }
                        if let Some(out) = format_listen_line(trimmed, json, seq) {
                            println!("{out}");
                            io::stdout().flush().ok();
                            seq += 1;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod monitor_listen_format_tests {
    use super::*;

    #[test]
    fn json_escape_escapes_quotes_backslash_and_whitespace_controls() {
        assert_eq!(json_escape("a\"b\\c\nd\re\tf"), r#"a\"b\\c\nd\re\tf"#);
    }

    #[test]
    fn json_escape_escapes_other_control_chars_as_unicode() {
        assert_eq!(json_escape("a\u{1}b"), "a\\u0001b");
    }

    #[test]
    fn json_escape_passes_through_plain_unicode() {
        assert_eq!(json_escape("café ⌘"), "café ⌘");
    }

    fn base_monitor_event() -> MonitorEvent<'static> {
        MonitorEvent {
            ts_ms: 1000,
            delta_ms: None,
            device_name: "Keyboard",
            device_id: "dev0",
            keycode: 30,
            key: "a",
            dir: "down",
        }
    }

    #[test]
    fn format_monitor_plain_without_delta() {
        let line = format_monitor_plain(&base_monitor_event());
        assert_eq!(line, "1000\tKeyboard\tdev0\ta down");
    }

    #[test]
    fn format_monitor_plain_with_delta() {
        let ev = MonitorEvent { delta_ms: Some(12), ..base_monitor_event() };
        let line = format_monitor_plain(&ev);
        assert_eq!(line, "1000\t+12 ms\tKeyboard\tdev0\ta down");
    }

    #[test]
    fn format_monitor_json_without_delta() {
        let line = format_monitor_json(0, &base_monitor_event());
        assert_eq!(
            line,
            r#"{"seq":0,"ts_ms":1000,"delta_ms":null,"device_name":"Keyboard","device_id":"dev0","keycode":30,"key":"a","event":"key","direction":"down"}"#
        );
    }

    #[test]
    fn format_monitor_json_with_delta_and_escaped_device_name() {
        let ev = MonitorEvent {
            delta_ms: Some(12),
            device_name: "My \"KB\"",
            dir: "up",
            ..base_monitor_event()
        };
        let line = format_monitor_json(1, &ev);
        assert_eq!(
            line,
            r#"{"seq":1,"ts_ms":1000,"delta_ms":12,"device_name":"My \"KB\"","device_id":"dev0","keycode":30,"key":"a","event":"key","direction":"up"}"#
        );
    }

    #[test]
    fn format_listen_line_layer_activate() {
        assert_eq!(format_listen_line("1000\t+nav", false, 0).unwrap(), "1000\t+nav");
        assert_eq!(
            format_listen_line("1000\t+nav", true, 0).unwrap(),
            r#"{"seq":0,"ts_ms":1000,"event":"layer","layer":"nav","action":"activate"}"#
        );
    }

    #[test]
    fn format_listen_line_layer_deactivate() {
        assert_eq!(
            format_listen_line("1000\t-nav", true, 2).unwrap(),
            r#"{"seq":2,"ts_ms":1000,"event":"layer","layer":"nav","action":"deactivate"}"#
        );
    }

    #[test]
    fn format_listen_line_layout_switch() {
        assert_eq!(
            format_listen_line("1000\t/dvorak", true, 5).unwrap(),
            r#"{"seq":5,"ts_ms":1000,"event":"layout","layer":"dvorak","action":"switch"}"#
        );
    }

    #[test]
    fn format_listen_line_rejects_malformed_input() {
        assert!(format_listen_line("garbage-no-tab", false, 0).is_none());
        assert!(format_listen_line("123\t", false, 0).is_none());
        assert!(format_listen_line("123\t?nav", false, 0).is_none());
    }

    #[test]
    fn format_listen_line_round_trips_daemon_wire_format() {
        // Mirrors the exact `format!` shape daemon.rs's on_layer_change builds.
        let ts: u128 = 1_700_000_000_000;
        let wire = format!("{ts}\t{}{}\n", '+', "control");
        let trimmed = wire.trim_end_matches('\n');
        assert_eq!(
            format_listen_line(trimmed, true, 0).unwrap(),
            r#"{"seq":0,"ts_ms":1700000000000,"event":"layer","layer":"control","action":"activate"}"#
        );
    }
}
