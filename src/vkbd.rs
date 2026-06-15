#[cfg(target_os = "linux")]
mod linux {
    use std::fs::{File, OpenOptions};
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::io::Write;
    use std::sync::Mutex;
    use crate::device::{
        ABS_X, ABS_Y, EV_ABS, EV_KEY, EV_LED, EV_REL, EV_SYN, KEYD_KEYBOARD_PRODUCT_ID,
        KEYD_POINTER_PRODUCT_ID, KEYD_VENDOR_ID, REL_HWHEEL, REL_WHEEL, REL_X, REL_Y,
    };
    use crate::keys::*;
    use libc::*;

    const UI_SET_EVBIT: u64 = 1074025828;
    const UI_SET_KEYBIT: u64 = 1074025829;
    const UI_SET_RELBIT: u64 = 1074025830;
    const UI_SET_ABSBIT: u64 = 1074025831;
    const UI_SET_LEDBIT: u64 = 1074025833;
    const UI_DEV_CREATE: u64 = 21761;

    const EV_REP: u16 = 0x14;

    const REL_Z: u16 = 0x02;

    const BUS_USB: u16 = 0x03;

    const BTN_LEFT: u16 = 0x110;
    const BTN_RIGHT: u16 = 0x111;
    const BTN_MIDDLE: u16 = 0x112;
    const BTN_SIDE: u16 = 0x113;
    const BTN_EXTRA: u16 = 0x114;
    const BTN_FORWARD: u16 = 0x115;
    const BTN_BACK: u16 = 0x116;
    const BTN_TASK: u16 = 0x117;

    const KEY_ZOOM: u16 = 0x1a2;
    const KEY_VOICECOMMAND: u16 = 0x1bc;

    const LED_NUML: u16 = 0x00;
    const LED_MISC: u16 = 0x07;

    #[repr(C)]
    struct UinputUserDev {
        name: [c_char; 80],
        id: input_id,
        ff_effects_max: u32,
        absmax: [i32; 64],
        absmin: [i32; 64],
        absfuzz: [i32; 64],
        absflat: [i32; 64],
    }

    pub struct Vkbd {
        fd: File,
        pfd: File,
        mtx: Mutex<()>,
    }

    fn ioctl(fd: RawFd, request: u64, arg: c_int) -> c_int {
        // SAFETY: fd is a valid uinput fd; request and arg are valid uinput ioctl values.
        unsafe { libc::ioctl(fd, request, arg) }
    }

    fn ioctl_no_arg(fd: RawFd, request: u64) -> c_int {
        // SAFETY: fd is a valid uinput fd; request is a valid no-argument uinput ioctl.
        unsafe { libc::ioctl(fd, request) }
    }

    impl Vkbd {
        pub fn init(name: &str) -> Result<Self, String> {
            let fd = Self::create_virtual_keyboard(name)?;
            let pfd = Self::create_virtual_pointer("keyd virtual pointer")?;
            Ok(Self { fd, pfd, mtx: Mutex::new(()) })
        }

        fn create_virtual_keyboard(name: &str) -> Result<File, String> {
            let file = OpenOptions::new().write(true).open("/dev/uinput").map_err(|e| e.to_string())?;
            let fd = file.as_raw_fd();

            ioctl(fd, UI_SET_EVBIT, EV_REP as i32);
            ioctl(fd, UI_SET_EVBIT, EV_KEY as i32);
            ioctl(fd, UI_SET_EVBIT, EV_LED as i32);
            ioctl(fd, UI_SET_EVBIT, EV_SYN as i32);

            for (code, ent) in KEYCODE_TABLE.iter().enumerate() {
                if ent.name.is_some() {
                    ioctl(fd, UI_SET_KEYBIT, code as i32);
                }
            }

            for i in LED_NUML..=LED_MISC {
                ioctl(fd, UI_SET_LEDBIT, i as i32);
            }

            ioctl(fd, UI_SET_KEYBIT, KEY_ZOOM as i32);

            // SAFETY: UinputUserDev is #[repr(C)] with integer and byte-array fields valid when zero-initialized.
            let mut udev: UinputUserDev = unsafe { std::mem::zeroed() };
            udev.id.bustype = BUS_USB;
            udev.id.vendor = KEYD_VENDOR_ID;
            udev.id.product = KEYD_KEYBOARD_PRODUCT_ID;

            let name_bytes = name.as_bytes();
            let len = std::cmp::min(name_bytes.len(), udev.name.len() - 1);
            for (i, &byte) in name_bytes.iter().enumerate().take(len) {
                udev.name[i] = byte as c_char;
            }

            // SAFETY: udev is a fully-initialized local value; the byte slice covers exactly size_of bytes.
            let udev_slice = unsafe {
                std::slice::from_raw_parts(&udev as *const _ as *const u8, std::mem::size_of::<UinputUserDev>())
            };

            let mut file = file;
            file.write_all(udev_slice).map_err(|e| e.to_string())?;

            if ioctl_no_arg(fd, UI_DEV_CREATE) < 0 {
                return Err(format!("Failed to create uinput device: {}", std::io::Error::last_os_error()));
            }

            Ok(file)
        }

