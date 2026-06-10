#![allow(non_camel_case_types, dead_code)]
//! macOS keyboard capture (CGEventTap) and injection (CGEventPost).

use std::os::unix::io::RawFd;
use libc::{c_void, c_long, c_char};

// ── FFI types ────────────────────────────────────────────────────────────────

type CFTypeRef          = *const c_void;
type CFAllocatorRef     = *const c_void;
type CFStringRef        = *const c_void;
type CFRunLoopRef       = *const c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFMachPortRef      = *mut c_void;
pub type CGEventRef     = *mut c_void;
type CGEventTapProxy    = *mut c_void;
type CGEventType        = u32;
type CGEventMask        = u64;
pub type CGEventFlags   = u64;
pub type CGKeyCode      = u16;
type CGEventField       = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CGPoint { pub x: f64, pub y: f64 }

// ── Constants ────────────────────────────────────────────────────────────────

const CG_EVENT_KEY_DOWN:       CGEventType = 10;
const CG_EVENT_KEY_UP:         CGEventType = 11;
const CG_EVENT_FLAGS_CHANGED:  CGEventType = 12;
const NX_SYSDEFINED:           CGEventType = 14;
const CG_TAP_DISABLED_TIMEOUT: CGEventType = 0xFFFF_FFFE;
const CG_TAP_DISABLED_USER:    CGEventType = 0xFFFF_FFFF;

const CG_HID_EVENT_TAP: u32 = 0;
const CG_HEAD_INSERT:   u32 = 0;
const CG_TAP_DEFAULT:   u32 = 0;

const FIELD_KBD_KEYCODE:      CGEventField = 9;
const FIELD_KBD_AUTOREPEAT:   CGEventField = 8;
const FIELD_SOURCE_USERDATA:  CGEventField = 42;

// UTF-8 encoding value used by CFString.
const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

pub const FLAG_ALPHA_SHIFT:  CGEventFlags = 0x0001_0000;
pub const FLAG_SHIFT:        CGEventFlags = 0x0002_0000;
pub const FLAG_CONTROL:      CGEventFlags = 0x0004_0000;
pub const FLAG_ALTERNATE:    CGEventFlags = 0x0008_0000;
pub const FLAG_COMMAND:      CGEventFlags = 0x0010_0000;
pub const FLAG_NUMERIC_PAD:  CGEventFlags = 0x0020_0000;
pub const FLAG_SECONDARY_FN: CGEventFlags = 0x0080_0000;

const KEYD_MARKER: i64 = 0x6B65_7964;

type TapCallbackFn = unsafe extern "C" fn(
    CGEventTapProxy, CGEventType, CGEventRef, *mut c_void,
) -> CGEventRef;

// ── Framework linkage ────────────────────────────────────────────────────────

#[link(name = "CoreGraphics", kind = "framework")]
#[expect(clippy::duplicated_attributes, reason = "separate framework links share kind = framework")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32, place: u32, options: u32,
        evmask: CGEventMask,
        callback: TapCallbackFn,
        userinfo: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    fn CFMachPortCreateRunLoopSource(
        alloc: CFAllocatorRef,
        port: CFMachPortRef,
        order: c_long,
    ) -> CFRunLoopSourceRef;

    static kCFRunLoopCommonModes: CFStringRef;

    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, src: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
    fn CFRelease(obj: CFTypeRef);

    pub fn CGEventGetIntegerValueField(ev: CGEventRef, field: CGEventField) -> i64;
    pub fn CGEventSetIntegerValueField(ev: CGEventRef, field: CGEventField, val: i64);
    pub fn CGEventGetFlags(ev: CGEventRef) -> CGEventFlags;
    pub fn CGEventSetFlags(ev: CGEventRef, flags: CGEventFlags);
    pub fn CGEventSetType(ev: CGEventRef, ty: CGEventType);
    pub fn CGEventPost(tap: u32, ev: CGEventRef);
    pub fn CGEventCreateKeyboardEvent(
        source: *const c_void, key: CGKeyCode, key_down: bool,
    ) -> CGEventRef;

    // CFPreferences — for reading system key-repeat settings.
    fn CFPreferencesGetAppIntegerValue(
        key:            CFStringRef,
        application_id: CFStringRef,
        key_exists:     *mut bool,
    ) -> libc::c_long;
    fn CFStringCreateWithCString(
        alloc:    CFAllocatorRef,
        c_str:    *const libc::c_char,
        encoding: u32,
    ) -> CFStringRef;
    static kCFPreferencesAnyApplication: CFStringRef;
}


