//! Windows input backend: a WH_KEYBOARD_LL low-level keyboard hook plus the
//! scancode↔keyd translation tables.
//!
//! Mirrors the macOS CGEventTap design: a single system-wide tap on a
//! background thread swallows hardware events, forwards them to the daemon
//! over a channel, and tags re-injected events with a marker so the hook
//! ignores keydo's own output.
//!
//! The translation tables are compiled on every platform so they can be
//! unit-tested without a Windows machine; only the hook FFI is gated.

use crate::keys::*;

/// Marker stored in injected events' `dwExtraInfo` so the hook ignores our own
/// output — "keyd" in ASCII, mirroring the macOS CGEvent user-data marker.
pub const KEYD_MARKER: usize = 0x6B65_7964;

/// Scancode Windows reports for the fake LCtrl it synthesizes alongside
/// RightAlt on AltGr layouts. Dropped by the hook: re-injecting RightAlt makes
/// the layout driver regenerate it, so forwarding it would double it up.
pub const FAKE_ALTGR_SCAN: u32 = 0x21D;

/// Virtual-key overrides, checked before scancode translation, for keys whose
/// scancode reporting is unreliable in low-level hooks (NumLock and Pause
/// famously share scancode 0x45).
fn vk_to_keyd(vk: u32) -> Option<u8> {
    Some(match vk {
        0x03 | 0x13 => KEYD_PAUSE, // VK_CANCEL (ctrl+break), VK_PAUSE
        0x2C => KEYD_SYSRQ,        // VK_SNAPSHOT (print screen)
        0x90 => KEYD_NUMLOCK,      // VK_NUMLOCK
        _ => return None,
    })
}

/// E0-extended PC set-1 scancode → keyd code (the standard atkbd mapping,
/// matching what Linux evdev reports for the same physical keys).
fn extended_to_keyd(scan: u16) -> Option<u8> {
    Some(match scan {
        0x10 => KEYD_PREVIOUSSONG,
        0x19 => KEYD_NEXTSONG,
        0x1C => KEYD_KPENTER,
        0x1D => KEYD_RIGHTCTRL,
        0x20 => KEYD_MUTE,
        0x21 => KEYD_CALC,
        0x22 => KEYD_PLAYPAUSE,
        0x24 => KEYD_STOPCD,
        0x2E => KEYD_VOLUMEDOWN,
        0x30 => KEYD_VOLUMEUP,
        0x32 => KEYD_HOMEPAGE,
        0x35 => KEYD_KPSLASH,
        0x37 => KEYD_SYSRQ,
        0x38 => KEYD_RIGHTALT,
        0x45 => KEYD_NUMLOCK,
        0x46 => KEYD_PAUSE,
        0x47 => KEYD_HOME,
        0x48 => KEYD_UP,
        0x49 => KEYD_PAGEUP,
        0x4B => KEYD_LEFT,
        0x4D => KEYD_RIGHT,
        0x4F => KEYD_END,
        0x50 => KEYD_DOWN,
        0x51 => KEYD_PAGEDOWN,
        0x52 => KEYD_INSERT,
        0x53 => KEYD_DELETE,
        0x5B => KEYD_LEFTMETA,
        0x5C => KEYD_RIGHTMETA,
        0x5D => KEYD_COMPOSE,
        0x5E => KEYD_POWER,
        0x5F => KEYD_SLEEP,
        0x63 => KEYD_WAKEUP,
        0x66 => KEYD_BOOKMARKS,
        0x67 => KEYD_REFRESH,
        0x68 => KEYD_STOP,
        0x69 => KEYD_FORWARD,
        0x6A => KEYD_BACK,
        0x6B => KEYD_COMPUTER,
        0x6C => KEYD_MAIL,
        _ => return None,
    })
}

/// Translate one low-level hook event to a keyd code.
///
/// `scan`/`extended` come from `KBDLLHOOKSTRUCT.scanCode` and the
/// `LLKHF_EXTENDED` flag; `vk` from `vkCode`. Evdev codes 1–88 are literally
/// PC/AT set-1 scancodes, so the non-extended translation is an identity map —
/// layout-independent, exactly matching keydo's behaviour on Linux.
pub fn scancode_to_keyd(scan: u16, extended: bool, vk: u32) -> Option<u8> {
    if let Some(code) = vk_to_keyd(vk) {
        return Some(code);
    }
    if extended {
        return extended_to_keyd(scan);
    }
    if (1..=88).contains(&scan) {
        return Some(scan as u8);
    }
    None
}