        fn create_virtual_pointer(name: &str) -> Result<File, String> {
            let file = OpenOptions::new().write(true).open("/dev/uinput").map_err(|e| e.to_string())?;
            let fd = file.as_raw_fd();

            ioctl(fd, UI_SET_EVBIT, EV_REL as i32);
            ioctl(fd, UI_SET_EVBIT, EV_ABS as i32);
            ioctl(fd, UI_SET_EVBIT, EV_KEY as i32);
            ioctl(fd, UI_SET_EVBIT, EV_SYN as i32);

            ioctl(fd, UI_SET_ABSBIT, ABS_X as i32);
            ioctl(fd, UI_SET_ABSBIT, ABS_Y as i32);
            ioctl(fd, UI_SET_RELBIT, REL_X as i32);
            ioctl(fd, UI_SET_RELBIT, REL_WHEEL as i32);
            ioctl(fd, UI_SET_RELBIT, REL_HWHEEL as i32);
            ioctl(fd, UI_SET_RELBIT, REL_Y as i32);
            ioctl(fd, UI_SET_RELBIT, REL_Z as i32);

            for code in BTN_LEFT..=BTN_TASK {
                ioctl(fd, UI_SET_KEYBIT, code as i32);
            }

            // SAFETY: UinputUserDev is #[repr(C)] with integer and byte-array fields valid when zero-initialized.
            let mut udev: UinputUserDev = unsafe { std::mem::zeroed() };
            udev.id.bustype = BUS_USB;
            udev.id.vendor = KEYD_VENDOR_ID;
            udev.id.product = KEYD_POINTER_PRODUCT_ID;
            udev.absmax[ABS_X as usize] = 1024;
            udev.absmax[ABS_Y as usize] = 1024;

            let name_bytes = name.as_bytes();
            let len = std::cmp::min(name_bytes.len(), udev.name.len() - 1);
            for (i, &byte) in name_bytes.iter().enumerate().take(len) {
                udev.name[i] = byte as c_char;
            }

            // SAFETY: udev is a fully-initialized local value; the byte slice covers exactly size_of bytes.
            let udev_slice = unsafe {
                std::slice::from_raw_parts(&udev as *const _ as *const u8, std::mem::size_of::<UinputUserDev>())
            };

            let mut file = file;
            file.write_all(udev_slice).map_err(|e| e.to_string())?;

            if ioctl_no_arg(fd, UI_DEV_CREATE) < 0 {
                return Err(format!("Failed to create uinput device: {}", std::io::Error::last_os_error()));
            }

            Ok(file)
        }

        pub fn send_key(&self, code: u8, state: u8) {
            let _lock = self.mtx.lock().unwrap();
            let mut is_btn = true;
            let mapped_code = match code {
                KEYD_LEFT_MOUSE => BTN_LEFT,
                KEYD_MIDDLE_MOUSE => BTN_MIDDLE,
                KEYD_RIGHT_MOUSE => BTN_RIGHT,
                KEYD_MOUSE_1 => BTN_SIDE,
                KEYD_MOUSE_2 => BTN_EXTRA,
                KEYD_MOUSE_BACK => BTN_BACK,
                KEYD_MOUSE_FORWARD => BTN_FORWARD,
                KEYD_ZOOM => { is_btn = false; KEY_ZOOM },
                KEYD_VOICECOMMAND => { is_btn = false; KEY_VOICECOMMAND },
                _ => { is_btn = false; code as u16 },
            };

            let fd = if is_btn {
                // Give key events preceding a button press a chance to propagate
                // from the keyboard uinput device before writing to the pointer
                // device, to avoid event-order transposition between the two
                // (faithful port of keyd's usleep(1000) in vkbd/uinput.c).
                std::thread::sleep(std::time::Duration::from_millis(1));
                self.pfd.as_raw_fd()
            } else {
                self.fd.as_raw_fd()
            };

            self.write_event(fd, EV_KEY, mapped_code, state as i32);
            self.write_event(fd, EV_SYN, 0, 0);
        }

