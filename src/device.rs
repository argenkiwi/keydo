use std::os::unix::io::RawFd;

pub const CAP_MOUSE: u8 = 0x1;
pub const CAP_MOUSE_ABS: u8 = 0x2;
pub const CAP_KEYBOARD: u8 = 0x4;
pub const CAP_KEY: u8 = 0x8;

/// USB vendor ID assigned to all keyd virtual input devices.
pub const KEYD_VENDOR_ID: u16 = 0x0FAC;
/// USB product ID for the keyd virtual keyboard.
pub const KEYD_KEYBOARD_PRODUCT_ID: u16 = 0x0ADE;
/// USB product ID for the keyd virtual pointer.
pub const KEYD_POINTER_PRODUCT_ID: u16 = 0x1ADE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceEventType {
    Key,
    Led,
    MouseMove,
    MouseMoveAbs,
    MouseScroll,
    Removed,
}

pub struct DeviceEvent {
    pub event_type: DeviceEventType,
    pub code: u8,
    pub pressed: u8,
    pub x: i32,
    pub y: i32,
}

pub struct Device {
    pub fd: RawFd,
    pub grabbed: bool,
    pub capabilities: u8,
    pub is_virtual: bool,
    pub id: String,
    pub name: String,
    pub path: String,
    pub minx: u32,
    pub maxx: u32,
    pub miny: u32,
    pub maxy: u32,
    pub pending_rel_x: i32,
    pub pending_rel_y: i32,
}

/// djb2-variant UID matching C's generate_uid() — platform-independent for testing.
pub fn generate_uid(num_keys: u32, absmask: u8, relmask: u8, name: &str) -> u32 {
    let mut h: u32 = 5183;
    h = h.wrapping_mul(33).wrapping_add((num_keys >> 24) as u8 as u32);
    h = h.wrapping_mul(33).wrapping_add((num_keys >> 16) as u8 as u32);
    h = h.wrapping_mul(33).wrapping_add((num_keys >> 8) as u8 as u32);
    h = h.wrapping_mul(33).wrapping_add(num_keys as u8 as u32);
    h = h.wrapping_mul(33).wrapping_add(absmask as u32);
    h = h.wrapping_mul(33).wrapping_add(relmask as u32);
    for b in name.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    h
}

