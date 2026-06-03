// macOS keyboard input via CGEventTap.
// Port of keyd/src/macos/input.c

use std::ffi::c_void;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, Ordering};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// CGEventFlag masks
// ---------------------------------------------------------------------------

const MASK_ALPHA_SHIFT: u64   = 0x0001_0000;
const MASK_SHIFT: u64         = 0x0002_0000;
const MASK_CONTROL: u64       = 0x0004_0000;
const MASK_ALTERNATE: u64     = 0x0008_0000;
const MASK_COMMAND: u64       = 0x0010_0000;
const MASK_SECONDARY_FN: u64  = 0x0080_0000;

// CGEventType values
const EV_KEY_DOWN:             u32 = 10;
const EV_KEY_UP:               u32 = 11;
const EV_FLAGS_CHANGED:        u32 = 12;
const EV_TAP_DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const EV_TAP_DISABLED_INPUT:   u32 = 0xFFFF_FFFF;

// CGEventField values
const FIELD_SOURCE_USER_DATA: u32 = 60;
const FIELD_KEYBOARD_KEYCODE: u32 = 9;

// CGEventTap location / placement / options
const TAP_LOC_HID:     u32 = 0;
const TAP_PLACE_HEAD:  u32 = 0;
const TAP_OPT_DEFAULT: u32 = 0;

// Injected-event marker — same value as keyd so both can coexist
const KEYD_EVENT_MARKER: i64 = 0x6B657964;

// CGEventMaskBit(type) = 1 << type
const EVENT_MASK: u64 =
    (1u64 << EV_KEY_DOWN) |
    (1u64 << EV_KEY_UP)   |
    (1u64 << EV_FLAGS_CHANGED);

// ---------------------------------------------------------------------------
// FFI — CoreGraphics & CoreFoundation
// ---------------------------------------------------------------------------

type CGEventRef       = *mut c_void;
type CGEventTapProxy  = *mut c_void;
type CFMachPortRef    = *mut c_void;
type CFRunLoopRef     = *mut c_void;
type CFRunLoopSrcRef  = *mut c_void;
type CFAllocatorRef   = *mut c_void;

type TapCallback = unsafe extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32, place: u32, options: u32,
        eventsOfInterest: u64,
        callback: TapCallback,
        userInfo: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFMachPortCreateRunLoopSource(
        alloc: CFAllocatorRef,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSrcRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSrcRef, mode: *const c_void);
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopRun();
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopCommonModes: *const c_void;
}

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static PIPE_READ_FD:  AtomicI32 = AtomicI32::new(-1);
static PIPE_WRITE_FD: AtomicI32 = AtomicI32::new(-1);
static TAP_PORT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Per-CGKeyCode latch — tracks which modifier keys are currently held.
static MOD_DOWN: [AtomicBool; 128] = {
    const FALSE: AtomicBool = AtomicBool::new(false);
    [FALSE; 128]
};

static INIT: OnceLock<()> = OnceLock::new();

// ---------------------------------------------------------------------------
// Modifier press/release inference
// ---------------------------------------------------------------------------

fn modifier_flag_bit(cgkey: u16) -> u64 {
    match cgkey {
        0x38 | 0x3C => MASK_SHIFT,
        0x3B | 0x3E => MASK_CONTROL,
        0x3A | 0x3D => MASK_ALTERNATE,
        0x37 | 0x36 => MASK_COMMAND,
        0x39        => MASK_ALPHA_SHIFT,
        0x3F        => MASK_SECONDARY_FN,
        _           => 0,
    }
}

fn flags_changed_pressed(cgkey: u16, new_flags: u64) -> bool {
    if cgkey >= 128 {
        return false;
    }
    let bit = modifier_flag_bit(cgkey);
    let pressed = if bit != 0 {
        let flag_set  = (new_flags & bit) != 0;
        let was_down  = MOD_DOWN[cgkey as usize].load(Ordering::Relaxed);
        flag_set && !was_down
    } else {
        !MOD_DOWN[cgkey as usize].load(Ordering::Relaxed)
    };
    MOD_DOWN[cgkey as usize].store(pressed, Ordering::Relaxed);
    pressed
}