/// Reverse translation for `SendInput` scancode injection.
/// Returns `(scancode, extended)`.
///
/// Keys injected by virtual key instead (media keys, Pause, NumLock, PrtScn —
/// see [`keyd_to_vk`]) are intentionally absent.
pub fn keyd_to_scancode(code: u8) -> Option<(u16, bool)> {
    // Identity range: evdev codes 1..=88 are PC/AT set-1 scancodes.
    if (1..=88).contains(&code) {
        return Some((code as u16, false));
    }
    Some(match code {
        c if c == KEYD_KPENTER   => (0x1C, true),
        c if c == KEYD_RIGHTCTRL => (0x1D, true),
        c if c == KEYD_KPSLASH   => (0x35, true),
        c if c == KEYD_RIGHTALT  => (0x38, true),
        c if c == KEYD_HOME      => (0x47, true),
        c if c == KEYD_UP        => (0x48, true),
        c if c == KEYD_PAGEUP    => (0x49, true),
        c if c == KEYD_LEFT      => (0x4B, true),
        c if c == KEYD_RIGHT     => (0x4D, true),
        c if c == KEYD_END       => (0x4F, true),
        c if c == KEYD_DOWN      => (0x50, true),
        c if c == KEYD_PAGEDOWN  => (0x51, true),
        c if c == KEYD_INSERT    => (0x52, true),
        c if c == KEYD_DELETE    => (0x53, true),
        c if c == KEYD_LEFTMETA  => (0x5B, true),
        c if c == KEYD_RIGHTMETA => (0x5C, true),
        c if c == KEYD_COMPOSE   => (0x5D, true),
        _ => return None,
    })
}

/// Keys injected by virtual key rather than scancode: VK injection reliably
/// triggers the OS handlers for media actions (the Windows analog of the macOS
/// NX_SYSDEFINED workaround), and sidesteps the scancode ambiguity of
/// Pause/NumLock/PrtScn.
pub fn keyd_to_vk(code: u8) -> Option<u16> {
    Some(match code {
        c if c == KEYD_PAUSE        => 0x13, // VK_PAUSE
        c if c == KEYD_NUMLOCK      => 0x90, // VK_NUMLOCK
        c if c == KEYD_SYSRQ        => 0x2C, // VK_SNAPSHOT
        c if c == KEYD_MUTE         => 0xAD, // VK_VOLUME_MUTE
        c if c == KEYD_VOLUMEDOWN   => 0xAE, // VK_VOLUME_DOWN
        c if c == KEYD_VOLUMEUP     => 0xAF, // VK_VOLUME_UP
        c if c == KEYD_NEXTSONG     => 0xB0, // VK_MEDIA_NEXT_TRACK
        c if c == KEYD_PREVIOUSSONG => 0xB1, // VK_MEDIA_PREV_TRACK
        c if c == KEYD_STOPCD       => 0xB2, // VK_MEDIA_STOP
        c if c == KEYD_PLAYPAUSE    => 0xB3, // VK_MEDIA_PLAY_PAUSE
        c if c == KEYD_BACK         => 0xA6, // VK_BROWSER_BACK
        c if c == KEYD_FORWARD      => 0xA7, // VK_BROWSER_FORWARD
        c if c == KEYD_REFRESH      => 0xA8, // VK_BROWSER_REFRESH
        c if c == KEYD_STOP         => 0xA9, // VK_BROWSER_STOP
        c if c == KEYD_BOOKMARKS    => 0xAB, // VK_BROWSER_FAVORITES
        c if c == KEYD_HOMEPAGE     => 0xAC, // VK_BROWSER_HOME
        c if c == KEYD_MAIL         => 0xB4, // VK_LAUNCH_MAIL
        c if c == KEYD_CALC         => 0xB7, // VK_LAUNCH_APP2 (calculator)
        _ => return None,
    })
}

/// True for keyd modifier codes — these must not arm the software key-repeat.
pub fn is_modifier_keyd(code: u8) -> bool {
    matches!(
        code,
        c if c == KEYD_LEFTCTRL || c == KEYD_RIGHTCTRL
          || c == KEYD_LEFTSHIFT || c == KEYD_RIGHTSHIFT
          || c == KEYD_LEFTALT || c == KEYD_RIGHTALT
          || c == KEYD_LEFTMETA || c == KEYD_RIGHTMETA
    )
}

