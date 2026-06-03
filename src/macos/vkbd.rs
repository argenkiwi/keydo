// macOS virtual keyboard output via CGEventPost.
// Port of keyd/src/vkbd/macos.c

use std::ffi::c_void;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Duration;
use crate::keyboard::OutputEvent;
use crate::keys::{KEYD_LEFT_MOUSE, KEYD_MIDDLE_MOUSE, KEYD_RIGHT_MOUSE};
use crate::macos::keycodes::keyd_to_cgkey;

// ---------------------------------------------------------------------------
// FFI — CoreGraphics & CoreFoundation
// ---------------------------------------------------------------------------

const KEYD_EVENT_MARKER: i64 = 0x6B657964;

// CGEventType
const EV_LEFT_MOUSE_DOWN:   u32 = 1;
const EV_LEFT_MOUSE_UP:     u32 = 2;
const EV_RIGHT_MOUSE_DOWN:  u32 = 3;
const EV_RIGHT_MOUSE_UP:    u32 = 4;
const EV_MOUSE_MOVED:       u32 = 5;
const EV_OTHER_MOUSE_DOWN:  u32 = 25;
const EV_OTHER_MOUSE_UP:    u32 = 26;

// CGMouseButton
const BTN_LEFT:   u32 = 0;
const BTN_RIGHT:  u32 = 1;
const BTN_CENTER: u32 = 2;

// CGEventField
const FIELD_SOURCE_USER_DATA:        u32 = 60;
const FIELD_KBD_AUTOREPEAT:          u32 = 8;

// CGScrollEventUnit: kCGScrollEventUnitLine = 1
const SCROLL_UNIT_LINE: u32 = 1;

// CGEventTapLocation: kCGHIDEventTap = 0
const TAP_HID: u32 = 0;

#[repr(C)]
struct CGPoint { x: f64, y: f64 }

type CGEventRef      = *mut c_void;
type CGEventSrcRef   = *mut c_void;
type CGDirectDisplay = u32;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGEventCreateKeyboardEvent(
        source: CGEventSrcRef,
        keycode: u16,
        keydown: bool,
    ) -> CGEventRef;
    fn CGEventCreateMouseEvent(
        source: CGEventSrcRef,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u32,
    ) -> CGEventRef;
    fn CGEventCreateScrollWheelEvent2(
        source: CGEventSrcRef,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> CGEventRef;
    fn CGEventCreate(source: CGEventSrcRef) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGMainDisplayID() -> CGDirectDisplay;
    fn CGDisplayPixelsWide(display: CGDirectDisplay) -> usize;
    fn CGDisplayPixelsHigh(display: CGDirectDisplay) -> usize;
    fn CFRelease(cf: *const c_void);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mark_and_post(event: CGEventRef) {
    if event.is_null() { return; }
    unsafe {
        CGEventSetIntegerValueField(event, FIELD_SOURCE_USER_DATA, KEYD_EVENT_MARKER);
        CGEventPost(TAP_HID, event);
        CFRelease(event);
    }
}

fn current_cursor() -> CGPoint {
    let cursor = unsafe { CGEventCreate(std::ptr::null_mut()) };
    let pos = unsafe { CGEventGetLocation(cursor) };
    unsafe { CFRelease(cursor) };
    pos
}

// ---------------------------------------------------------------------------
// Software key repeat
// ---------------------------------------------------------------------------

struct RepeatState {
    key:    u16,
    armed:  bool,
    gen:    u32,
}

static REPEAT: OnceLock<(Mutex<RepeatState>, Condvar)> = OnceLock::new();

fn repeat_sync() -> &'static (Mutex<RepeatState>, Condvar) {
    REPEAT.get_or_init(|| {
        (
            Mutex::new(RepeatState { key: 0, armed: false, gen: 0 }),
            Condvar::new(),
        )
    })
}

fn repeat_thread() {
    // Defaults: 500 ms initial delay, 33 ms interval.
    // These match Apple's out-of-the-box settings.
    let delay_ms    = 500u64;
    let interval_ms = 33u64;

    let (mtx, cvar) = repeat_sync();

    loop {
        let (key, gen) = {
            let mut state = mtx.lock().unwrap();
            while !state.armed {
                state = cvar.wait(state).unwrap();
            }
            (state.key, state.gen)
        };

        std::thread::sleep(Duration::from_millis(delay_ms));

        loop {
            {
                let state = mtx.lock().unwrap();
                if state.gen != gen { break; }
            }
            post_key_repeat(key);
            std::thread::sleep(Duration::from_millis(interval_ms));
        }
    }
}