        /// Raw fd of the virtual keyboard uinput device (for reading LED feedback).
        pub fn keyboard_fd(&self) -> RawFd {
            self.fd.as_raw_fd()
        }

        /// Read one EV_LED event from the uinput keyboard fd.
        /// Returns `(led_code, state)` or `None` if nothing is available.
        pub fn read_led_event(&self) -> Option<(u8, bool)> {
            #[repr(C)]
            struct Ev { time: libc::timeval, type_: u16, code: u16, value: i32 }
            // SAFETY: Ev is #[repr(C)] with integer fields valid when zero-initialized.
            let mut ev: Ev = unsafe { std::mem::zeroed() };
            // SAFETY: self.fd is a valid uinput fd; buffer pointer and size match Ev layout.
            let n = unsafe {
                libc::read(self.fd.as_raw_fd(),
                           &mut ev as *mut _ as *mut c_void,
                           std::mem::size_of::<Ev>())
            };
            if n == std::mem::size_of::<Ev>() as isize && ev.type_ == EV_LED {
                Some((ev.code as u8, ev.value != 0))
            } else {
                None
            }
        }

        pub fn mouse_move(&self, x: i32, y: i32) {
            let _lock = self.mtx.lock().unwrap();
            let fd = self.pfd.as_raw_fd();
            if x != 0 { self.write_event(fd, EV_REL, REL_X, x); }
            if y != 0 { self.write_event(fd, EV_REL, REL_Y, y); }
            if x != 0 || y != 0 { self.write_event(fd, EV_SYN, 0, 0); }
        }

        pub fn mouse_scroll(&self, x: i32, y: i32) {
            let _lock = self.mtx.lock().unwrap();
            let fd = self.pfd.as_raw_fd();
            if y != 0 { self.write_event(fd, EV_REL, REL_WHEEL, y); }
            if x != 0 { self.write_event(fd, EV_REL, REL_HWHEEL, x); }
            if x != 0 || y != 0 { self.write_event(fd, EV_SYN, 0, 0); }
        }

        fn write_event(&self, fd: RawFd, type_: u16, code: u16, value: i32) {
            // SAFETY: input_event is #[repr(C)] with integer fields valid when zero-initialized.
            let mut ev: input_event = unsafe { std::mem::zeroed() };
            ev.type_ = type_;
            ev.code = code;
            ev.value = value;

            // SAFETY: ev is fully initialized; the byte slice covers exactly size_of bytes.
            let ev_slice = unsafe {
                std::slice::from_raw_parts(&ev as *const _ as *const u8, std::mem::size_of::<input_event>())
            };

            // SAFETY: fd is a valid uinput fd; buffer pointer and size are correct.
            unsafe {
                libc::write(fd, ev_slice.as_ptr() as *const c_void, ev_slice.len());
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::Vkbd;

#[cfg(target_os = "macos")]
mod macos_vkbd {
    use std::sync::{Arc, Mutex, Condvar};
    use crate::macos_input;

    struct RepeatState {
        key:        u16,
        armed:      bool,
        revision:   u32,
    }

    struct SharedState {
        repeat:     RepeatState,
        key_states: [u8; 128],
    }

    pub struct Vkbd {
        shared: Arc<(Mutex<SharedState>, Condvar)>,
    }

    impl Vkbd {
        pub fn init(_name: &str) -> Result<Self, String> {
            let shared = Arc::new((
                Mutex::new(SharedState {
                    repeat: RepeatState { key: 0, armed: false, revision: 0 },
                    key_states: [0u8; 128],
                }),
                Condvar::new(),
            ));

            let shared_clone = Arc::clone(&shared);
            std::thread::spawn(move || {
                let (delay_ms, interval_ms) = macos_input::get_repeat_settings();
                let (lock, cvar) = &*shared_clone;

                loop {
                    // Wait until a key is armed.
                    let (rev, key) = {
                        let mut st = lock.lock().unwrap_or_else(|e| e.into_inner());
                        while !st.repeat.armed {
                            st = cvar.wait(st).unwrap_or_else(|e| e.into_inner());
                        }
                        (st.repeat.revision, st.repeat.key)
                    };

                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));

                    // Fire repeats at the system interval until cancelled.
                    loop {
                        let st = lock.lock().unwrap_or_else(|e| e.into_inner());
                        if st.repeat.revision != rev {
                            break;
                        }
                        let key_states = st.key_states;
                        drop(st);

                        macos_input::post_key_repeat(key, &key_states);
                        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    }
                }
            });

            Ok(Vkbd { shared })
        }

        pub fn send_key(&self, code: u8, state: u8) {
            // Media/system keys need NX_SYSDEFINED events on macOS.
            // CGEventCreateKeyboardEvent does not trigger volume/brightness/media
            // actions, and play/next/prev have no CGKey mapping at all.
            if let Some(nx_type) = macos_input::keyd_to_nx_keytype(code) {
                macos_input::post_media_key(nx_type, state != 0);
                return;
            }

            let Some(cgkey) = macos_input::keyd_to_cgkey_code(code) else { return };

            let (lock, cvar) = &*self.shared;
            {
                let mut st = lock.lock().unwrap_or_else(|e| e.into_inner());
                if (cgkey as usize) < 128 {
                    st.key_states[cgkey as usize] = state;
                }
                let key_states = st.key_states;
                drop(st);

                macos_input::post_key(cgkey, state != 0, &key_states);
            }

            // Arm or cancel the software repeat timer for non-modifier keys.
            if !macos_input::is_modifier_cgkey(cgkey) {
                let mut st = lock.lock().unwrap_or_else(|e| e.into_inner());
                if state != 0 {
                    st.repeat.key       = cgkey;
                    st.repeat.armed     = true;
                    st.repeat.revision += 1;
                    cvar.notify_one();
                } else if st.repeat.key == cgkey && st.repeat.armed {
                    st.repeat.armed     = false;
                    st.repeat.revision += 1;
                }
            }
        }

        pub fn mouse_move(&self, _x: i32, _y: i32) {}
        pub fn mouse_scroll(&self, _x: i32, _y: i32) {}
    }
}

#[cfg(target_os = "macos")]
pub use macos_vkbd::Vkbd;

#[cfg(windows)]
mod windows_vkbd {
    use std::sync::{Arc, Condvar, Mutex};

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
        MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
        SendInput,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETKEYBOARDDELAY, SPI_GETKEYBOARDSPEED, SystemParametersInfoW,
    };

