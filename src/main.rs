mod keys;
mod config;
mod ini;
mod macro_parser;
mod config_parser;
mod device;
mod keyboard;
mod vkbd;
mod unicode;
mod ipc;
mod tests;

use clap::{Parser, Subcommand};
use crate::config_parser::ConfigParser;
use crate::device::{scan_devices, EvdevDevice};
use crate::keyboard::{Keyboard, OutputEvent};
use crate::vkbd::Vkbd;
use crate::ipc::{create_server, send_message, IpcMessage, IpcMessageType};
use nix::poll::{poll, PollFd, PollFlags};
#[cfg(target_os = "linux")]
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, BorrowedFd};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about = "keydo — keyboard remapper daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Monitor and print key events in real time.
    Monitor {
        #[arg(short, long, help = "Show time delta between events")]
        time: bool,
    },
    /// Reload the daemon configuration.
    Reload,
    /// Validate config files in /etc/keyd/ without running the daemon.
    Check,
    /// List all known key names.
    #[command(name = "list-keys")]
    ListKeys,
    /// Dynamically bind a key expression (e.g. "a = b").
    Bind {
        expression: String,
    },
    /// Send a macro string to be executed by the daemon.
    Input {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Alias for input.
    Do {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Stream layer change events from the daemon.
    Listen,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Monitor { time }) => run_monitor(time),
        Some(Commands::Reload) => {
            send_message(&IpcMessage {
                message_type: IpcMessageType::Reload,
                timeout: 0,
                data: String::new(),
            })?;
            println!("Reload signal sent.");
            Ok(())
        }
        Some(Commands::Check) => run_check(),
        Some(Commands::ListKeys) => {
            run_list_keys();
            Ok(())
        }
        Some(Commands::Bind { expression }) => {
            send_message(&IpcMessage {
                message_type: IpcMessageType::Bind,
                timeout: 0,
                data: expression,
            })?;
            Ok(())
        }
        Some(Commands::Input { args }) | Some(Commands::Do { args }) => {
            send_message(&IpcMessage {
                message_type: IpcMessageType::Input,
                timeout: 0,
                data: args.join(" "),
            })?;
            Ok(())
        }
        Some(Commands::Listen) => run_listen(),
        None => run_daemon(),
    }
}

fn run_monitor(show_time: bool) -> anyhow::Result<()> {
    let mut devices = scan_devices();
    let borrowed_fds: Vec<BorrowedFd> =
        devices.iter().map(|d| unsafe { BorrowedFd::borrow_raw(d.device.as_raw_fd()) }).collect();
    let mut poll_fds: Vec<PollFd> =
        borrowed_fds.iter().map(|fd| PollFd::new(fd, PollFlags::POLLIN)).collect();

    println!("Monitoring {} devices...", devices.len());
    let mut last_time = std::time::Instant::now();

    loop {
        poll(&mut poll_fds, -1)?;
        for i in 0..poll_fds.len() {
            if let Some(revents) = poll_fds[i].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    let events: Vec<_> = devices[i].device.fetch_events()?.collect();
                    for ev in events {
                        if ev.event_type() == evdev::EventType::KEY {
                            if show_time {
                                let now = std::time::Instant::now();
                                print!("+{} ms\t", now.duration_since(last_time).as_millis());
                                last_time = now;
                            }
                            println!(
                                "{}\t{}\t{}\t{}",
                                devices[i].device.name().unwrap_or("Unknown"),
                                devices[i].id,
                                crate::keys::get_key_name(ev.code()),
                                if ev.value() == 1 { "down" } else { "up" }
                            );
                        }
                    }
                }
            }
        }
    }
}

