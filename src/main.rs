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
#[cfg(target_os = "macos")]
pub mod macos_input;
#[cfg(test)]
pub mod tests;
#[cfg(test)]
pub mod test_io;

use clap::{Parser, Subcommand};
use crate::daemon::Daemon;
use crate::device::{Device, DeviceEventType};
use crate::ipc::{IpcMessage, IpcMessageType};
use crate::keys::KEYCODE_TABLE;
use std::io::{self, Read, Write};
use std::process;

// ── CLI definition ─────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    version = "2.6.0",
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
        /// Load a single config file instead of scanning default directories (~/.config/keydo/ or /etc/keyd/)
        #[arg(short, long)]
        config: Option<String>,
    },

    /// List all valid key names
    #[command(name = "list-keys")]
    ListKeys,

    /// Print key events in real time (requires root)
    Monitor {
        /// Print time in milliseconds between events
        #[arg(short = 't', long)]
        timestamp: bool,
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
    Listen,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Returns the default configuration directory.
/// Checks ~/.config/keydo/ first, then falls back to /etc/keyd/.
fn get_config_dir() -> String {
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::PathBuf::from(home).join(".config/keydo");
        if path.is_dir() {
            return path.to_string_lossy().into_owned();
        }
    }
    "/etc/keyd/".to_string()
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
        Some(Commands::Monitor { timestamp }) => {
            let mut devices = Device::scan();
            if devices.is_empty() {
                eprintln!("No input devices found (try running as root).");
                process::exit(1);
            }
            let start = std::time::Instant::now();
            let mut last_ms: i64 = 0;

            loop {
                for dev in &mut devices {
                    while let Some(ev) = dev.read_event() {
                        if ev.event_type == DeviceEventType::Key {
                            let now = start.elapsed().as_millis() as i64;
                            let name = KEYCODE_TABLE[ev.code as usize].name.unwrap_or("UNKNOWN");

                            if timestamp && last_ms != 0 {
                                print!("+{} ms\t", now - last_ms);
                            }
                            let dir = if ev.pressed != 0 { "down" } else { "up" };
                            println!("{}\t{}\t{name} {dir}", dev.name, dev.id);

                            last_ms = now;
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

        // ── listen ─────────────────────────────────────────────────────────
        Some(Commands::Listen) => {
            let mut stream = ipc::ipc_connect().unwrap_or_else(|e| {
                eprintln!("ERROR: Failed to connect to daemon: {e}");
                process::exit(1);
            });

            let msg = IpcMessage::new(IpcMessageType::LayerListen, 0);
            msg.write_to(&mut stream).unwrap_or_else(|e| {
                eprintln!("ERROR: {e}");
                process::exit(1);
            });

            let mut buf = [0u8; 512];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if io::stdout().write_all(&buf[..n]).is_err() { break; }
                        io::stdout().flush().ok();
                    }
                }
            }
        }
    }
}