fn post_key_repeat(cgkey: u16) {
    let ev = unsafe { CGEventCreateKeyboardEvent(std::ptr::null_mut(), cgkey, true) };
    if ev.is_null() { return; }
    unsafe {
        CGEventSetIntegerValueField(ev, FIELD_KBD_AUTOREPEAT, 1);
        CGEventSetIntegerValueField(ev, FIELD_SOURCE_USER_DATA, KEYD_EVENT_MARKER);
        CGEventPost(TAP_HID, ev);
        CFRelease(ev);
    }
}

fn is_modifier_cgkey(cgkey: u16) -> bool {
    matches!(cgkey, 0x38 | 0x3C | 0x3B | 0x3E | 0x3A | 0x3D | 0x37 | 0x36 | 0x3F | 0x39)
}

fn arm_repeat(cgkey: u16) {
    let (mtx, cvar) = repeat_sync();
    let mut state = mtx.lock().unwrap();
    state.key   = cgkey;
    state.armed = true;
    state.gen  += 1;
    cvar.notify_one();
}

fn cancel_repeat(cgkey: u16) {
    let (mtx, _cvar) = repeat_sync();
    let mut state = mtx.lock().unwrap();
    if state.key == cgkey && state.armed {
        state.armed = false;
        state.gen  += 1;
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn init() {
    // Ensure the repeat state is initialised and the thread is running.
    repeat_sync();
    std::thread::spawn(repeat_thread);
}

pub fn send_event(ev: &OutputEvent) -> anyhow::Result<()> {
    match ev {
        OutputEvent::Key(code, pressed) => {
            let code = *code;
            let pressed = *pressed;

            // Mouse buttons — route through CGEventCreateMouseEvent.
            let (btn_down, btn_up, btn_num) = match code {
                KEYD_LEFT_MOUSE   => (EV_LEFT_MOUSE_DOWN,  EV_LEFT_MOUSE_UP,  BTN_LEFT),
                KEYD_RIGHT_MOUSE  => (EV_RIGHT_MOUSE_DOWN, EV_RIGHT_MOUSE_UP, BTN_RIGHT),
                KEYD_MIDDLE_MOUSE => (EV_OTHER_MOUSE_DOWN, EV_OTHER_MOUSE_UP, BTN_CENTER),
                _ => {
                    // Keyboard key.
                    if let Some(cgkey) = keyd_to_cgkey(code) {
                        let event = unsafe {
                            CGEventCreateKeyboardEvent(std::ptr::null_mut(), cgkey, pressed)
                        };
                        mark_and_post(event);

                        if !is_modifier_cgkey(cgkey) {
                            if pressed { arm_repeat(cgkey); } else { cancel_repeat(cgkey); }
                        }
                    }
                    return Ok(());
                }
            };

            let mouse_type = if pressed { btn_down } else { btn_up };
            let pos = current_cursor();
            let event = unsafe {
                CGEventCreateMouseEvent(std::ptr::null_mut(), mouse_type, pos, btn_num)
            };
            mark_and_post(event);
        }

        OutputEvent::Scroll(x, y) => {
            let event = unsafe {
                CGEventCreateScrollWheelEvent2(
                    std::ptr::null_mut(),
                    SCROLL_UNIT_LINE,
                    2,
                    *y,
                    *x,
                    0,
                )
            };
            mark_and_post(event);
        }

        OutputEvent::Command(cmd) => {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .spawn()
                .ok();
        }
    }
    Ok(())
}

pub fn mouse_move(x: i32, y: i32) {
    let mut pos = current_cursor();
    pos.x += x as f64;
    pos.y += y as f64;
    let event = unsafe {
        CGEventCreateMouseEvent(std::ptr::null_mut(), EV_MOUSE_MOVED, pos, BTN_LEFT)
    };
    mark_and_post(event);
}

pub fn mouse_move_abs(x: i32, y: i32) {
    let disp = unsafe { CGMainDisplayID() };
    let w = unsafe { CGDisplayPixelsWide(disp) } as f64;
    let h = unsafe { CGDisplayPixelsHigh(disp) } as f64;
    let pos = CGPoint {
        x: (x as f64 / 1024.0) * w,
        y: (y as f64 / 1024.0) * h,
    };
    let event = unsafe {
        CGEventCreateMouseEvent(std::ptr::null_mut(), EV_MOUSE_MOVED, pos, BTN_LEFT)
    };
    mark_and_post(event);
}