// ── Keycode tables ───────────────────────────────────────────────────────────

// CGKeyCode (index 0–127) → keyd code; 0 = unmapped
static CGKEY_TO_KEYD: [u8; 128] = [
    30,  // 0x00 kVK_ANSI_A
    31,  // 0x01 kVK_ANSI_S
    32,  // 0x02 kVK_ANSI_D
    33,  // 0x03 kVK_ANSI_F
    35,  // 0x04 kVK_ANSI_H
    34,  // 0x05 kVK_ANSI_G
    44,  // 0x06 kVK_ANSI_Z
    45,  // 0x07 kVK_ANSI_X
    46,  // 0x08 kVK_ANSI_C
    47,  // 0x09 kVK_ANSI_V
    86,  // 0x0A kVK_ISO_Section
    48,  // 0x0B kVK_ANSI_B
    16,  // 0x0C kVK_ANSI_Q
    17,  // 0x0D kVK_ANSI_W
    18,  // 0x0E kVK_ANSI_E
    19,  // 0x0F kVK_ANSI_R
    21,  // 0x10 kVK_ANSI_Y
    20,  // 0x11 kVK_ANSI_T
    2,   // 0x12 kVK_ANSI_1
    3,   // 0x13 kVK_ANSI_2
    4,   // 0x14 kVK_ANSI_3
    5,   // 0x15 kVK_ANSI_4
    7,   // 0x16 kVK_ANSI_6
    6,   // 0x17 kVK_ANSI_5
    13,  // 0x18 kVK_ANSI_Equal
    10,  // 0x19 kVK_ANSI_9
    8,   // 0x1A kVK_ANSI_7
    12,  // 0x1B kVK_ANSI_Minus
    9,   // 0x1C kVK_ANSI_8
    11,  // 0x1D kVK_ANSI_0
    27,  // 0x1E kVK_ANSI_RightBracket
    24,  // 0x1F kVK_ANSI_O
    22,  // 0x20 kVK_ANSI_U
    26,  // 0x21 kVK_ANSI_LeftBracket
    23,  // 0x22 kVK_ANSI_I
    25,  // 0x23 kVK_ANSI_P
    28,  // 0x24 kVK_Return
    38,  // 0x25 kVK_ANSI_L
    36,  // 0x26 kVK_ANSI_J
    40,  // 0x27 kVK_ANSI_Quote
    37,  // 0x28 kVK_ANSI_K
    39,  // 0x29 kVK_ANSI_Semicolon
    43,  // 0x2A kVK_ANSI_Backslash
    51,  // 0x2B kVK_ANSI_Comma
    53,  // 0x2C kVK_ANSI_Slash
    49,  // 0x2D kVK_ANSI_N
    50,  // 0x2E kVK_ANSI_M
    52,  // 0x2F kVK_ANSI_Period
    15,  // 0x30 kVK_Tab
    57,  // 0x31 kVK_Space
    41,  // 0x32 kVK_ANSI_Grave
    14,  // 0x33 kVK_Delete (backspace)
    0,   // 0x34 unused
    1,   // 0x35 kVK_Escape
    126, // 0x36 kVK_RightCommand
    125, // 0x37 kVK_Command
    42,  // 0x38 kVK_Shift
    58,  // 0x39 kVK_CapsLock
    56,  // 0x3A kVK_Option
    29,  // 0x3B kVK_Control
    54,  // 0x3C kVK_RightShift
    100, // 0x3D kVK_RightOption
    97,  // 0x3E kVK_RightControl
    254, // 0x3F kVK_Function
    187, // 0x40 kVK_F17
    83,  // 0x41 kVK_KP_Decimal
    0,   // 0x42
    55,  // 0x43 kVK_KP_Multiply
    0,   // 0x44
    78,  // 0x45 kVK_KP_Plus
    0,   // 0x46
    69,  // 0x47 kVK_KP_Clear (numlock)
    115, // 0x48 kVK_VolumeUp
    114, // 0x49 kVK_VolumeDown
    113, // 0x4A kVK_Mute
    98,  // 0x4B kVK_KP_Divide
    96,  // 0x4C kVK_KP_Enter
    0,   // 0x4D
    74,  // 0x4E kVK_KP_Minus
    188, // 0x4F kVK_F18
    189, // 0x50 kVK_F19
    117, // 0x51 kVK_KP_Equals
    82,  // 0x52 kVK_KP_0
    79,  // 0x53 kVK_KP_1
    80,  // 0x54 kVK_KP_2
    81,  // 0x55 kVK_KP_3
    75,  // 0x56 kVK_KP_4
    76,  // 0x57 kVK_KP_5
    77,  // 0x58 kVK_KP_6
    71,  // 0x59 kVK_KP_7
    190, // 0x5A kVK_F20
    72,  // 0x5B kVK_KP_8
    73,  // 0x5C kVK_KP_9
    124, // 0x5D kVK_JIS_Yen
    89,  // 0x5E kVK_JIS_Underscore
    95,  // 0x5F kVK_JIS_KP_Comma
    63,  // 0x60 kVK_F5
    64,  // 0x61 kVK_F6
    65,  // 0x62 kVK_F7
    61,  // 0x63 kVK_F3
    66,  // 0x64 kVK_F8
    67,  // 0x65 kVK_F9
    93,  // 0x66 kVK_JIS_Eisu
    87,  // 0x67 kVK_F11
    90,  // 0x68 kVK_JIS_Kana
    183, // 0x69 kVK_F13
    186, // 0x6A kVK_F16
    184, // 0x6B kVK_F14
    0,   // 0x6C
    68,  // 0x6D kVK_F10
    0,   // 0x6E
    88,  // 0x6F kVK_F12
    0,   // 0x70
    185, // 0x71 kVK_F15
    110, // 0x72 kVK_Help (insert)
    102, // 0x73 kVK_Home
    104, // 0x74 kVK_PageUp
    111, // 0x75 kVK_ForwardDelete
    62,  // 0x76 kVK_F4
    107, // 0x77 kVK_End
    60,  // 0x78 kVK_F2
    109, // 0x79 kVK_PageDown
    59,  // 0x7A kVK_F1
    105, // 0x7B kVK_LeftArrow
    106, // 0x7C kVK_RightArrow
    108, // 0x7D kVK_DownArrow
    103, // 0x7E kVK_UpArrow
    0,   // 0x7F
];