    use crate::keys::*;
    use crate::windows_input::{KEYD_MARKER, is_modifier_keyd, keyd_to_scancode, keyd_to_vk};

    /// Distance the OS expects for one wheel notch (WHEEL_DELTA).
    const WHEEL_DELTA: i32 = 120;
    const XBUTTON1: i32 = 1;
    const XBUTTON2: i32 = 2;

    struct RepeatState {
        key:      u8,
        armed:    bool,
        revision: u32,
    }

    /// Virtual keyboard backed by `SendInput`. Injected keys do not auto-repeat
    /// (typematic repeat is generated below the injection point), so a software
    /// repeat thread re-fires the held key — same design as the macOS backend.
    pub struct Vkbd {
        shared: Arc<(Mutex<RepeatState>, Condvar)>,
    }

    fn send_inputs(inputs: &[INPUT]) {
        // SAFETY: inputs is a valid, contiguous slice of fully-initialized
        // INPUT structs; cbSize matches the struct the slice contains.
        unsafe {
            SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn keyboard_input(vk: u16, scan: u16, flags: u32) -> INPUT {
        // SAFETY: INPUT is a C struct fully overwritten below; zeroed is a
        // valid initial state for its integer/union fields.
        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: vk,
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: KEYD_MARKER,
        };
        input
    }

    fn mouse_input(dx: i32, dy: i32, mouse_data: i32, flags: u32) -> INPUT {
        // SAFETY: as above — fully overwritten C struct.
        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_MOUSE;
        input.Anonymous.mi = MOUSEINPUT {
            dx,
            dy,
            // DWORD field carrying a signed value (wheel deltas); wrap-cast.
            mouseData: mouse_data as u32,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: KEYD_MARKER,
        };
        input
    }

    /// Inject one key transition, choosing VK injection (media/quirky keys) or
    /// scancode injection (everything else; goes through the active layout
    /// exactly like a hardware key).
    fn inject_key(code: u8, pressed: bool) {
        let up = if pressed { 0 } else { KEYEVENTF_KEYUP };

        if let Some(vk) = keyd_to_vk(code) {
            send_inputs(&[keyboard_input(vk, 0, up)]);
            return;
        }
        let Some((scan, extended)) = keyd_to_scancode(code) else { return };
        let ext = if extended { KEYEVENTF_EXTENDEDKEY } else { 0 };
        send_inputs(&[keyboard_input(0, scan, KEYEVENTF_SCANCODE | ext | up)]);
    }

    /// System repeat settings → (initial delay ms, repeat interval ms).
    /// SPI_GETKEYBOARDDELAY: 0–3 ≈ 250–1000 ms; SPI_GETKEYBOARDSPEED:
    /// 0–31 ≈ 2.5–30 repeats/sec (documented as approximate).
    fn get_repeat_settings() -> (u64, u64) {
        let mut delay: u32 = 1;
        let mut speed: u32 = 31;
        // SAFETY: pvParam points to a u32, the documented out-parameter type
        // for both SPI_GETKEYBOARDDELAY and SPI_GETKEYBOARDSPEED.
        unsafe {
            SystemParametersInfoW(SPI_GETKEYBOARDDELAY, 0, (&raw mut delay).cast(), 0);
            SystemParametersInfoW(SPI_GETKEYBOARDSPEED, 0, (&raw mut speed).cast(), 0);
        }
        let delay_ms = (u64::from(delay.min(3)) + 1) * 250;
        let cps = 2.5 + f64::from(speed.min(31)) * 27.5 / 31.0;
        let interval_ms = (1000.0 / cps) as u64;
        (delay_ms, interval_ms)
    }

    impl Vkbd {
        pub fn init(_name: &str) -> Result<Self, String> {
            let shared = Arc::new((
                Mutex::new(RepeatState { key: 0, armed: false, revision: 0 }),
                Condvar::new(),
            ));

            let shared_clone = Arc::clone(&shared);
            std::thread::spawn(move || {
                let (delay_ms, interval_ms) = get_repeat_settings();
                let (lock, cvar) = &*shared_clone;

                loop {
                    // Wait until a key is armed.
                    let (rev, key) = {
                        let mut st = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        while !st.armed {
                            st = cvar.wait(st).unwrap_or_else(std::sync::PoisonError::into_inner);
                        }
                        (st.revision, st.key)
                    };

                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));

                    // Fire repeats at the system interval until cancelled.
                    loop {
                        let st = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        if st.revision != rev {
                            break;
                        }
                        drop(st);

                        inject_key(key, true);
                        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    }
                }
            });

            Ok(Vkbd { shared })
        }