fn run_check() -> anyhow::Result<()> {
    let config_dir = Path::new("/etc/keyd");
    if !config_dir.exists() {
        anyhow::bail!("/etc/keyd does not exist");
    }
    let mut ok = 0;
    let mut fail = 0;
    for entry in std::fs::read_dir(config_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("conf") {
            let mut parser = ConfigParser::new();
            match parser.parse(&path) {
                Ok(_) => {
                    println!("OK   {}", path.display());
                    ok += 1;
                }
                Err(e) => {
                    eprintln!("FAIL {}: {}", path.display(), e);
                    fail += 1;
                }
            }
        }
    }
    println!("{} ok, {} failed", ok, fail);
    if fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn run_list_keys() {
    for (i, entry) in crate::keys::KEYCODE_TABLE.iter().enumerate() {
        if let Some(ent) = entry {
            print!("{}", ent.name);
            if let Some(alt) = ent.alt_name {
                print!(" ({})", alt);
            }
            if let Some(shifted) = ent.shifted_name {
                print!(" [{}]", shifted);
            }
            println!(" = {}", i);
        }
    }
}

fn run_listen() -> anyhow::Result<()> {
    let mut stream = ipc::connect()?;
    let msg = IpcMessage {
        message_type: IpcMessageType::LayerListen,
        timeout: 0,
        data: String::new(),
    };
    let buf = serde_json::to_vec(&msg)?;
    stream.write_all(&buf)?;
    // Stream responses until EOF.
    let mut line = String::new();
    loop {
        let mut byte = [0u8; 1];
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                if byte[0] == b'\n' {
                    print!("{}", line);
                    println!();
                    line.clear();
                } else {
                    line.push(byte[0] as char);
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

struct Daemon {
    keyboard: Keyboard,
    vkbd: Vkbd,
    grabbed_devices: Vec<EvdevDevice>,
    listeners: Vec<std::os::unix::net::UnixStream>,
}

impl Daemon {
    fn new(config: crate::config::Config) -> anyhow::Result<Self> {
        let mut grabbed_devices = Vec::new();
        for mut dev in scan_devices() {
            if dev.should_grab(&config.ids, config.wildcard) {
                match dev.grab() {
                    Ok(_) => grabbed_devices.push(dev),
                    Err(e) => eprintln!("Failed to grab {}: {}", dev.path, e),
                }
            }
        }
        let keyboard = Keyboard::new(config);
        let vkbd = Vkbd::new("keydo virtual keyboard")?;
        Ok(Daemon { keyboard, vkbd, grabbed_devices, listeners: Vec::new() })
    }

    fn reload(&mut self) -> anyhow::Result<()> {
        for dev in &mut self.grabbed_devices {
            let _ = dev.ungrab();
        }
        self.grabbed_devices.clear();

        let config_dir = Path::new("/etc/keyd");
        let mut parser = ConfigParser::new();
        let config = if let Ok(c) = parser.parse(&config_dir.join("default.conf")) {
            c
        } else {
            let mut c = parser.parse(Path::new("/dev/null")).unwrap();
            c.wildcard = true;
            c
        };

        for mut dev in scan_devices() {
            if dev.should_grab(&config.ids, config.wildcard) {
                match dev.grab() {
                    Ok(_) => self.grabbed_devices.push(dev),
                    Err(e) => eprintln!("Failed to grab {}: {}", dev.path, e),
                }
            }
        }
        self.keyboard = Keyboard::new(config);
        Ok(())
    }

    fn try_add_device(&mut self, path: &str) {
        if let Ok(mut dev) = EvdevDevice::open(Path::new(path)) {
            if dev.should_grab(&self.keyboard.config.ids, self.keyboard.config.wildcard) {
                match dev.grab() {
                    Ok(_) => self.grabbed_devices.push(dev),
                    Err(e) => eprintln!("Failed to grab {}: {}", path, e),
                }
            }
        }
    }

    fn remove_device_by_fd(&mut self, fd: i32) {
        if let Some(pos) = self.grabbed_devices.iter().position(|d| d.device.as_raw_fd() == fd) {
            let _ = self.grabbed_devices[pos].ungrab();
            self.grabbed_devices.remove(pos);
        }
    }

    fn handle_ipc(&mut self, ipc_listener: &std::os::unix::net::UnixListener) {
        if let Ok((mut stream, _)) = ipc_listener.accept() {
            let mut buf = Vec::new();
            if stream.read_to_end(&mut buf).is_ok() {
                if let Ok(msg) = serde_json::from_slice::<IpcMessage>(&buf) {
                    match msg.message_type {
                        IpcMessageType::Reload => {
                            if let Err(e) = self.reload() {
                                eprintln!("Reload failed: {}", e);
                            }
                        }
                        IpcMessageType::Bind => {
                            // Inject a binding into the main layer at runtime.
                            // Parse "key = action" and add to layer 0.
                            if let Some((key_str, act_str)) = msg.data.split_once('=') {
                                let key_str = key_str.trim();
                                let act_str = act_str.trim();
                                if let Some(code) = crate::keys::lookup_keycode(key_str) {
                                    if let Some(action) =
                                        self.parse_simple_action(act_str)
                                    {
                                        self.keyboard.config.layers[0]
                                            .keymap
                                            .insert(code, action);
                                    }
                                }
                            }
                        }
                        IpcMessageType::Input | IpcMessageType::Macro => {
                            if let Ok(m) = crate::macro_parser::parse_macro(&msg.data) {
                                let mut events: Vec<OutputEvent> = Vec::new();
                                self.keyboard.execute_macro_pub(&m, &mut events);
                                for ev in &events {
                                    let _ = self.vkbd.send_event(ev);
                                }
                            }
                        }
                        IpcMessageType::LayerListen => {
                            self.listeners.push(stream);
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn notify_listeners(&mut self, layer_name: &str, active: bool) {
        let line = format!("{} {}\n", layer_name, if active { "on" } else { "off" });
        self.listeners.retain_mut(|s| s.write_all(line.as_bytes()).is_ok());
    }

    fn parse_simple_action(&self, s: &str) -> Option<crate::config::Action> {
        if let Some(code) = crate::keys::lookup_keycode(s) {
            Some(crate::config::Action::KeySequence(code, 0))
        } else {
            None
        }
    }

    fn process_key_event(&mut self, code: u16, pressed: bool) -> anyhow::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.keyboard.set_time(now);
        let output = self.keyboard.process_event(code, pressed);
        for ev in &output {
            self.vkbd.send_event(ev)?;
        }
        Ok(())
    }
}

fn run_daemon() -> anyhow::Result<()> {
    // Set up signal handling.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&r))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, running.clone())?;

    let config_dir = Path::new("/etc/keyd");
    let mut parser = ConfigParser::new();
    let config = if let Ok(c) = parser.parse(&config_dir.join("default.conf")) {
        c
    } else {
        let mut c = parser.parse(Path::new("/dev/null")).unwrap();
        c.wildcard = true;
        c
    };

    let mut daemon = Daemon::new(config)?;
    let ipc_listener = create_server()?;
    ipc_listener.set_nonblocking(true)?;

    // Set up inotify to watch /dev/input for new devices (Linux only).
    #[cfg(target_os = "linux")]
    let inotify = {
        let ino = Inotify::init(InitFlags::IN_NONBLOCK)?;
        ino.add_watch("/dev/input", AddWatchFlags::IN_CREATE)?;
        ino
    };

    // Panic escape: track Esc + Backspace + Enter.
    let mut panic_keys = [false; 3]; // [ESC, BACKSPACE, ENTER]
    const PANIC_CODES: [u16; 3] = [
        crate::keys::KEYD_ESC,
        crate::keys::KEYD_BACKSPACE,
        crate::keys::KEYD_ENTER,
    ];

    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Build poll fds: devices + ipc listener + inotify (Linux).
        let borrowed_dev_fds: Vec<BorrowedFd> = daemon
            .grabbed_devices
            .iter()
            .map(|d| unsafe { BorrowedFd::borrow_raw(d.device.as_raw_fd()) })
            .collect();
        let ipc_borrowed = unsafe { BorrowedFd::borrow_raw(ipc_listener.as_raw_fd()) };

        let mut poll_fds: Vec<PollFd> = borrowed_dev_fds
            .iter()
            .map(|fd| PollFd::new(fd, PollFlags::POLLIN | PollFlags::POLLERR | PollFlags::POLLHUP))
            .collect();
        let ipc_idx = poll_fds.len();
        poll_fds.push(PollFd::new(&ipc_borrowed, PollFlags::POLLIN));

        #[cfg(target_os = "linux")]
        let inotify_idx = {
            let inotify_borrowed = unsafe { BorrowedFd::borrow_raw(inotify.as_raw_fd()) };
            let idx = poll_fds.len();
            poll_fds.push(PollFd::new(&inotify_borrowed, PollFlags::POLLIN));
            idx
        };
        #[cfg(not(target_os = "linux"))]
        let _inotify_idx: usize = usize::MAX;

        // Compute timeout for pending keyboard timeouts (50ms tick).
        let _ = poll(&mut poll_fds, 50);

        // Keyboard tick for pending timeouts (code=0, pressed=false).
        let _ = daemon.process_key_event(0, false);

        // IPC.
        if let Some(revents) = poll_fds.get(ipc_idx).and_then(|p| p.revents()) {
            if revents.contains(PollFlags::POLLIN) {
                daemon.handle_ipc(&ipc_listener);
            }
        }

        // Inotify — new /dev/input/event* devices (Linux only).
        #[cfg(target_os = "linux")]
        if let Some(revents) = poll_fds.get(inotify_idx).and_then(|p| p.revents()) {
            if revents.contains(PollFlags::POLLIN) {
                if let Ok(events) = inotify.read_events() {
                    for ev in events {
                        if let Some(name) = ev.name {
                            let name_str = name.to_string_lossy();
                            if name_str.starts_with("event") {
                                let path = format!("/dev/input/{}", name_str);
                                // Give the kernel a moment to finish creating the node.
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                daemon.try_add_device(&path);
                            }
                        }
                    }
                }
            }
        }

        // Device events — collect disconnected fds first.
        let mut disconnected_fds: Vec<i32> = Vec::new();
        for i in 0..daemon.grabbed_devices.len() {
            let revents = match poll_fds.get(i).and_then(|p| p.revents()) {
                Some(r) => r,
                None => continue,
            };
            if revents.contains(PollFlags::POLLERR) || revents.contains(PollFlags::POLLHUP) {
                disconnected_fds.push(daemon.grabbed_devices[i].device.as_raw_fd());
                continue;
            }
            if !revents.contains(PollFlags::POLLIN) {
                continue;
            }
            let dev_fd = daemon.grabbed_devices[i].device.as_raw_fd();
            let events: Vec<_> = match daemon.grabbed_devices[i].device.fetch_events() {
                Ok(evs) => evs.collect(),
                Err(_) => {
                    disconnected_fds.push(dev_fd);
                    continue;
                }
            };
            for ev in events {
                if ev.event_type() == evdev::EventType::KEY {
                    let code = ev.code();
                    let pressed = ev.value() != 0;

                    // Check panic escape: Esc + Backspace + Enter held simultaneously.
                    for (j, &pc) in PANIC_CODES.iter().enumerate() {
                        if code == pc {
                            panic_keys[j] = pressed;
                        }
                    }
                    if panic_keys.iter().all(|&v| v) {
                        eprintln!("keydo: panic escape activated, exiting.");
                        running.store(false, Ordering::Relaxed);
                        break;
                    }

                    let _ = daemon.process_key_event(code, pressed);
                }
            }
        }
        for fd in disconnected_fds {
            daemon.remove_device_by_fd(fd);
        }
    }

    // Cleanup.
    for dev in &mut daemon.grabbed_devices {
        let _ = dev.ungrab();
    }
    if Path::new(ipc::SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(ipc::SOCKET_PATH);
    }
    Ok(())
}
