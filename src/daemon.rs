//! Main daemon loop — scans input devices, dispatches key events through keyboard state machines,
//! and services IPC connections (bind, macro, reload, layer-listen).

use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixListener;

use crate::config::*;
use crate::config_impl::{config_add_entry, config_check_match, config_parse};
use crate::ipc::{IpcMessage, IpcMessageType};
use crate::keyboard_types::*;
use crate::keys::{KEYCODE_TABLE, KEYD_BACKSPACE, KEYD_ENTER, KEYD_ESC, KEYD_LEFTSHIFT, KEYD_SPACE, KEYD_TAB};

// ── Panic sequence detector ────────────────────────────────────────────────

struct PanicState { enter: bool, backspace: bool, escape: bool }

impl PanicState {
    fn new() -> Self { Self { enter: false, backspace: false, escape: false } }

    fn check(&mut self, code: u8, pressed: bool) {
        match code {
            c if c == KEYD_ENTER     => self.enter     = pressed,
            c if c == KEYD_BACKSPACE => self.backspace  = pressed,
            c if c == KEYD_ESC       => self.escape     = pressed,
            _ => {}
        }
        if self.enter && self.backspace && self.escape {
            eprintln!("Panic sequence detected — exiting.");
            std::process::exit(1);
        }
    }
}
use crate::device::*;
use crate::vkbd::Vkbd;

/// SCHED_FIFO priority for the daemon loop — high enough to preempt most user tasks.
#[cfg(target_os = "linux")]
const REALTIME_SCHED_PRIORITY: libc::c_int = 49;

fn current_time_ms() -> i64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as i64
}

fn caps_to_id_flags(caps: u8) -> u8 {
    let mut flags: u8 = 0;
    if caps & CAP_KEY != 0       { flags |= ID_KEY; }
    if caps & CAP_KEYBOARD != 0  { flags |= ID_KEYBOARD; }
    if caps & CAP_MOUSE_ABS != 0 { flags |= ID_TRACKPAD; }
    if caps & CAP_MOUSE != 0     { flags |= ID_MOUSE; }
    flags
}

fn manage_device(keyboards: &[Keyboard], device: &mut Device) -> Option<usize> {
    if device.is_virtual { return None; }
    let flags = caps_to_id_flags(device.capabilities);
    let mut best_idx: Option<usize> = None;
    let mut best_rank: i32 = 0;
    for (i, kbd) in keyboards.iter().enumerate() {
        let r = config_check_match(&kbd.config, &device.id, flags);
        if r > best_rank { best_rank = r; best_idx = Some(i); }
    }
    if best_rank == 1 && !((flags & ID_KEYBOARD != 0) && (flags & ID_TRACKPAD == 0)) {
        best_idx = None;
    }
    if best_idx.is_some() {
        if device.grab().is_err() {
            eprintln!("WARNING: Failed to grab {}", device.path);
            return None;
        }
        eprintln!("DEVICE: match    {}  ({})", device.id, device.name);
    } else {
        let _ = device.ungrab();
        eprintln!("DEVICE: ignoring {}  ({})", device.id, device.name);
    }
    best_idx
}

// ── LED helpers (Linux only) ───────────────────────────────────────────────

/// Write one EV_LED event directly to an fd (used for layer_indicator and LED forwarding).
#[cfg(target_os = "linux")]
fn write_led_fd(fd: RawFd, led: u8, state: bool) {
    #[repr(C)]
    struct Ev { time: libc::timeval, type_: u16, code: u16, value: i32 }
    // SAFETY: libc::timeval contains only integer fields and is valid when zero-initialized.
    let ev = Ev { time: unsafe { std::mem::zeroed() }, type_: 0x11, code: led as u16, value: state as i32 };
    // SAFETY: fd is a valid open device fd; pointer and size exactly match the Ev layout.
    unsafe { libc::write(fd, &ev as *const _ as *const libc::c_void, std::mem::size_of::<Ev>()); }
}

// ── Input text (for IPC_INPUT) ─────────────────────────────────────────────