pub fn cgkey_to_keyd_code(cgkey: u16) -> Option<u8> {
    if cgkey >= 128 { return None; }
    let k = CGKEY_TO_KEYD[cgkey as usize];
    if k == 0 { None } else { Some(k) }
}

pub fn keyd_to_cgkey_code(keyd: u8) -> Option<u16> {
    CGKEY_TO_KEYD
        .iter()
        .position(|&k| k == keyd && keyd != 0)
        .map(|i| i as u16)
}

pub fn modifier_bit_for_cgkey(cgkey: u16) -> CGEventFlags {
    match cgkey {
        0x38 | 0x3C => FLAG_SHIFT,
        0x3B | 0x3E => FLAG_CONTROL,
        0x3A | 0x3D => FLAG_ALTERNATE,
        0x37 | 0x36 => FLAG_COMMAND,
        0x39        => FLAG_ALPHA_SHIFT,
        _           => 0,
    }
}

pub fn is_modifier_cgkey(cgkey: u16) -> bool {
    matches!(cgkey, 0x36..=0x3E)
}

pub fn active_modifier_flags(key_states: &[u8; 128]) -> CGEventFlags {
    let mut flags: CGEventFlags = 0;
    for (i, &s) in key_states.iter().enumerate() {
        if s != 0 { flags |= modifier_bit_for_cgkey(i as u16); }
    }
    flags
}