// ── Linux ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
struct input_id {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
struct input_absinfo {
    value: i32,
    minimum: i32,
    maximum: i32,
    fuzz: i32,
    flat: i32,
    resolution: i32,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
struct input_event {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

// ioctl number helpers — _IOC(dir, 'E', nr, size)
#[cfg(target_os = "linux")]
#[inline]
fn ioc_r(size: u32, nr: u8) -> libc::c_ulong {
    ((2u32 << 30) | (size << 16) | (b'E' as u32) << 8 | nr as u32) as libc::c_ulong
}
#[cfg(target_os = "linux")]
#[inline]
#[allow(dead_code)]
fn ioc_w(size: u32, nr: u8) -> libc::c_ulong {
    ((1u32 << 30) | (size << 16) | (b'E' as u32) << 8 | nr as u32) as libc::c_ulong
}

// EVIOCGBIT(ev_type, len)  = _IOR('E', 0x20+ev, len)
#[cfg(target_os = "linux")]
#[inline]
fn eviocgbit(ev: u8, len: u32) -> libc::c_ulong { ioc_r(len, 0x20 + ev) }

// EVIOCGKEY(len) = _IOR('E', 0x18, len)
#[cfg(target_os = "linux")]
#[inline]
fn eviocgkey(len: u32) -> libc::c_ulong { ioc_r(len, 0x18) }

// EVIOCGABS(abs_code) = _IOR('E', 0x40+code, input_absinfo)  sizeof=24
#[cfg(target_os = "linux")]
#[inline]
fn eviocgabs(code: u8) -> libc::c_ulong { ioc_r(24, 0x40 + code) }

// EVIOCGNAME(64)  = _IOR('E', 0x06, [u8;64])
#[cfg(target_os = "linux")]
const EVIOCGNAME_64: libc::c_ulong = 0x8040_4506;  // (2<<30)|(64<<16)|(0x45<<8)|0x06

// EVIOCGID  = _IOR('E', 0x02, input_id)  sizeof=8
#[cfg(target_os = "linux")]
const EVIOCGID: libc::c_ulong = 0x8008_4502;

// EVIOCGRAB = _IOW('E', 0x90, int)  sizeof(int)=4
#[cfg(target_os = "linux")]
const EVIOCGRAB: libc::c_ulong = 0x4004_4590;

// evdev event type constants (shared with vkbd)
#[cfg(target_os = "linux")] pub(crate) const EV_SYN: u16 = 0x00;
#[cfg(target_os = "linux")] pub(crate) const EV_KEY: u16 = 0x01;
#[cfg(target_os = "linux")] pub(crate) const EV_REL: u16 = 0x02;
#[cfg(target_os = "linux")] pub(crate) const EV_ABS: u16 = 0x03;
#[cfg(target_os = "linux")] pub(crate) const EV_LED: u16 = 0x11;

// EV_REL codes (shared with vkbd)
#[cfg(target_os = "linux")] pub(crate) const REL_X: u16 = 0x00;
#[cfg(target_os = "linux")] pub(crate) const REL_Y: u16 = 0x01;
#[cfg(target_os = "linux")] pub(crate) const REL_WHEEL: u16 = 0x08;
#[cfg(target_os = "linux")] pub(crate) const REL_HWHEEL: u16 = 0x06;

// EV_ABS codes (shared with vkbd)
#[cfg(target_os = "linux")] pub(crate) const ABS_X: u16 = 0x00;
#[cfg(target_os = "linux")] pub(crate) const ABS_Y: u16 = 0x01;

// EV_KEY codes used in capability detection (from linux/input-event-codes.h)
#[cfg(target_os = "linux")] const MEDIA_KEYS: &[u32] = &[
    225, // KEY_BRIGHTNESSUP
    115, // KEY_VOLUMEUP
    530, // KEY_TOUCHPAD_TOGGLE (0x212)
    531, // KEY_TOUCHPAD_OFF    (0x213)
    248, // KEY_MICMUTE         (0xF8)
];
#[cfg(target_os = "linux")] const KEYBOARD_KEYS: &[u32] = &[
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, // KEY_1..KEY_0
    16, 17, 18, 19, 20, 21,          // KEY_Q..KEY_Y
];
#[cfg(target_os = "linux")] const KEY_MASK_LEN: u32 = 96; // (KEY_MAX+7)/8 = (767+7)/8

// Map extended Linux key codes (≥256) to KEYD codes, mirroring device.c
#[cfg(target_os = "linux")]
fn map_extended_key(code: u16) -> Option<u8> {
    use crate::keys::*;
    Some(match code {
        // Mouse buttons (BTN_0..BTN_9 = 0x100..0x109)
        0x100 => KEYD_F13, 0x101 => KEYD_F14, 0x102 => KEYD_F15,
        0x103 => KEYD_F16, 0x104 => KEYD_F17, 0x105 => KEYD_F18,
        0x106 => KEYD_F19, 0x107 => KEYD_F20, 0x108 => KEYD_F21,
        0x109 => KEYD_F22,
        // BTN_LEFT..BTN_TASK (0x110..0x117)
        0x110 => KEYD_LEFT_MOUSE,
        0x111 => KEYD_RIGHT_MOUSE,
        0x112 => KEYD_MIDDLE_MOUSE,
        0x113 => KEYD_MOUSE_1,
        0x114 => KEYD_MOUSE_2,
        0x115 => KEYD_MOUSE_FORWARD,
        0x116 => KEYD_MOUSE_BACK,
        0x117 => KEYD_F18, // BTN_TASK
        // KEY_PROG1..4 (0x148..0x14b)
        0x148 => KEYD_F21, 0x149 => KEYD_F22, 0x14a => KEYD_F23, 0x14b => KEYD_F24,
        // KEY_FAVORITES (0x164)
        0x164 => KEYD_BOOKMARKS,
        // KEY_ZOOM (0x174)
        0x174 => KEYD_ZOOM,
        // KEY_FN (0x1d0) and KEY_FN_F1..F12 (0x1d2..0x1dd), KEY_FN_RIGHT_SHIFT (0x1e5)
        0x1d0 => KEYD_FN,
        0x1d2 => KEYD_F13, 0x1d3 => KEYD_F14, 0x1d4 => KEYD_F15,
        0x1d5 => KEYD_F16, 0x1d6 => KEYD_F17, 0x1d7 => KEYD_F18,
        0x1d8 => KEYD_F19, 0x1d9 => KEYD_F20, 0x1da => KEYD_F21,
        0x1db => KEYD_F22, 0x1dc => KEYD_F23, 0x1dd => KEYD_F24,
        0x1e5 => KEYD_F13, // KEY_FN_RIGHT_SHIFT
        // KEY_KEYBOARD (0x1b0)
        0x1b0 => KEYD_F14,
        // KEY_TOUCHPAD_TOGGLE/OFF/ON (0x212..0x214)
        0x212 => KEYD_F21, 0x213 => KEYD_F17, 0x214 => KEYD_F18,
        // KEY_NOTIFICATION_CENTER (0x246)
        0x246 => KEYD_F21,
        // KEY_PICKUP_PHONE (0x289), KEY_HANGUP_PHONE (0x28a)
        0x289 => KEYD_F22, 0x28a => KEYD_F23, 0x290 => KEYD_F23,
        // Miscellaneous keyboard LCD menu keys (0x222..0x226)
        0x222 => KEYD_F20, 0x223 => KEYD_F21, 0x224 => KEYD_F22,
        0x225 => KEYD_F23, 0x226 => KEYD_F24,
        // KEY_EDITOR..KEY_MESSENGER (0x1b2..0x1bb)
        0x1b2 => KEYD_F13, 0x1b3 => KEYD_F14, 0x1b4 => KEYD_F15,
        0x1b5 => KEYD_F16, 0x1b6 => KEYD_F17, 0x1b7 => KEYD_F18,
        0x1b8 => KEYD_F19, 0x1b9 => KEYD_F20, 0x1bb => KEYD_F21,
        // KEY_ACCESSIBILITY (0x286)
        0x286 => KEYD_F23,
        _ => return None,
    })
}

#[cfg(target_os = "linux")]
fn has_key(mask: &[u8], key: u32) -> bool {
    let byte = (key / 8) as usize;
    let bit = (key % 8) as u8;
    byte < mask.len() && (mask[byte] >> bit) & 1 != 0
}

/// Returns (capabilities, num_keys, relmask, absmask).
#[cfg(target_os = "linux")]
fn resolve_device_capabilities(fd: std::os::unix::io::RawFd)
    -> (u8, u32, u8, u8)
{
    let mut keymask = [0u8; KEY_MASK_LEN as usize];
    let mut relmask: u8 = 0;
    let mut absmask: u8 = 0;

    // SAFETY: fd is a valid open evdev file descriptor; keymask is sized to hold KEY_MASK_LEN bytes.
    let r = unsafe {
        libc::ioctl(fd, eviocgbit(1 /* EV_KEY */, KEY_MASK_LEN), keymask.as_mut_ptr())
    };
    if r < 0 { return (0, 0, 0, 0); }

    // SAFETY: fd is valid; absmask is a single byte buffer matching the requested 1-byte read.
    let r = unsafe {
        libc::ioctl(fd, eviocgbit(3 /* EV_ABS */, 1), &mut absmask as *mut u8)
    };
    if r < 0 { return (0, 0, 0, 0); }

    // SAFETY: fd is valid; relmask is a single byte buffer matching the requested 1-byte read.
    let r = unsafe {
        libc::ioctl(fd, eviocgbit(2 /* EV_REL */, 1), &mut relmask as *mut u8)
    };
    if r < 0 { return (0, 0, 0, 0); }

    let num_keys: u32 = keymask.iter().map(|b| b.count_ones()).sum();

    let mut caps: u8 = 0;
    if relmask != 0 || absmask != 0 { caps |= CAP_MOUSE; }
    if absmask != 0                  { caps |= CAP_MOUSE_ABS; }
    if num_keys > 0                  { caps |= CAP_KEY; }

    let media_count = MEDIA_KEYS.iter().filter(|&&k| has_key(&keymask, k)).count();
    let kb_count    = KEYBOARD_KEYS.iter().filter(|&&k| has_key(&keymask, k)).count();
    if kb_count == KEYBOARD_KEYS.len() || media_count > 0 {
        caps |= CAP_KEYBOARD;
    }

    (caps, num_keys, relmask, absmask)
}

#[cfg(target_os = "linux")]
impl Device {
    pub fn scan() -> Vec<Device> {
        let mut devices = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/input/") {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if fname.starts_with("event") && let Ok(dev) = Device::init(path.to_str().unwrap()) {
                    devices.push(dev);
                }
            }
        }
        devices
    }

    pub fn init(path: &str) -> Result<Self, String> {
        let cpath = std::ffi::CString::new(path).map_err(|e| e.to_string())?;
        // SAFETY: cpath is a valid NUL-terminated C string; flags are valid O_* constants.
        let fd = unsafe {
            libc::open(cpath.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC)
        };
        if fd < 0 {
            return Err(format!("Failed to open {}", path));
        }

        let (caps, num_keys, relmask, absmask) = resolve_device_capabilities(fd);
        if caps == 0 {
            // SAFETY: fd was successfully opened above and has not been closed.
            unsafe { libc::close(fd) };
            return Err(format!("{} has no usable capabilities", path));
        }

        let mut name_buf = [0u8; 64];
        // SAFETY: fd is valid; name_buf is 64 bytes, matching EVIOCGNAME_64's expected buffer size.
        unsafe { libc::ioctl(fd, EVIOCGNAME_64, name_buf.as_mut_ptr()) };
        let nul = name_buf.iter().position(|&b| b == 0).unwrap_or(64);
        let name = String::from_utf8_lossy(&name_buf[..nul]).into_owned();

        // SAFETY: input_id is #[repr(C)] with only integer fields valid when zero-initialized.
        let mut info: input_id = unsafe { std::mem::zeroed() };
        // SAFETY: fd is valid; info is a properly-sized C struct for EVIOCGID.
        unsafe { libc::ioctl(fd, EVIOCGID, &mut info) };

        let uid = generate_uid(num_keys, absmask, relmask, &name);
        let id = format!("{:04x}:{:04x}:{:08x}", info.vendor, info.product, uid);

        let (mut minx, mut maxx, mut miny, mut maxy) = (0u32, 0u32, 0u32, 0u32);
        if caps & CAP_MOUSE_ABS != 0 {
            // SAFETY: input_absinfo is #[repr(C)] with only integer fields valid when zero-initialized.
            let mut ai: input_absinfo = unsafe { std::mem::zeroed() };
            // SAFETY: fd is valid; ai is the correct C struct for eviocgabs.
            if unsafe { libc::ioctl(fd, eviocgabs(0 /*ABS_X*/), &mut ai) } == 0 {
                minx = ai.minimum as u32; maxx = ai.maximum as u32;
            }
            if unsafe { libc::ioctl(fd, eviocgabs(1 /*ABS_Y*/), &mut ai) } == 0 {
                miny = ai.minimum as u32; maxy = ai.maximum as u32;
            }
        }

        Ok(Device {
            fd,
            grabbed: false,
            capabilities: caps,
            is_virtual: info.vendor == KEYD_VENDOR_ID,
            id,
            name,
            path: path.to_string(),
            minx, maxx, miny, maxy,
            pending_rel_x: 0,
            pending_rel_y: 0,
        })
    }

    pub fn grab(&mut self) -> Result<(), String> {
        if self.grabbed { return Ok(()); }

        // Wait for all keys to be released before grabbing to avoid stuck keys.
        let state_len = KEY_MASK_LEN;
        let mut state = [0u8; 96];
        let mut pending_release = false;
        loop {
            // SAFETY: self.fd is valid; state buffer is KEY_MASK_LEN (96) bytes matching EVIOCGKEY.
            let r = unsafe { libc::ioctl(self.fd, eviocgkey(state_len), state.as_mut_ptr()) };
            if r < 0 {
                return Err("EVIOCGKEY failed".to_string());
            }
            if state.iter().all(|&b| b == 0) { break; }
            pending_release = true;
        }
        if pending_release {
            // SAFETY: usleep has no memory-safety requirements.
            unsafe { libc::usleep(100) };
        }

        // SAFETY: self.fd is valid; argument 1 requests exclusive grab.
        if unsafe { libc::ioctl(self.fd, EVIOCGRAB, 1usize) } < 0 {
            return Err("EVIOCGRAB failed".to_string());
        }
        // Drain queued events accumulated before the grab.
        // SAFETY: input_event is #[repr(C)] with integer fields valid when zero-initialized.
        let mut ev: input_event = unsafe { std::mem::zeroed() };
        // SAFETY: self.fd is valid; buffer pointer and size match input_event layout.
        while unsafe {
            libc::read(self.fd, &mut ev as *mut _ as *mut libc::c_void,
                       std::mem::size_of::<input_event>())
        } > 0 {}

        self.grabbed = true;
        Ok(())
    }

    pub fn ungrab(&mut self) -> Result<(), String> {
        if !self.grabbed { return Ok(()); }
        // SAFETY: self.fd is valid; argument 0 releases the exclusive grab.
        if unsafe { libc::ioctl(self.fd, EVIOCGRAB, 0usize) } < 0 {
            return Err("EVIOCGRAB(0) failed".to_string());
        }
        self.grabbed = false;
        Ok(())
    }

    pub fn set_led(&self, led: u8, state: bool) {
        let ev = input_event {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            type_: EV_LED,
            code: led as u16,
            value: state as i32,
        };
        // SAFETY: self.fd is valid; pointer and size match the input_event layout.
        unsafe {
            libc::write(self.fd, &ev as *const _ as *const libc::c_void,
                        std::mem::size_of::<input_event>());
        }
    }

    pub fn read_event(&mut self) -> Option<DeviceEvent> {
        // SAFETY: input_event is #[repr(C)] with integer fields valid when zero-initialized.
        let mut ev: input_event = unsafe { std::mem::zeroed() };
        let sz = std::mem::size_of::<input_event>();
        // SAFETY: self.fd is valid; buffer pointer and size match input_event layout.
        let r = unsafe {
            libc::read(self.fd, &mut ev as *mut _ as *mut libc::c_void, sz)
        };

        if r < 0 {
            // SAFETY: __errno_location returns a valid pointer to the thread-local errno on Linux.
            if unsafe { *libc::__errno_location() } == libc::EAGAIN {
                return None;
            }
            // Device removed.
            self.fd = -1;
            return Some(DeviceEvent { event_type: DeviceEventType::Removed, code: 0, pressed: 0, x: 0, y: 0 });
        }

        match ev.type_ {
            EV_KEY => {
                if ev.value == 2 { return None; } // ignore key-repeat
                let code = if ev.code >= 256 {
                    map_extended_key(ev.code)?
                } else {
                    ev.code as u8
                };
                Some(DeviceEvent { event_type: DeviceEventType::Key, code, pressed: ev.value as u8, x: 0, y: 0 })
            }
            EV_REL => {
                match ev.code {
                    REL_WHEEL  => Some(DeviceEvent { event_type: DeviceEventType::MouseScroll, code: 0, pressed: 0, x: 0, y: ev.value }),
                    REL_HWHEEL => Some(DeviceEvent { event_type: DeviceEventType::MouseScroll, code: 0, pressed: 0, x: ev.value, y: 0 }),
                    REL_X => { self.pending_rel_x += ev.value; None }
                    REL_Y => { self.pending_rel_y += ev.value; None }
                    _ => None,
                }
            }
            EV_SYN => {
                if self.pending_rel_x != 0 || self.pending_rel_y != 0 {
                    let x = self.pending_rel_x;
                    let y = self.pending_rel_y;
                    self.pending_rel_x = 0;
                    self.pending_rel_y = 0;
                    Some(DeviceEvent { event_type: DeviceEventType::MouseMove, code: 0, pressed: 0, x, y })
                } else {
                    None
                }
            }
            EV_ABS => {
                match ev.code {
                    ABS_X => {
                        let w = self.maxx.saturating_sub(self.minx).max(1);
                        Some(DeviceEvent { event_type: DeviceEventType::MouseMoveAbs, code: 0, pressed: 0,
                                           x: ((ev.value as u32).wrapping_sub(self.minx) * 1024 / w) as i32, y: 0 })
                    }
                    ABS_Y => {
                        let h = self.maxy.saturating_sub(self.miny).max(1);
                        Some(DeviceEvent { event_type: DeviceEventType::MouseMoveAbs, code: 0, pressed: 0,
                                           x: 0, y: ((ev.value as u32).wrapping_sub(self.miny) * 1024 / h) as i32 })
                    }
                    _ => None,
                }
            }
            EV_LED => Some(DeviceEvent { event_type: DeviceEventType::Led, code: ev.code as u8, pressed: ev.value as u8, x: 0, y: 0 }),
            _ => None,
        }
    }
}