fn input_text(vkbd: &Vkbd, text: &str, delay_us: u32) {
    for c in text.chars() {
        let s = c.to_string();
        let mut found = false;

        for (i, ent) in KEYCODE_TABLE.iter().enumerate() {
            if ent.name.is_some_and(|n| n == s) {
                vkbd.send_key(i as u8, 1);
                vkbd.send_key(i as u8, 0);
                found = true;
                break;
            }
            if ent.shifted_name.is_some_and(|n| n == s) {
                vkbd.send_key(KEYD_LEFTSHIFT, 1);
                vkbd.send_key(i as u8, 1);
                vkbd.send_key(i as u8, 0);
                vkbd.send_key(KEYD_LEFTSHIFT, 0);
                found = true;
                break;
            }
        }

        if !found {
            match c {
                ' '  => { vkbd.send_key(KEYD_SPACE, 1); vkbd.send_key(KEYD_SPACE, 0); found = true; }
                '\n' => { vkbd.send_key(KEYD_ENTER, 1); vkbd.send_key(KEYD_ENTER, 0); found = true; }
                '\t' => { vkbd.send_key(KEYD_TAB, 1);   vkbd.send_key(KEYD_TAB, 0);   found = true; }
                _ => {}
            }
        }

        if let Some(idx) = (!found).then(|| crate::unicode::unicode_lookup_index(c as u32)).flatten() {
            let codes = crate::unicode::unicode_get_sequence(idx);
            for &code in &codes {
                if code != 0 { vkbd.send_key(code, 1); vkbd.send_key(code, 0); }
            }
        }

        if delay_us > 0 {
            std::thread::sleep(std::time::Duration::from_micros(delay_us as u64));
        }
    }
}

// ── Output state ───────────────────────────────────────────────────────────

struct OutputState {
    vkbd:      Vkbd,
    keystate:  [u8; 256],
    listeners: Vec<RawFd>,
}

// ── Daemon ─────────────────────────────────────────────────────────────────

pub struct Daemon {
    output:     OutputState,
    pub keyboards:   Vec<Keyboard>,
    pub devices:     Vec<Device>,
    pub device_kbd:  Vec<Option<usize>>,
    ipc_server: Option<(UnixListener, std::fs::File)>,
    config_dir: String,
}

impl Daemon {
    /// Create a new daemon, initialising the virtual keyboard and IPC server socket.
    pub fn new() -> Result<Self, String> {
        let vkbd = Vkbd::init("keyd virtual keyboard")?;
        let ipc_server = crate::ipc::ipc_create_server().ok();
        if ipc_server.is_none() {
            eprintln!("WARNING: IPC server unavailable (another daemon running, or permissions issue)");
        }
        Ok(Daemon {
            output: OutputState { vkbd, keystate: [0; 256], listeners: Vec::new() },
            keyboards:  Vec::new(),
            devices:    Vec::new(),
            device_kbd: Vec::new(),
            ipc_server,
            config_dir: "/etc/keyd".to_string(),
        })
    }

    /// Parse a single `.conf` file and register the resulting keyboard config.
    pub fn load_config(&mut self, path: &str) -> Result<(), String> {
        let cfg = config_parse(path).map_err(|e| e.to_string())?;
        self.keyboards.push(Keyboard::new(cfg));
        Ok(())
    }