// ---------------------------------------------------------------------------
// CGEventTap callback — runs on the tap thread's CFRunLoop
// ---------------------------------------------------------------------------

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // Re-enable the tap if macOS disables it on timeout.
    if event_type == EV_TAP_DISABLED_TIMEOUT || event_type == EV_TAP_DISABLED_INPUT {
        let port = TAP_PORT.load(Ordering::Relaxed);
        if !port.is_null() {
            unsafe { CGEventTapEnable(port, true) };
        }
        return event;
    }

    // Pass through events we injected ourselves.
    let user_data = unsafe { CGEventGetIntegerValueField(event, FIELD_SOURCE_USER_DATA) };
    if user_data == KEYD_EVENT_MARKER {
        return event;
    }

    let cgkey = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYBOARD_KEYCODE) } as u16;

    let pressed = match event_type {
        EV_KEY_DOWN     => true,
        EV_KEY_UP       => false,
        EV_FLAGS_CHANGED => {
            let flags = unsafe { CGEventGetFlags(event) };
            flags_changed_pressed(cgkey, flags)
        }
        _ => return event,
    };

    // Write a 3-byte record: cgkey (u16 LE) + pressed (u8).
    // Non-blocking — dropping a single event is safer than blocking the tap.
    let write_fd = PIPE_WRITE_FD.load(Ordering::Relaxed);
    if write_fd >= 0 {
        let mut buf = [0u8; 3];
        buf[0..2].copy_from_slice(&cgkey.to_ne_bytes());
        buf[2] = pressed as u8;
        unsafe { libc::write(write_fd, buf.as_ptr() as *const c_void, 3) };
    }

    std::ptr::null_mut() // suppress the original event
}

// ---------------------------------------------------------------------------
// CGEventTap thread
// ---------------------------------------------------------------------------

fn tap_thread() {
    let port = unsafe {
        CGEventTapCreate(
            TAP_LOC_HID,
            TAP_PLACE_HEAD,
            TAP_OPT_DEFAULT,
            EVENT_MASK,
            tap_callback,
            std::ptr::null_mut(),
        )
    };

    if port.is_null() {
        eprintln!(
            "keydo: failed to create CGEventTap.\n\
             Grant Accessibility permission to keydo in:\n\
             System Settings → Privacy & Security → Accessibility\n\
             (or run keydo as root)"
        );
        std::process::exit(1);
    }

    TAP_PORT.store(port, Ordering::SeqCst);

    let source = unsafe {
        CFMachPortCreateRunLoopSource(std::ptr::null_mut(), port, 0)
    };
    let rl = unsafe { CFRunLoopGetCurrent() };
    unsafe { CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes) };
    unsafe { CGEventTapEnable(port, true) };

    eprintln!("keydo: CGEventTap active");

    unsafe { CFRunLoopRun() };

    unsafe { CFRelease(source) };
    unsafe { CFRelease(port) };
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialise the CGEventTap (idempotent — safe to call multiple times).
/// Returns the read end of the event pipe.
pub fn init() -> RawFd {
    INIT.get_or_init(|| {
        let mut fds = [0i32; 2];
        let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
        assert_eq!(ret, 0, "pipe() failed");

        // Non-blocking read so the poll loop doesn't stall.
        unsafe { libc::fcntl(fds[0], libc::F_SETFL, libc::O_NONBLOCK) };

        PIPE_WRITE_FD.store(fds[1], Ordering::Release);
        PIPE_READ_FD.store(fds[0], Ordering::Release);

        std::thread::spawn(tap_thread);
    });

    PIPE_READ_FD.load(Ordering::Acquire)
}

/// Read one pending key event from the pipe.
/// Returns (keyd_code, pressed) or None when the pipe is empty.
pub fn read_one_event() -> Option<(u16, bool)> {
    let fd = PIPE_READ_FD.load(Ordering::Relaxed);
    let mut buf = [0u8; 3];
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, 3) };
    if n != 3 {
        return None;
    }
    let cgkey = u16::from_ne_bytes([buf[0], buf[1]]);
    let pressed = buf[2] != 0;
    let keyd_code = crate::macos::keycodes::cgkey_to_keyd(cgkey);
    if keyd_code == 0 {
        None
    } else {
        Some((keyd_code, pressed))
    }
}