// ── Hook (Windows only) ─────────────────────────────────────────────────────

#[cfg(windows)]
mod hook {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Receiver, Sender, channel};

    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_EXTENDED,
        LLKHF_INJECTED, MSG, SetWindowsHookExW, TranslateMessage, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_SYSKEYDOWN,
    };

    /// When set, the hook swallows translated hardware events (the daemon has
    /// "grabbed" the keyboard). When clear (monitor mode, or no matching
    /// config), events are observed but passed through untouched.
    static SWALLOW: AtomicBool = AtomicBool::new(false);
    static TX: OnceLock<Sender<(u8, u8)>> = OnceLock::new();

    pub fn set_swallow(on: bool) {
        SWALLOW.store(on, Ordering::SeqCst);
    }

    /// Install the low-level keyboard hook on a dedicated message-pump thread.
    /// Returns the channel on which `(keyd_code, pressed)` pairs arrive.
    pub fn hook_init() -> Result<Receiver<(u8, u8)>, String> {
        let (tx, rx) = channel();
        TX.set(tx).map_err(|_| "keyboard hook already installed".to_string())?;

        let (ready_tx, ready_rx) = channel();
        std::thread::spawn(move || {
            // SAFETY: hook_proc matches the HOOKPROC signature; a null module
            // name returns the handle of the current executable, which outlives
            // the hook; thread id 0 makes the hook global.
            let hhook = unsafe {
                SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(hook_proc),
                    GetModuleHandleW(std::ptr::null()),
                    0,
                )
            };
            let ok = !hhook.is_null();
            let _ = ready_tx.send(ok);
            if !ok {
                return;
            }

            // A message pump is required on the installing thread for the
            // low-level hook callback to be delivered. The callback must stay
            // fast (one channel send) or Windows silently removes the hook.
            // SAFETY: MSG contains only integer/pointer fields, valid zeroed.
            let mut msg: MSG = unsafe { std::mem::zeroed() };
            // SAFETY: msg is a valid MSG buffer; null hwnd receives all
            // thread messages; GetMessageW blocks until one arrives.
            while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
                // SAFETY: msg was filled in by GetMessageW above.
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        match ready_rx.recv() {
            Ok(true) => Ok(rx),
            _ => Err(format!(
                "SetWindowsHookExW(WH_KEYBOARD_LL) failed: {}",
                std::io::Error::last_os_error()
            )),
        }
    }

    /// # Safety
    /// Called by Windows on the hook thread; `lparam` points to a valid
    /// `KBDLLHOOKSTRUCT` whenever `code >= 0` (HC_ACTION).
    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // SAFETY: forwarding unchanged arguments as the hook contract requires.
        let next = || unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) };

        if code < 0 {
            return next();
        }
        // SAFETY: for code >= 0 the system guarantees lparam points to a
        // valid KBDLLHOOKSTRUCT for the duration of this call.
        let kb = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };

        // Ignore anything injected — our own output (marker) and other tools'.
        if kb.flags & LLKHF_INJECTED != 0 || kb.dwExtraInfo == super::KEYD_MARKER {
            return next();
        }

        let swallow = SWALLOW.load(Ordering::SeqCst);

        // Fake-AltGr companion LCtrl: never forward; swallow when grabbed
        // (re-injecting RightAlt makes the layout driver regenerate it).
        if kb.scanCode == super::FAKE_ALTGR_SCAN {
            return if swallow { 1 } else { next() };
        }

        let pressed =
            u8::from(wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN);
        let extended = kb.flags & LLKHF_EXTENDED != 0;

        if let Some(keyd) = super::scancode_to_keyd(kb.scanCode as u16, extended, kb.vkCode) {
            if let Some(tx) = TX.get() {
                let _ = tx.send((keyd, pressed));
            }
            if swallow {
                return 1;
            }
        }
        // Untranslatable keys always pass through so they keep working.
        next()
    }
}

#[cfg(windows)]
pub use hook::{hook_init, set_swallow};