    /// Scan `dir` for `.conf` files and load each one. Returns the count loaded.
    pub fn load_configs_from_dir(&mut self, dir: &str) -> usize {
        self.config_dir = dir.to_string();
        let mut n = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<_> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "conf"))
                .collect();
            paths.sort();
            for path in paths {
                if let Some(p) = path.to_str() {
                    match config_parse(p) {
                        Ok(cfg) => { self.keyboards.push(Keyboard::new(cfg)); n += 1; }
                        Err(e)  => eprintln!("WARNING: {p}: {e}"),
                    }
                }
            }
        }
        n
    }

    fn dispatch_kbd(&mut self, kbd_idx: usize, events: &[KeyEvent]) -> i64 {
        // Collect raw fds of grabbed, non-virtual physical devices for this keyboard
        // (used by DaemonOutput::on_layer_change for the layer_indicator LED).
        let grabbed_fds: Vec<RawFd> = self.devices.iter()
            .zip(self.device_kbd.iter())
            .filter(|(dev, ki)| **ki == Some(kbd_idx) && dev.grabbed && !dev.is_virtual)
            .map(|(dev, _)| dev.fd)
            .collect();

        let (keyboards, out) = (&mut self.keyboards, &mut self.output);
        let mut adapter = DaemonOutput {
            vkbd:       &out.vkbd,
            keystate:   &mut out.keystate,
            listeners:  &mut out.listeners,
            device_fds: &grabbed_fds,
        };
        keyboards[kbd_idx].kbd_process_events(&mut adapter, events)
    }

    fn reload(&mut self) {
        self.keyboards.clear();
        if let Ok(entries) = std::fs::read_dir(&self.config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_conf = path.extension().is_some_and(|e| e == "conf");
                if let Some(p) = path.to_str().filter(|_| is_conf) {
                    match config_parse(p) {
                        Ok(cfg) => self.keyboards.push(Keyboard::new(cfg)),
                        Err(e)  => eprintln!("WARNING: {p}: {e}"),
                    }
                }
            }
        }
        for i in 0..self.devices.len() {
            self.device_kbd[i] = manage_device(&self.keyboards, &mut self.devices[i]);
        }
        // Release all held virtual keys.
        for code in 0u8..=255 {
            if self.output.keystate[code as usize] != 0 {
                self.output.vkbd.send_key(code, 0);
                self.output.keystate[code as usize] = 0;
            }
        }
    }

    fn send_success(conn: &mut std::os::unix::net::UnixStream) {
        let resp = IpcMessage::new(IpcMessageType::Success, 0);
        let _ = resp.write_to(conn);
    }

    fn send_fail(conn: &mut std::os::unix::net::UnixStream, msg: &str) {
        let mut resp = IpcMessage::new(IpcMessageType::Fail, 0);
        resp.set_data(msg.as_bytes());
        let _ = resp.write_to(conn);
    }

    fn handle_client(&mut self, mut conn: std::os::unix::net::UnixStream) {
        let Ok(msg) = IpcMessage::read_from(&mut conn) else { return };

        match IpcMessageType::try_from(msg.msg_type) {
            Ok(IpcMessageType::Reload) => {
                self.reload();
                Self::send_success(&mut conn);
            }

            Ok(IpcMessageType::Bind) => {
                let expr = msg.data_str().to_string();
                let mut ok = false;
                for kbd in &mut self.keyboards {
                    if config_add_entry(&mut kbd.config, &expr).is_ok() {
                        ok = true;
                    }
                }
                if ok { Self::send_success(&mut conn); }
                else  { Self::send_fail(&mut conn, "bind failed"); }
            }

            Ok(IpcMessageType::Macro) => {
                let src = msg.data_str().trim_end_matches('\n').to_string();
                let seq_us = msg.timeout as u64;
                match crate::config_parse::config_parse_macro_expression(&src) {
                    Ok(mac) => {
                        let mut out = VkbdOutput { vkbd: &self.output.vkbd };
                        Keyboard::macro_execute_blocking(&mut out, &mac, seq_us);
                        Self::send_success(&mut conn);
                    }
                    Err(e) => Self::send_fail(&mut conn, &e.to_string()),
                }
            }

            Ok(IpcMessageType::Input) => {
                let text   = msg.data_str().to_string();
                let delay  = msg.timeout;
                input_text(&self.output.vkbd, &text, delay);
                Self::send_success(&mut conn);
            }

            Ok(IpcMessageType::LayerListen) => {
                use std::os::unix::io::IntoRawFd;
                // Transfer ownership; drop will NOT close (fd is in listeners).
                self.output.listeners.push(conn.into_raw_fd());
                // Don't send a response — the connection stays open for streaming.
            }

            _ => Self::send_fail(&mut conn, "unknown command"),
        }
    }

    /// Enter the main event loop. Blocks until the process is killed or an unrecoverable error occurs.
    ///
    /// On Linux, sets real-time scheduling (`SCHED_FIFO`) and locks process memory
    /// (`mlockall`) before entering the loop to reduce input latency.
    pub fn run(&mut self) -> Result<(), String> {
        self.devices = Device::scan();
        self.device_kbd = self.devices.iter_mut()
            .map(|dev| manage_device(&self.keyboards, dev))
            .collect();

        if self.devices.is_empty() {
            eprintln!("WARNING: No input devices found at startup (waiting for hot-plug).");
        }

        // Linux: real-time scheduling and memory locking for low input latency.
        #[cfg(target_os = "linux")]
        {
            let sp = libc::sched_param { sched_priority: REALTIME_SCHED_PRIORITY };
            // SAFETY: pid 0 means current process; SCHED_FIFO with priority 49 is a valid real-time policy.
            if unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &sp) } != 0 {
                eprintln!("WARNING: sched_setscheduler: {}", std::io::Error::last_os_error());
            }
            // SAFETY: MCL_CURRENT | MCL_FUTURE are valid mlockall flags with no memory-safety requirements.
            if unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) } != 0 {
                eprintln!("WARNING: mlockall: {}", std::io::Error::last_os_error());
            }
        }

        // Linux-only: inotify watch for hot-plugged input devices.
        #[cfg(target_os = "linux")]
        let mut devmon: Option<inotify::Inotify> = {
            use inotify::{Inotify, WatchMask};
            Inotify::init().ok().and_then(|ino| {
                ino.watches().add("/dev/input/", WatchMask::CREATE).ok()?;
                Some(ino)
            })
        };

        let mut panic_state = PanicState::new();
        let mut timeout_ms: i64 = -1;

        loop {
            let dev_count = self.devices.len();
            let ipc_fd = self.ipc_server.as_ref().map(|(l, _)| l.as_raw_fd());

            // ── Build pollfd array ────────────────────────────────────────
            let mut pfds: Vec<libc::pollfd> = self.devices.iter()
                .map(|d| libc::pollfd { fd: d.fd, events: libc::POLLIN | libc::POLLERR, revents: 0 })
                .collect();

            let ipc_pfd_idx = ipc_fd.map(|fd| {
                let idx = pfds.len();
                pfds.push(libc::pollfd { fd, events: libc::POLLIN, revents: 0 });
                idx
            });

            #[cfg(target_os = "linux")]
            let devmon_pfd_idx = {
                use std::os::unix::io::AsRawFd;
                devmon.as_ref().map(|ino| {
                    let idx = pfds.len();
                    pfds.push(libc::pollfd { fd: ino.as_raw_fd(), events: libc::POLLIN, revents: 0 });
                    idx
                })
            };

            // Virtual keyboard fd: receives EV_LED feedback from the OS.
            #[cfg(target_os = "linux")]
            let vkbd_pfd_idx = {
                let idx = pfds.len();
                pfds.push(libc::pollfd { fd: self.output.vkbd.keyboard_fd(), events: libc::POLLIN, revents: 0 });
                idx
            };

            // ── Poll ──────────────────────────────────────────────────────
            let poll_timeout = if timeout_ms < 0 { -1i32 }
                               else { timeout_ms.min(i32::MAX as i64) as i32 };
            let poll_start = current_time_ms();
            // SAFETY: pfds is a valid, contiguous slice; nfds is the exact slice length; timeout is valid.
            let poll_ret = unsafe { libc::poll(pfds.as_mut_ptr(), pfds.len() as libc::nfds_t, poll_timeout) };
            let now     = current_time_ms();
            let elapsed = now - poll_start;

            if poll_ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    eprintln!("ERROR: poll failed: {err}");
                }
                continue;
            }

            // ── Keyboard timeout ──────────────────────────────────────────
            if timeout_ms >= 0 {
                timeout_ms -= elapsed;
                if timeout_ms <= 0 {
                    let mut next: i64 = -1;
                    let ev = KeyEvent { code: 0, pressed: 0, timestamp: now as i32 };
                    for ki in 0..self.keyboards.len() {
                        let t = self.dispatch_kbd(ki, &[ev]);
                        if t > 0 { next = if next < 0 { t } else { next.min(t) }; }
                    }
                    timeout_ms = next;
                }
            }

            // ── Device events ─────────────────────────────────────────────
            for (i, pfd) in pfds.iter().enumerate().take(dev_count) {
                if pfd.revents == 0 { continue; }
                let kbd_idx = self.device_kbd[i];
                while let Some(devev) = self.devices[i].read_event() {
                    match devev.event_type {
                        DeviceEventType::Removed => {
                            eprintln!("DEVICE: removed {}", self.devices[i].path);
                            self.devices[i].fd = -1;
                            break;
                        }
                        DeviceEventType::Key => {
                            // Safety-exit if backspace + enter + escape all held.
                            if !self.devices[i].is_virtual {
                                panic_state.check(devev.code, devev.pressed != 0);
                            }
                            if let Some(ki) = kbd_idx {
                                let ev = KeyEvent {
                                    code:      devev.code,
                                    pressed:   devev.pressed,
                                    timestamp: now as i32,
                                };
                                let next = self.dispatch_kbd(ki, &[ev]);
                                if next > 0 {
                                    timeout_ms = if timeout_ms < 0 { next }
                                                 else { timeout_ms.min(next) };
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // ── IPC connections ───────────────────────────────────────────
            if ipc_pfd_idx.filter(|&idx| pfds[idx].revents & libc::POLLIN != 0).is_some() {
                let conn_opt = self.ipc_server.as_ref()
                    .and_then(|(l, _)| l.accept().ok())
                    .map(|(c, _)| c);
                if let Some(conn) = conn_opt {
                    self.handle_client(conn);
                }
            }

            // ── LED feedback from virtual keyboard → physical devices ──────
            #[cfg(target_os = "linux")]
            if pfds[vkbd_pfd_idx].revents & libc::POLLIN != 0 {
                while let Some((code, state)) = self.output.vkbd.read_led_event() {
                    for dev in &self.devices {
                        if dev.grabbed && !dev.is_virtual {
                            dev.set_led(code, state);
                        }
                    }
                }
            }

            // ── Hot-plug (Linux only) ─────────────────────────────────────
            #[cfg(target_os = "linux")]
            if let Some((_idx, ino)) = devmon_pfd_idx
                .filter(|&idx| pfds[idx].revents & libc::POLLIN != 0)
                .and_then(|idx| devmon.as_mut().map(|ino| (idx, ino)))
            {
                let mut buf = [0u8; 4096];
                if let Ok(events) = ino.read_events(&mut buf) {
                    for event in events {
                        if let Some(name) = event.name {
                            let name_str = name.to_str().unwrap_or("");
                            if name_str.starts_with("event") {
                                let path = format!("/dev/input/{name_str}");
                                if let Ok(mut dev) = Device::init(&path) {
                                    let kbd_idx = manage_device(&self.keyboards, &mut dev);
                                    eprintln!("DEVICE: hot-plugged {}", path);
                                    self.devices.push(dev);
                                    self.device_kbd.push(kbd_idx);
                                }
                            }
                        }
                    }
                }
            }

            // ── Compact removed devices ───────────────────────────────────
            let mut j = 0;
            for i in 0..self.devices.len() {
                if self.devices[i].fd != -1 {
                    self.devices.swap(i, j);
                    self.device_kbd.swap(i, j);
                    j += 1;
                }
            }
            self.devices.truncate(j);
            self.device_kbd.truncate(j);
        }
    }
}

// ── Output adapters ────────────────────────────────────────────────────────

#[allow(dead_code)] // device_fds is read inside #[cfg(target_os = "linux")] only
struct DaemonOutput<'a> {
    vkbd:       &'a Vkbd,
    keystate:   &'a mut [u8; 256],
    listeners:  &'a mut Vec<RawFd>,
    /// Raw fds of grabbed physical devices for this keyboard (layer_indicator LED).
    device_fds: &'a [RawFd],
}

impl<'a> Output for DaemonOutput<'a> {
    fn send_key(&mut self, code: u8, state: u8) {
        self.keystate[code as usize] = state;
        self.vkbd.send_key(code, state);
    }

    fn on_layer_change(&mut self, kbd: &Keyboard, layer_idx: usize, active: u8) {
        // Layer-indicator LED: set LED 1 on all associated devices to reflect
        // whether any non-layout layer is currently active.
        #[cfg(target_os = "linux")]
        if kbd.config.layer_indicator != 0 {
            let has_active = (1..kbd.config.layers.len()).any(|i| {
                kbd.config.layers[i].layer_type != LayerType::Layout
                    && kbd.layer_state[i].active != 0
            });
            for &fd in self.device_fds {
                write_led_fd(fd, 1, has_active);
            }
        }

        if self.listeners.is_empty() { return; }
        let layer = &kbd.config.layers[layer_idx];
        let msg = if layer.layer_type == LayerType::Layout {
            format!("/{}\n", layer.name)
        } else {
            format!("{}{}\n", if active != 0 { '+' } else { '-' }, layer.name)
        };
        let msg_bytes = msg.as_bytes();
        let len = msg_bytes.len() as isize;
        self.listeners.retain(|&fd| {
            // SAFETY: fd is a valid open socket fd added in LayerListen; buffer pointer and size are correct.
            let n = unsafe {
                libc::write(fd, msg_bytes.as_ptr() as *const libc::c_void, msg_bytes.len())
            };
            // SAFETY: fd was written to above; close is safe when write fails (pipe broken, client gone).
            if n == len { true } else { unsafe { libc::close(fd); }; false }
        });
    }
}

/// Minimal Output adapter that routes directly to the virtual keyboard —
/// used for IPC macro execution outside a keyboard state machine.
struct VkbdOutput<'a> {
    vkbd: &'a Vkbd,
}
impl<'a> Output for VkbdOutput<'a> {
    fn send_key(&mut self, code: u8, state: u8) { self.vkbd.send_key(code, state); }
    fn on_layer_change(&mut self, _: &Keyboard, _: usize, _: u8) {}
}