// ── macOS ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
impl Device {
    pub fn scan() -> Vec<Device> {
        let read_fd = crate::macos_input::tap_init();
        vec![Device {
            fd: read_fd,
            grabbed: false,
            capabilities: CAP_KEYBOARD | CAP_KEY,
            is_virtual: false,
            id: "0000:0000".to_string(),
            name: "CGEventTap".to_string(),
            path: String::new(),
            minx: 0, maxx: 0, miny: 0, maxy: 0,
            pending_rel_x: 0, pending_rel_y: 0,
        }]
    }

    pub fn init(_path: &str) -> Result<Self, String> {
        Err("Direct device access not supported on macOS".to_string())
    }

    pub fn grab(&mut self) -> Result<(), String> {
        self.grabbed = true;
        Ok(())
    }

    pub fn ungrab(&mut self) -> Result<(), String> {
        self.grabbed = false;
        Ok(())
    }

    pub fn set_led(&self, _led: u8, _state: bool) {}

    pub fn read_event(&mut self) -> Option<DeviceEvent> {
        match crate::macos_input::tap_read(self.fd) {
            crate::macos_input::TapReadResult::Ok(cgkey, pressed) => {
                let code = crate::macos_input::cgkey_to_keyd_code(cgkey)?;
                Some(DeviceEvent { event_type: DeviceEventType::Key, code, pressed, x: 0, y: 0 })
            }
            crate::macos_input::TapReadResult::EOF => {
                self.fd = -1;
                Some(DeviceEvent { event_type: DeviceEventType::Removed, code: 0, pressed: 0, x: 0, y: 0 })
            }
            crate::macos_input::TapReadResult::None => None,
        }
    }
}