// ── Tests (run on every platform) ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KEYCODE_TABLE;

    #[test]
    fn identity_range_maps_to_itself() {
        for scan in 1u16..=88 {
            assert_eq!(scancode_to_keyd(scan, false, 0), Some(scan as u8));
        }
    }

    #[test]
    fn identity_range_has_key_names() {
        // Every code in the identity range must be printable by `keydo monitor`.
        for (code, ent) in KEYCODE_TABLE.iter().enumerate().take(89).skip(1) {
            assert!(ent.name.is_some(), "keyd code {code} has no name");
        }
    }

    #[test]
    fn extended_table_has_key_names() {
        for scan in 0u16..=0xFF {
            if let Some(code) = scancode_to_keyd(scan, true, 0) {
                assert!(
                    KEYCODE_TABLE[code as usize].name.is_some(),
                    "E0 {scan:#x} maps to unnamed keyd code {code}"
                );
            }
        }
    }

    #[test]
    fn scancode_round_trip() {
        // Every code we can inject by scancode must translate back to itself
        // when it re-enters a hook (relevant for other hook-based tools, and a
        // consistency check on the tables).
        for code in 1u8..=255 {
            if keyd_to_vk(code).is_some() {
                continue; // injected by VK, not scancode
            }
            if let Some((scan, extended)) = keyd_to_scancode(code) {
                assert_eq!(
                    scancode_to_keyd(scan, extended, 0),
                    Some(code),
                    "keyd {code} -> scan {scan:#x} (ext={extended}) does not round-trip"
                );
            }
        }
    }

    #[test]
    fn vk_overrides_win_over_scancode() {
        use crate::keys::{KEYD_NUMLOCK, KEYD_PAUSE, KEYD_SYSRQ};
        // NumLock: vk 0x90, scancode 0x45.
        assert_eq!(scancode_to_keyd(0x45, false, 0x90), Some(KEYD_NUMLOCK));
        // Pause: vk 0x13 (scancode reporting varies; vk decides).
        assert_eq!(scancode_to_keyd(0x45, false, 0x13), Some(KEYD_PAUSE));
        // PrtScn: vk 0x2C, scancode E0 0x37.
        assert_eq!(scancode_to_keyd(0x37, true, 0x2C), Some(KEYD_SYSRQ));
    }

    #[test]
    fn nav_cluster_extended_codes() {
        use crate::keys::*;
        for (scan, expect) in [
            (0x47u16, KEYD_HOME), (0x48, KEYD_UP), (0x49, KEYD_PAGEUP),
            (0x4B, KEYD_LEFT), (0x4D, KEYD_RIGHT), (0x4F, KEYD_END),
            (0x50, KEYD_DOWN), (0x51, KEYD_PAGEDOWN), (0x52, KEYD_INSERT),
            (0x53, KEYD_DELETE), (0x5B, KEYD_LEFTMETA), (0x5C, KEYD_RIGHTMETA),
            (0x1D, KEYD_RIGHTCTRL), (0x38, KEYD_RIGHTALT),
        ] {
            assert_eq!(scancode_to_keyd(scan, true, 0), Some(expect));
        }
        // The same scancodes WITHOUT the extended flag are keypad/main keys.
        assert_eq!(scancode_to_keyd(0x47, false, 0), Some(KEYD_KP7));
        assert_eq!(scancode_to_keyd(0x1D, false, 0), Some(KEYD_LEFTCTRL));
    }

    #[test]
    fn media_keys_inject_by_vk() {
        use crate::keys::*;
        for code in [KEYD_MUTE, KEYD_VOLUMEDOWN, KEYD_VOLUMEUP,
                     KEYD_NEXTSONG, KEYD_PREVIOUSSONG, KEYD_PLAYPAUSE, KEYD_STOPCD] {
            assert!(keyd_to_vk(code).is_some(), "media key {code} missing VK mapping");
        }
    }

    #[test]
    fn modifiers_detected() {
        use crate::keys::*;
        for code in [KEYD_LEFTCTRL, KEYD_RIGHTCTRL, KEYD_LEFTSHIFT, KEYD_RIGHTSHIFT,
                     KEYD_LEFTALT, KEYD_RIGHTALT, KEYD_LEFTMETA, KEYD_RIGHTMETA] {
            assert!(is_modifier_keyd(code));
        }
        assert!(!is_modifier_keyd(KEYD_A));
        assert!(!is_modifier_keyd(KEYD_CAPSLOCK)); // capslock repeats like a normal key
    }
}