        pub fn send_key(&self, code: u8, state: u8) {
            // Mouse buttons route to mouse injection.
            let button = match code {
                c if c == KEYD_LEFT_MOUSE => Some((MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, 0)),
                c if c == KEYD_RIGHT_MOUSE => Some((MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, 0)),
                c if c == KEYD_MIDDLE_MOUSE => Some((MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 0)),
                c if c == KEYD_MOUSE_1 || c == KEYD_MOUSE_BACK =>
                    Some((MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, XBUTTON1)),
                c if c == KEYD_MOUSE_2 || c == KEYD_MOUSE_FORWARD =>
                    Some((MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, XBUTTON2)),
                _ => None,
            };
            if let Some((down, up, data)) = button {
                let flags = if state != 0 { down } else { up };
                send_inputs(&[mouse_input(0, 0, data, flags)]);
                return;
            }

            inject_key(code, state != 0);

            // Arm or cancel the software repeat timer for non-modifier keys.
            if !is_modifier_keyd(code) {
                let (lock, cvar) = &*self.shared;
                let mut st = lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if state != 0 {
                    st.key       = code;
                    st.armed     = true;
                    st.revision += 1;
                    cvar.notify_one();
                } else if st.key == code && st.armed {
                    st.armed     = false;
                    st.revision += 1;
                }
            }
        }

        pub fn mouse_move(&self, x: i32, y: i32) {
            if x != 0 || y != 0 {
                send_inputs(&[mouse_input(x, y, 0, MOUSEEVENTF_MOVE)]);
            }
        }

        pub fn mouse_scroll(&self, x: i32, y: i32) {
            // Positive y scrolls up (wheel away from user), matching REL_WHEEL.
            if y != 0 {
                send_inputs(&[mouse_input(0, 0, y * WHEEL_DELTA, MOUSEEVENTF_WHEEL)]);
            }
            if x != 0 {
                send_inputs(&[mouse_input(0, 0, x * WHEEL_DELTA, MOUSEEVENTF_HWHEEL)]);
            }
        }
    }
}

#[cfg(windows)]
pub use windows_vkbd::Vkbd;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub struct Vkbd;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
impl Vkbd {
    pub fn init(_name: &str) -> Result<Self, String> {
        log::info!("vkbd: stub init (unsupported platform)");
        Ok(Vkbd)
    }

    pub fn send_key(&self, code: u8, state: u8) {
        log::info!("vkbd: stub send_key code={} state={}", code, state);
    }
    pub fn mouse_move(&self, _x: i32, _y: i32) {}
    pub fn mouse_scroll(&self, _x: i32, _y: i32) {}
}