// ── CGEventTap callback ──────────────────────────────────────────────────────

struct TapCtx {
    write_fd: RawFd,
    port:     CFMachPortRef,  // set before CFRunLoopRun, before callback can fire
    mod_down: [u8; 128],
}

unsafe extern "C" fn tap_callback(
    _proxy:     CGEventTapProxy,
    event_type: CGEventType,
    event:      CGEventRef,
    user_info:  *mut c_void,
) -> CGEventRef {
    unsafe {
        // SAFETY: user_info is the TapCtx pointer passed to CGEventTapCreate; it is valid
        // for the lifetime of the run loop, and the callback is single-threaded.
        let ctx = &mut *(user_info as *mut TapCtx);

        if event_type == CG_TAP_DISABLED_TIMEOUT || event_type == CG_TAP_DISABLED_USER {
            log::warn!("keyd: event tap disabled, re-enabling");
            CGEventTapEnable(ctx.port, true);
            return event;
        }

        if CGEventGetIntegerValueField(event, FIELD_SOURCE_USERDATA) == KEYD_MARKER {
            return event;
        }

        let (cgkey, pressed): (u16, u8) = match event_type {
            CG_EVENT_KEY_DOWN => {
                (CGEventGetIntegerValueField(event, FIELD_KBD_KEYCODE) as u16, 1)
            }
            CG_EVENT_KEY_UP => {
                (CGEventGetIntegerValueField(event, FIELD_KBD_KEYCODE) as u16, 0)
            }
            CG_EVENT_FLAGS_CHANGED => {
                let k = CGEventGetIntegerValueField(event, FIELD_KBD_KEYCODE) as u16;
                let p = infer_mod_pressed(ctx, k, CGEventGetFlags(event));
                (k, p)
            }
            _ => return event,  // NX_SYSDEFINED (media keys) etc: pass through
        };

        log::trace!("keyd tap: cgkey={cgkey}, pressed={pressed}");
        let raw: [u8; 3] = [(cgkey >> 8) as u8, (cgkey & 0xFF) as u8, pressed];
        if libc::write(ctx.write_fd, raw.as_ptr() as *const c_void, 3) < 0 {
            log::error!("keyd: failed to write to pipe");
        }

        std::ptr::null_mut()
    }
}