// ── Other platforms (stub) ─────────────────────────────────────────────────

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl Device {
    pub fn scan() -> Vec<Device> { Vec::new() }
    pub fn init(_path: &str) -> Result<Self, String> { Err("Unsupported platform".to_string()) }
    pub fn grab(&mut self) -> Result<(), String> { Err("Unsupported platform".to_string()) }
    pub fn ungrab(&mut self) -> Result<(), String> { Err("Unsupported platform".to_string()) }
    pub fn set_led(&self, _led: u8, _state: bool) {}
    pub fn read_event(&mut self) -> Option<DeviceEvent> { None }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uid_deterministic() {
        let a = generate_uid(100, 0, 0, "Test Keyboard");
        let b = generate_uid(100, 0, 0, "Test Keyboard");
        assert_eq!(a, b);
    }

    #[test]
    fn test_generate_uid_different_names() {
        let a = generate_uid(100, 0, 0, "Keyboard A");
        let b = generate_uid(100, 0, 0, "Keyboard B");
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_uid_different_key_counts() {
        let a = generate_uid(80, 0, 0, "Kbd");
        let b = generate_uid(120, 0, 0, "Kbd");
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_uid_caps_matter() {
        let a = generate_uid(100, 1, 0, "Kbd");
        let b = generate_uid(100, 0, 1, "Kbd");
        let c = generate_uid(100, 0, 0, "Kbd");
        assert_ne!(a, c);
        assert_ne!(b, c);
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_uid_empty_name() {
        let uid = generate_uid(0, 0, 0, "");
        assert_ne!(uid, 0); // non-zero even for empty input (initial hash 5183)
    }
}