fn infer_mod_pressed(ctx: &mut TapCtx, cgkey: u16, flags: CGEventFlags) -> u8 {
    if cgkey >= 128 { return 0; }
    let bit = modifier_bit_for_cgkey(cgkey);
    if bit == 0 { return 0; }
    let flag_set = (flags & bit) != 0;
    let was_down = ctx.mod_down[cgkey as usize] != 0;
    let pressed  = u8::from(flag_set && !was_down);
    ctx.mod_down[cgkey as usize] = pressed;
    pressed
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Start the CGEventTap on a background thread. Returns the pipe read fd.
pub fn tap_init() -> RawFd {
    let mut fds = [0i32; 2];
    // SAFETY: fds is a 2-element array; pipe() fills it with valid read/write file descriptors.
    assert!(unsafe { libc::pipe(fds.as_mut_ptr()) } == 0, "pipe() failed");
    let read_fd  = fds[0];
    let write_fd = fds[1];
    // SAFETY: read_fd and write_fd are valid fds just created by pipe(); O_NONBLOCK is a valid flag.
    unsafe {
        libc::fcntl(read_fd, libc::F_SETFL, libc::O_NONBLOCK);
        libc::fcntl(write_fd, libc::F_SETFL, libc::O_NONBLOCK);
    };

    // Cast to usize so the closure is Send (usize is Send; *mut is not).
    let ctx_addr: usize = Box::into_raw(Box::new(TapCtx {
        write_fd,
        port:     std::ptr::null_mut(),
        mod_down: [0; 128],
    })) as usize;

    let mask: CGEventMask = (1u64 << CG_EVENT_KEY_DOWN)
        | (1u64 << CG_EVENT_KEY_UP)
        | (1u64 << CG_EVENT_FLAGS_CHANGED)
        | (1u64 << NX_SYSDEFINED);

    std::thread::spawn(move || {
        // SAFETY: ctx_addr was obtained from Box::into_raw; the Box is not freed until the run loop exits.
        let ctx_ptr = ctx_addr as *mut TapCtx;
        unsafe {
            // SAFETY: All CGEventTap API pointers are valid CFTypeRefs; tap_callback is a valid C fn pointer.
            let port = CGEventTapCreate(
                CG_HID_EVENT_TAP, CG_HEAD_INSERT, CG_TAP_DEFAULT,
                mask, tap_callback, ctx_ptr as *mut c_void,
            );
            if port.is_null() {
                eprintln!(
                    "keyd: CGEventTap creation failed.\n\
                     Grant Accessibility permission in System Settings → Privacy & Security → Accessibility\n\
                     (or run as root)"
                );
                std::process::exit(1);
            }

            (*ctx_ptr).port = port;  // safe: set before CFRunLoopRun starts dispatching

            let src = CFMachPortCreateRunLoopSource(std::ptr::null(), port, 0);
            CFRunLoopAddSource(CFRunLoopGetCurrent(), src, kCFRunLoopCommonModes);
            CGEventTapEnable(port, true);

            log::info!("keyd: CGEventTap active");
            CFRunLoopRun();

            CFRelease(src as CFTypeRef);
            CFRelease(port as CFTypeRef);
        }
    });

    read_fd
}

#[derive(Debug, PartialEq)]
pub enum TapReadResult {
    Ok(u16, u8),
    None,
    EOF,
}

/// Non-blocking read of one pending event from the tap pipe.
pub fn tap_read(fd: RawFd) -> TapReadResult {
    let mut buf = [0u8; 3];
    // SAFETY: fd is the read end of the pipe created in tap_init; buffer is 3 bytes as expected.
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, 3) };
    if n == 0 {
        return TapReadResult::EOF;
    }
    if n < 3 {
        return TapReadResult::None;
    }
    TapReadResult::Ok(u16::from_be_bytes([buf[0], buf[1]]), buf[2])
}

/// Read the system key-repeat delay and interval (both in milliseconds).
/// Falls back to macOS defaults (500 ms delay, 33 ms interval) if unavailable.
pub fn get_repeat_settings() -> (u64, u64) {
    // SAFETY: CoreFoundation API calls; all returned CFTypeRefs are released before returning.
    unsafe {
        let make_cfstr = |s: &std::ffi::CStr| -> CFStringRef {
            CFStringCreateWithCString(std::ptr::null(), s.as_ptr(), CF_STRING_ENCODING_UTF8)
        };
        let initial_key  = make_cfstr(c"InitialKeyRepeat");
        let interval_key = make_cfstr(c"KeyRepeat");

        let mut ok = false;
        let delay_units = CFPreferencesGetAppIntegerValue(
            initial_key, kCFPreferencesAnyApplication, &mut ok,
        );
        let delay_ms = if ok && delay_units > 0 {
            (delay_units as u64) * 1000 / 60
        } else {
            500
        };

        ok = false;
        let interval_units = CFPreferencesGetAppIntegerValue(
            interval_key, kCFPreferencesAnyApplication, &mut ok,
        );
        let interval_ms = if ok && interval_units > 0 {
            (interval_units as u64) * 1000 / 60
        } else {
            33
        };

        CFRelease(initial_key  as CFTypeRef);
        CFRelease(interval_key as CFTypeRef);

        (delay_ms, interval_ms)
    }
}

/// Inject a repeat key-down event tagged with kCGKeyboardEventAutorepeat=1.
pub fn post_key_repeat(cgkey: u16, key_states: &[u8; 128]) {
    // SAFETY: CGEventCreateKeyboardEvent and CGEventPost are thread-safe CGEvent APIs.
    unsafe {
        let ev = CGEventCreateKeyboardEvent(std::ptr::null(), cgkey, true);
        if ev.is_null() { return; }

        let mut flags = CGEventGetFlags(ev);
        let mod_mask = FLAG_SHIFT | FLAG_CONTROL | FLAG_ALTERNATE | FLAG_COMMAND | FLAG_ALPHA_SHIFT;
        flags &= !mod_mask;
        flags |= active_modifier_flags(key_states);
        CGEventSetFlags(ev, flags);

        CGEventSetIntegerValueField(ev, FIELD_KBD_AUTOREPEAT, 1);
        CGEventSetIntegerValueField(ev, FIELD_SOURCE_USERDATA, KEYD_MARKER);
        CGEventPost(CG_HID_EVENT_TAP, ev);
        CFRelease(ev as CFTypeRef);
    }
}

// ── AppKit / Objective-C runtime — media key injection ───────────────────────
//
// CGEventCreateKeyboardEvent does not trigger OS-level volume/brightness/media
// actions on modern macOS. These keys require NX_SYSDEFINED events, which are
// created via NSEvent.otherEventWithType:... from the AppKit framework.

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> *const c_void;
    fn sel_registerName(name: *const c_char) -> *const c_void;

    // objc_msgSend typed for [NSEvent otherEventWithType:location:modifierFlags:
    //                          timestamp:windowNumber:context:subtype:data1:data2:]
    #[link_name = "objc_msgSend"]
    fn ns_event_other_event(
        receiver:   *const c_void,
        sel:        *const c_void,
        ty:         u64,     // NSEventType (NSUInteger)
        loc_x:      f64,     // NSPoint.x  (CGFloat)
        loc_y:      f64,     // NSPoint.y  (CGFloat)
        flags:      u64,     // NSEventModifierFlags (NSUInteger)
        timestamp:  f64,     // NSTimeInterval (double)
        window_num: isize,   // NSInteger
        context:    *const c_void,  // NSGraphicsContext* (nullable)
        subtype:    i32,     // NSEventSubtype (short, promoted to int in C ABI)
        data1:      isize,   // NSInteger
        data2:      isize,   // NSInteger
    ) -> *const c_void;     // NSEvent*

    // objc_msgSend typed for [nsEvent CGEvent]
    #[allow(clashing_extern_declarations)]
    #[link_name = "objc_msgSend"]
    fn ns_event_get_cgevent(
        receiver: *const c_void,
        sel:      *const c_void,
    ) -> CGEventRef;
}

// NX media-key types (from IOKit/hidsystem/ev_keymap.h)
const NX_KEYTYPE_SOUND_UP:        isize = 0;
const NX_KEYTYPE_SOUND_DOWN:      isize = 1;
const NX_KEYTYPE_BRIGHTNESS_UP:   isize = 2;
const NX_KEYTYPE_BRIGHTNESS_DOWN: isize = 3;
const NX_KEYTYPE_MUTE:            isize = 7;
const NX_KEYTYPE_PLAY:            isize = 16;
const NX_KEYTYPE_NEXT:            isize = 17;
const NX_KEYTYPE_PREVIOUS:        isize = 18;

/// Map a keyd code to its NX media-key type, if it is a media/system key.
pub fn keyd_to_nx_keytype(keyd: u8) -> Option<isize> {
    use crate::keys::{
        KEYD_VOLUMEUP, KEYD_VOLUMEDOWN, KEYD_MUTE,
        KEYD_PLAYPAUSE, KEYD_NEXTSONG, KEYD_PREVIOUSSONG,
        KEYD_BRIGHTNESSUP, KEYD_BRIGHTNESSDOWN,
    };
    match keyd {
        KEYD_VOLUMEUP       => Some(NX_KEYTYPE_SOUND_UP),
        KEYD_VOLUMEDOWN     => Some(NX_KEYTYPE_SOUND_DOWN),
        KEYD_MUTE           => Some(NX_KEYTYPE_MUTE),
        KEYD_PLAYPAUSE      => Some(NX_KEYTYPE_PLAY),
        KEYD_NEXTSONG       => Some(NX_KEYTYPE_NEXT),
        KEYD_PREVIOUSSONG   => Some(NX_KEYTYPE_PREVIOUS),
        KEYD_BRIGHTNESSUP   => Some(NX_KEYTYPE_BRIGHTNESS_UP),
        KEYD_BRIGHTNESSDOWN => Some(NX_KEYTYPE_BRIGHTNESS_DOWN),
        _                   => None,
    }
}

/// Inject a media/system key via NSEvent NX_SYSDEFINED.
///
/// Volume, brightness, play/pause, next, previous keys cannot be injected via
/// CGEventCreateKeyboardEvent on modern macOS. They require NX_SYSDEFINED events.
pub fn post_media_key(nx_type: isize, pressed: bool) {
    // SAFETY: Objective-C runtime calls via raw selectors; all pointer arguments are valid or null as required.
    unsafe {
        let class = objc_getClass(c"NSEvent".as_ptr());
        if class.is_null() {
            log::error!("keyd: NSEvent class unavailable");
            return;
        }

        let sel = sel_registerName(
            c"otherEventWithType:location:modifierFlags:timestamp:windowNumber:context:subtype:data1:data2:".as_ptr(),
        );

        // data1 = (nx_type << 16) | (direction << 8)
        //   direction: 0xa = key-down, 0xb = key-up
        let direction: isize = if pressed { 0xa } else { 0xb };
        let data1: isize     = (nx_type << 16) | (direction << 8);

        let ns_event = ns_event_other_event(
            class,
            sel,
            14u64,              // NSEventTypeSystemDefined = 14
            0.0,
            0.0,
            0xa00u64,           // standard modifierFlags for media keys
            0.0,
            0isize,
            std::ptr::null(),   // context = nil
            8i32,               // NX_SUBTYPE_AUX_CONTROL_BUTTONS = 8
            data1,
            -1isize,
        );

        if ns_event.is_null() {
            log::warn!("keyd: failed to create NSEvent for media key nx_type={nx_type}");
            return;
        }

        let cg_sel   = sel_registerName(c"CGEvent".as_ptr());
        let cg_event = ns_event_get_cgevent(ns_event, cg_sel);

        if cg_event.is_null() {
            log::warn!("keyd: NSEvent.CGEvent is null for nx_type={nx_type}");
            return;
        }

        CGEventPost(CG_HID_EVENT_TAP, cg_event);
    }
}

/// Inject a key event via CGEventPost with correct modifier flags.
pub fn post_key(cgkey: u16, pressed: bool, key_states: &[u8; 128]) {
    log::trace!("keyd post: cgkey={cgkey}, pressed={pressed}");
    // SAFETY: CGEventCreateKeyboardEvent and CGEventPost are thread-safe CGEvent APIs;
    // the event is released via CGEventPost ownership transfer.
    unsafe {
        let ev = CGEventCreateKeyboardEvent(std::ptr::null(), cgkey, pressed);
        if ev.is_null() { return; }

        if is_modifier_cgkey(cgkey) {
            CGEventSetType(ev, CG_EVENT_FLAGS_CHANGED);
        }

        let mut flags = CGEventGetFlags(ev);
        let mod_mask = FLAG_SHIFT | FLAG_CONTROL | FLAG_ALTERNATE | FLAG_COMMAND | FLAG_ALPHA_SHIFT;
        let fn_mask  = FLAG_SECONDARY_FN | FLAG_NUMERIC_PAD;
        flags &= !mod_mask;
        if !pressed { flags &= !fn_mask; }
        flags |= active_modifier_flags(key_states);
        CGEventSetFlags(ev, flags);

        CGEventSetIntegerValueField(ev, FIELD_SOURCE_USERDATA, KEYD_MARKER);
        CGEventPost(CG_HID_EVENT_TAP, ev);
        CFRelease(ev as CFTypeRef);
    }
}
