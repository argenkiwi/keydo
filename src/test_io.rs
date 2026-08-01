use crate::config::*;
use crate::config_impl::*;
use crate::keyboard_types::*;

#[derive(Default)]
pub struct TestOutput {
    pub events: Vec<KeyEvent>,
}

impl TestOutput {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Output for TestOutput {
    fn send_key(&mut self, code: u8, state: u8) {
        self.events.push(KeyEvent { code, pressed: state, timestamp: 0 });
    }
    fn on_layer_change(&mut self, _kbd: &Keyboard, _layer_idx: usize, _active: u8) {}
}

/// Output sink that only counts, for the throughput harness — an unbounded
/// `Vec` push would otherwise dominate the measurement.
#[derive(Default)]
pub struct CountingOutput {
    pub keys: u64,
    pub layer_changes: u64,
}

impl Output for CountingOutput {
    fn send_key(&mut self, _code: u8, _state: u8) {
        self.keys += 1;
    }
    fn on_layer_change(&mut self, _kbd: &Keyboard, _layer_idx: usize, _active: u8) {
        self.layer_changes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::*;

    #[test]
    fn kbd_remaps_key_to_configured_target() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = b\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A, pressed: 0, timestamp: 0 },
        ];

        kbd.kbd_process_events(&mut output, &events);

        for event in &output.events {
            println!("Output Event: code={}, pressed={}", event.code, event.pressed);
        }

        assert_eq!(output.events.len(), 2);
        assert_eq!(output.events[0].code, KEYD_B);
        assert_eq!(output.events[0].pressed, 1);
        assert_eq!(output.events[1].code, KEYD_B);
        assert_eq!(output.events[1].pressed, 0);
    }

    #[test]
    fn test_layer_switching() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\ncapslock = layer(nav)\n\n[nav]\nh = left\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H, pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 0 },
        ];

        kbd.kbd_process_events(&mut output, &events);

        assert_eq!(output.events.len(), 2);
        assert_eq!(output.events[0].code, KEYD_LEFT);
        assert_eq!(output.events[0].pressed, 1);
        assert_eq!(output.events[1].code, KEYD_LEFT);
        assert_eq!(output.events[1].pressed, 0);
    }

    #[test]
    fn test_toggle_layer() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = toggle(nav)\n\n[nav]\nh = left\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // First tap: toggle nav ON → h produces left
        // Second tap: toggle nav OFF → h produces passthrough h
        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 0 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // First h → left (nav active)
        assert_eq!(output.events[0].code, KEYD_LEFT);
        assert_eq!(output.events[0].pressed, 1);
        assert_eq!(output.events[1].code, KEYD_LEFT);
        assert_eq!(output.events[1].pressed, 0);

        // Second h → h (nav deactivated)
        assert_eq!(output.events[2].code, KEYD_H);
        assert_eq!(output.events[2].pressed, 1);
        assert_eq!(output.events[3].code, KEYD_H);
        assert_eq!(output.events[3].pressed, 0);
    }

    #[test]
    fn test_default_modifier_remapping() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_LEFTSHIFT, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A, pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_LEFTSHIFT, pressed: 0, timestamp: 0 },
        ];

        kbd.kbd_process_events(&mut output, &events);

        // Check if LEFTSHIFT was sent when A was pressed
        let shift_pressed = output.events.iter().any(|ev| ev.code == KEYD_LEFTSHIFT && ev.pressed == 1);
        assert!(shift_pressed);
        
        let a_pressed = output.events.iter().any(|ev| ev.code == KEYD_A && ev.pressed == 1);
        assert!(a_pressed);
    }

    #[test]
    fn test_clear_op() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = toggle(nav)\nx = clear\n\n[nav]\nh = left\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // Toggle nav on, then clear it, then h should be passthrough
        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_X,        pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_X,        pressed: 0, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 0 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // h should be passthrough (nav was cleared)
        let h_events: Vec<_> = output.events.iter().filter(|e| e.code == KEYD_H).collect();
        assert!(!h_events.is_empty(), "expected h key events after clear");
        assert_eq!(h_events[0].code, KEYD_H);
    }

    // ── Phase 9: macros ───────────────────────────────────────────────────────

    #[test]
    fn test_macro_types_hello() {
        // a = macro(hello) should emit h, e, l, l, o key presses
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = macro(hello)\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A, pressed: 0, timestamp: 10 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let down: Vec<u8> = output.events.iter()
            .filter(|e| e.pressed != 0)
            .map(|e| e.code)
            .collect();

        assert!(down.contains(&KEYD_H), "macro should emit h");
        assert!(down.contains(&KEYD_E), "macro should emit e");
        assert!(down.contains(&KEYD_L), "macro should emit l");
        assert!(down.contains(&KEYD_O), "macro should emit o");
    }

    #[test]
    fn test_macro_simple_key_sequence() {
        // Single-keysequence macro shortcut path: a = C-c should emit ctrl+c
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = macro(C-c)\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A, pressed: 0, timestamp: 10 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_LEFTCTRL), "C-c macro should press ctrl");
        assert!(codes.contains(&KEYD_C), "C-c macro should press c");
    }

    // ── Phase 8: chords ───────────────────────────────────────────────────────

    #[test]
    fn test_chord_fires_when_both_keys_pressed() {
        // j+k = esc: pressing j then k within chord_interkey_timeout → escape
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nj+k = esc\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // Both keys pressed within default chord_interkey_timeout (50ms).
        let events = [
            KeyEvent { code: KEYD_J, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_K, pressed: 1, timestamp: 10 },
            KeyEvent { code: KEYD_J, pressed: 0, timestamp: 20 },
            KeyEvent { code: KEYD_K, pressed: 0, timestamp: 20 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_ESC), "j+k chord should produce escape");
        assert!(!codes.contains(&KEYD_J), "j must not appear as individual key");
        assert!(!codes.contains(&KEYD_K), "k must not appear as individual key");
    }

    #[test]
    fn test_chord_aborts_on_release_before_complete() {
        // j+k = esc: releasing j before k is pressed → j and k fire individually
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nj+k = esc\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // j pressed then released without k → abort, j fires normally
        let events = [
            KeyEvent { code: KEYD_J, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_J, pressed: 0, timestamp: 10 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_J), "j must fire normally on abort");
        assert!(!codes.contains(&KEYD_ESC), "escape must not fire when chord is incomplete");
    }

    #[test]
    fn test_chord_aborts_on_interkey_timeout() {
        // j+k = esc: k arrives after chord_interkey_timeout → abort, j fires first
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nj+k = esc\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // k arrives at t=100, past the default interkey_timeout of 50ms.
        // kbd_process_events will inject the timeout tick at t=50 first, aborting the chord.
        let events = [
            KeyEvent { code: KEYD_J, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_K, pressed: 1, timestamp: 100 },
            KeyEvent { code: KEYD_J, pressed: 0, timestamp: 100 },
            KeyEvent { code: KEYD_K, pressed: 0, timestamp: 100 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_J), "j must fire as individual key after timeout");
        assert!(!codes.contains(&KEYD_ESC), "escape must not fire after interkey timeout");
    }

    #[test]
    fn test_fourth_simultaneous_chord_does_not_alias_real_keycode() {
        // 4 disjoint chords exhaust the 3 real chord slots; the 4th must fall
        // back to its raw keys, and a genuine playcd press afterward must
        // resolve to its own binding rather than the failed 4th chord's slot.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nj+k = a\nh+l = b\nu+i = c\no+p = d\nplaycd = z\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_J, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_K, pressed: 1, timestamp: 10 },   // chord1 -> slot 0 (197)
            KeyEvent { code: KEYD_H, pressed: 1, timestamp: 20 },
            KeyEvent { code: KEYD_L, pressed: 1, timestamp: 30 },   // chord2 -> slot 1 (198)
            KeyEvent { code: KEYD_U, pressed: 1, timestamp: 40 },
            KeyEvent { code: KEYD_I, pressed: 1, timestamp: 50 },   // chord3 -> slot 2 (199), slots full
            KeyEvent { code: KEYD_O, pressed: 1, timestamp: 60 },
            KeyEvent { code: KEYD_P, pressed: 1, timestamp: 70 },   // chord4 match, no free slot -> raw passthrough
            KeyEvent { code: KEYD_PLAYCD, pressed: 1, timestamp: 80 }, // real hardware playcd key
            KeyEvent { code: KEYD_PLAYCD, pressed: 0, timestamp: 90 },
        ];
        kbd.kbd_process_events(&mut output, &events);
        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();

        assert!(codes.contains(&KEYD_A), "chord1 (j+k) should fire");
        assert!(codes.contains(&KEYD_B), "chord2 (h+l) should fire");
        assert!(codes.contains(&KEYD_C), "chord3 (u+i) should fire");
        assert!(!codes.contains(&KEYD_D), "chord4 must not resolve once all chord slots are exhausted");
        assert!(codes.contains(&KEYD_O), "o must fall back to its own binding when no chord slot is free");
        assert!(codes.contains(&KEYD_P), "p must fall back to its own binding when no chord slot is free");
        assert!(codes.contains(&KEYD_Z), "real playcd key must resolve to its own binding, not an aliased chord");
    }

    // ── Phase 7: timeout ──────────────────────────────────────────────────────

    #[test]
    fn test_timeout_tap_fires_action1() {
        // x = timeout(a, 200, layer(nav)): quick tap → 'a'
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nx = timeout(a, 200, layer(nav))\n\n[nav]\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // Press and release x in the same tick (spontaneous). Then press b so the
        // timeout resolver sees an event and fires action1.
        let events = [
            KeyEvent { code: KEYD_X, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_X, pressed: 0, timestamp: 0 }, // same tick → spontaneous
            KeyEvent { code: KEYD_B, pressed: 1, timestamp: 50 }, // triggers resolution
            KeyEvent { code: KEYD_B, pressed: 0, timestamp: 50 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_A), "tap should produce 'a' (action1)");
        assert!(!codes.contains(&KEYD_LEFT), "nav layer must not activate on tap");
    }

    #[test]
    fn test_timeout_hold_fires_action2() {
        // x = timeout(a, 200, layer(nav)): hold past deadline → layer(nav), h→left
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\nx = timeout(a, 200, layer(nav))\n\n[nav]\nh = left\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // h arrives at t=300, past the 200ms deadline; kbd_process_events injects a
        // synthetic timeout tick at t=200 before processing h.
        let events = [
            KeyEvent { code: KEYD_X, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_H, pressed: 1, timestamp: 300 },
            KeyEvent { code: KEYD_H, pressed: 0, timestamp: 300 },
            KeyEvent { code: KEYD_X, pressed: 0, timestamp: 350 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_LEFT), "hold should activate nav → h=left");
        assert!(!codes.contains(&KEYD_A), "'a' must not appear on hold");
    }

    // ── Phase 6: oneshot ──────────────────────────────────────────────────────

    #[test]
    fn test_oneshot_tap_shifts_one_key() {
        // capslock = oneshot(shift): only the first key after the tap is shifted.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = oneshot(shift)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            // Tap the oneshot key.
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 10 },
            // Press A — should be shifted.
            KeyEvent { code: KEYD_A,        pressed: 1, timestamp: 20 },
            KeyEvent { code: KEYD_A,        pressed: 0, timestamp: 30 },
            // Press B — should NOT be shifted (oneshot consumed).
            KeyEvent { code: KEYD_B,        pressed: 1, timestamp: 40 },
            KeyEvent { code: KEYD_B,        pressed: 0, timestamp: 50 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // Shift must be down before A and up after A.
        let shift_downs: Vec<_> = output.events.iter()
            .enumerate()
            .filter(|(_, e)| e.code == KEYD_LEFTSHIFT && e.pressed != 0)
            .collect();
        assert_eq!(shift_downs.len(), 1, "shift should go down exactly once");

        // A must come after the shift-down.
        let a_down_idx = output.events.iter().position(|e| e.code == KEYD_A && e.pressed != 0)
            .expect("A down must be emitted");
        let shift_down_idx = shift_downs[0].0;
        assert!(shift_down_idx < a_down_idx, "shift must precede A");

        // B must not be preceded by a shift in this batch.
        let b_down_idx = output.events.iter().position(|e| e.code == KEYD_B && e.pressed != 0)
            .expect("B down must be emitted");
        let shift_after_a: bool = output.events[a_down_idx + 1..b_down_idx]
            .iter().any(|e| e.code == KEYD_LEFTSHIFT && e.pressed != 0);
        assert!(!shift_after_a, "shift must not re-appear before B");
    }

    #[test]
    fn test_oneshot_hold_acts_as_regular_modifier() {
        // Holding the oneshot key while pressing A: acts as a regular modifier (not oneshot).
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = oneshot(shift)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            // A while capslock still held.
            KeyEvent { code: KEYD_A,        pressed: 1, timestamp: 10 },
            KeyEvent { code: KEYD_A,        pressed: 0, timestamp: 20 },
            // Release capslock.
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 30 },
            // B — should NOT be shifted.
            KeyEvent { code: KEYD_B,        pressed: 1, timestamp: 40 },
            KeyEvent { code: KEYD_B,        pressed: 0, timestamp: 50 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        assert!(output.events.iter().any(|e| e.code == KEYD_LEFTSHIFT), "shift must appear (modifier hold)");
        assert!(output.events.iter().any(|e| e.code == KEYD_A), "A must be emitted");

        // After capslock release and A, B must not be shifted.
        let b_down_idx = output.events.iter().position(|e| e.code == KEYD_B && e.pressed != 0)
            .expect("B down must be emitted");
        let shift_up_before_b = output.events[..b_down_idx]
            .iter().rev()
            .find(|e| e.code == KEYD_LEFTSHIFT);
        if let Some(last_shift) = shift_up_before_b {
            assert_eq!(last_shift.pressed, 0, "shift must be released before B");
        }
    }

    #[test]
    fn test_oneshot_timeout_clears() {
        // oneshot_timeout: if the next key isn't pressed quickly enough, cancel oneshot.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[global]\noneshot_timeout = 100\n\n[main]\ncapslock = oneshot(shift)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 10 },
            // A arrives after the timeout — should NOT be shifted.
            KeyEvent { code: KEYD_A,        pressed: 1, timestamp: 200 },
            KeyEvent { code: KEYD_A,        pressed: 0, timestamp: 210 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // Shift must not be active when A is pressed (timeout already fired).
        let a_down_idx = output.events.iter().position(|e| e.code == KEYD_A && e.pressed != 0)
            .expect("A must be emitted");
        let shift_at_a = output.events[..a_down_idx]
            .iter().rev()
            .find(|e| e.code == KEYD_LEFTSHIFT);
        if let Some(last_shift) = shift_at_a {
            assert_eq!(last_shift.pressed, 0, "shift must be released before A (timeout fired)");
        }
    }

    #[test]
    fn test_overload_tap() {
        // capslock = overload(control, esc): quick tap → ESC
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = overload(control, esc)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 50 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // The control layer activates briefly on press (leftctrl momentarily down/up),
        // then ESC fires on tap release — matching C's overload() behaviour.
        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_ESC), "tap should produce ESC");
        let esc_down = output.events.iter().find(|e| e.code == KEYD_ESC && e.pressed != 0);
        assert!(esc_down.is_some(), "ESC press event must be present");
    }

    #[test]
    fn test_overload_hold() {
        // capslock = overload(control, esc): hold while pressing a → C-a
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = overload(control, esc)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_A,        pressed: 1, timestamp: 10 },
            KeyEvent { code: KEYD_A,        pressed: 0, timestamp: 20 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 30 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        // Control modifier should have been pressed before A.
        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_LEFTCTRL), "control should be pressed during hold");
        assert!(codes.contains(&KEYD_A),        "a should be emitted");
        // ESC must NOT appear (no tap fired).
        assert!(!codes.contains(&KEYD_ESC), "esc must not fire on hold");
    }

    #[test]
    fn test_overloadt_tap() {
        // capslock = overloadt(nav, a, 200): released within 200ms → tap action 'a'
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = overloadt(nav, a, 200)\n\n[nav]\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // Release well before the 200ms deadline.
        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 50 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let sent: Vec<_> = output.events.iter().map(|e| e.code).collect();
        assert!(sent.contains(&KEYD_A), "tap should produce 'a'");
    }

    #[test]
    fn test_overloadt_timeout() {
        // capslock = overloadt(nav, a, 200): timeout fires → layer activated
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n[main]\ncapslock = overloadt(nav, a, 200)\n\n[nav]\nh = left\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // Synthetic timeout tick at t=200, then h while layer active, then capslock release.
        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 },
            // Simulate timeout expiry via synthetic event (code=0 via kbd_process_events).
            // Instead, drive it directly with the timeout injection in kbd_process_events
            // by having the next real event arrive after the deadline.
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 300 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 300 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 350 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let sent: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        // The timeout injection in kbd_process_events will fire at t=200,
        // resolving to layer(nav). Then h → left.
        assert!(sent.contains(&KEYD_LEFT), "after timeout, h should produce left in nav layer");
        assert!(!sent.contains(&KEYD_A),   "tap action must not fire on timeout");
    }

    #[test]
    fn test_macro_non_blocking_timeouts() {
        // a = macro(h 100ms e)
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = macro(h 100ms e)\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // 1. Press A: should fire 'h' and schedule a timeout.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 }]);
        
        let codes: Vec<u8> = output.events.iter().filter(|e| e.pressed != 0).map(|e| e.code).collect();
        assert_eq!(codes, vec![KEYD_H]);
        output.events.clear();

        // 2. Advance time past 100ms: should fire 'e'.
        // kbd_process_events with an empty list at t=150 should trigger the pending timeout.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: 0, pressed: 0, timestamp: 150 }]);
        
        let codes: Vec<u8> = output.events.iter().filter(|e| e.pressed != 0).map(|e| e.code).collect();
        assert_eq!(codes, vec![KEYD_E]);
    }

    #[test]
    fn test_macro_cancellation_on_interleaved_key() {
        // a = macro(h 100ms e)
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = macro(h 100ms e)\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // 1. Start macro.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 }]);
        output.events.clear();

        // 2. Press B before macro finishes.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: KEYD_B, pressed: 1, timestamp: 50 }]);
        
        // 3. Advance time past 100ms: macro should NOT fire 'e' because it was canceled.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: 0, pressed: 0, timestamp: 150 }]);
        
        let e_fired = output.events.iter().any(|e| e.code == KEYD_E);
        assert!(!e_fired, "macro should have been canceled by key B");
    }

    #[test]
    fn test_macro_repeat() {
        // a = macro(h 50ms) with 100ms repeat interval
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[global]\nmacro_repeat_timeout = 100\n\n[main]\na = macro(h 50ms)\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        // 1. Start macro. Fires 'h', schedules timeout at 50ms.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: KEYD_A, pressed: 1, timestamp: 0 }]);
        output.events.clear();

        // 2. Tick at 60ms: finishes first run, schedules repeat timeout at 60 + 100 = 160ms.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: 0, pressed: 0, timestamp: 60 }]);
        output.events.clear();

        // 3. Tick at 170ms: starts second run, fires 'h' again.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: 0, pressed: 0, timestamp: 170 }]);
        let codes: Vec<u8> = output.events.iter().filter(|e| e.pressed != 0).map(|e| e.code).collect();
        assert_eq!(codes, vec![KEYD_H]);
    }

    // ── Layer resolution precedence ───────────────────────────────────────────

    const TWO_LAYER_CONFIG: &str =
        "[ids]\n*\n\n[main]\ncapslock = layer(nav1)\ntab = layer(nav2)\n\n[nav1]\nh = left\n\n[nav2]\nh = right\n";

    #[test]
    fn test_most_recent_layer_wins() {
        // nav2 (higher index) activated first, nav1 activated later:
        // the most recently activated layer wins regardless of index.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, TWO_LAYER_CONFIG).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_TAB,      pressed: 1, timestamp: 0 },  // nav2 on
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 10 }, // nav1 on (more recent)
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 20 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 20 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 30 },
            KeyEvent { code: KEYD_TAB,      pressed: 0, timestamp: 30 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_LEFT), "nav1 (most recent) should win → left");
        assert!(!codes.contains(&KEYD_RIGHT), "nav2 binding must not fire");
    }

    #[test]
    fn test_layer_tie_breaks_to_later_layer() {
        // Both layers activated at the same timestamp: the higher layer index
        // wins, matching keyd's `activation_time >= maxts` tie-break.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, TWO_LAYER_CONFIG).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: 0 }, // nav1 on
            KeyEvent { code: KEYD_TAB,      pressed: 1, timestamp: 0 }, // nav2 on, same time
            KeyEvent { code: KEYD_H,        pressed: 1, timestamp: 10 },
            KeyEvent { code: KEYD_H,        pressed: 0, timestamp: 10 },
            KeyEvent { code: KEYD_TAB,      pressed: 0, timestamp: 20 },
            KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: 20 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_RIGHT), "tie should go to the higher-index layer (nav2) → right");
        assert!(!codes.contains(&KEYD_LEFT), "nav1 binding must not fire on a tie");
    }

    // ── Cache entry `layer` sentinel (swap-target hijack regression) ─────────
    //
    // Reduced from a randomized-fuzz repro against the real kenkyo config.
    // Before the fix, `process_event` cached a key-down's *lookup* layer
    // (the layer it was resolved against) instead of a `0` sentinel — only
    // `activate_layer` is supposed to stamp a cache entry with a real layer
    // index, marking it as the key currently holding that layer active.
    // `Op::Swap` searches the cache for the entry whose `layer` field matches
    // the active source layer; with every same-layer keypress tagged that
    // way, swap could hijack an unrelated key's cache slot instead of the
    // layer's true holder, permanently losing that key's release action.

    #[test]
    fn test_swap_does_not_hijack_unrelated_key_cache_entry() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n\
             [main]\n\
             s = overloadi(s, timeout(overloadt2(alt, s, 200), 500, s), 150)\n\
             space = overloadi(space, timeout(overloadt2(extend, space, 200), 500, space), 150)\n\
             k = overloadi(k, timeout(overloadt2(shift, k, 200), 500, k), 150)\n\n\
             [extend]\n\
             e = swap(shift)\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_S,     pressed: 1, timestamp: 1130 },
            KeyEvent { code: KEYD_SPACE, pressed: 1, timestamp: 1135 },
            KeyEvent { code: KEYD_K,     pressed: 1, timestamp: 1165 },
            KeyEvent { code: KEYD_S,     pressed: 0, timestamp: 1915 },
            KeyEvent { code: KEYD_E,     pressed: 1, timestamp: 2655 },
            KeyEvent { code: KEYD_SPACE, pressed: 0, timestamp: 4310 },
            KeyEvent { code: KEYD_K,     pressed: 0, timestamp: 4400 },
        ];
        kbd.kbd_process_events(&mut output, &events);
        // Give any pending timeouts a chance to resolve.
        kbd.kbd_process_events(&mut output, &[KeyEvent { code: 0, pressed: 0, timestamp: 8000 }]);

        for i in 0..256usize {
            assert_eq!(kbd.keystate[i], 0, "output key {i} left stuck down");
        }
        for i in 1..kbd.config.layers.len() {
            let ls = &kbd.layer_state[i];
            assert_eq!(ls.active, 0, "layer '{}' left active", kbd.config.layers[i].name);
        }
    }

    // ── has_chords fast-path guard ───────────────────────────────────────────
    //
    // `handle_chord` short-circuits on `!has_chords`, which is computed once in
    // `Keyboard::new`. That is only sound because `nr_chords` is written solely
    // by the parse-time `set_layer_entry` — the IPC `bind` path
    // (`config_add_entry`) touches `keymap` only, and `reload` builds fresh
    // `Keyboard`s. If chords ever become addable at runtime, this breaks.

    #[test]
    fn has_chords_reflects_parsed_config() {
        let mut without = Config::new();
        config_parse_string(&mut without, "[ids]\n*\n\n[main]\na = b\n").unwrap();
        assert!(!Keyboard::new(without).has_chords);

        let mut with = Config::new();
        config_parse_string(&mut with, "[ids]\n*\n\n[main]\nd+f = esc\n").unwrap();
        assert!(Keyboard::new(with).has_chords);
    }

    #[test]
    fn ipc_bind_cannot_introduce_chords_behind_has_chords() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = b\n").unwrap();
        let mut kbd = Keyboard::new(cfg);
        assert!(!kbd.has_chords);

        crate::config_impl::config_add_entry(&mut kbd.config, "main.c = d").unwrap();

        let total_chords: usize = kbd.config.layers.iter().map(|l| l.nr_chords).sum();
        assert_eq!(total_chords, 0, "bind added a chord; has_chords is now stale");
    }

    #[test]
    #[should_panic(expected = "layer_state is fixed at")]
    fn keyboard_new_rejects_config_with_more_layers_than_layer_state_can_hold() {
        // config_parse_string (used by every test and any future fuzz harness)
        // never calls config_validate::validate() — only the file-loading
        // config_parse does. So a hand-built or otherwise-unvalidated Config
        // with too many layers can reach Keyboard::new directly; this must
        // panic loudly in debug builds rather than silently indexing
        // layer_state out of bounds later.
        let mut config = Config::new();
        for i in 0..=MAX_LAYERS {
            config.layers.push(Layer::new(format!("layer_{i}")));
        }
        Keyboard::new(config);
    }

    // ── Throughput harness ───────────────────────────────────────────────────
    //
    // Not a correctness test — a repeatable before/after number for changes to
    // the state machine hot path. Run it with:
    //
    //     cargo test --release -- --ignored --nocapture
    //
    // Release mode matters: the debug build is dominated by bounds checks and
    // unelided copies and will not reflect what the daemon actually runs.

    #[test]
    #[ignore = "timing harness, not a correctness check"]
    fn bench_kbd_process_events_throughput() {
        const ITERATIONS: usize = 250_000; // 4 events each → 1M events

        let mut cfg = Config::new();
        config_parse_string(&mut cfg,
            "[ids]\n*\n\n\
             [main]\n\
             a = b\n\
             capslock = layer(nav)\n\
             s = overload(nav, s)\n\n\
             [nav]\n\
             h = left\n\
             j = down\n"
        ).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = CountingOutput::default();

        // A plain remap plus a held layer with a key pressed inside it — the
        // common mix, exercising resolve_descriptor, the cache, update_mods
        // and the layer bookkeeping.
        let start = std::time::Instant::now();
        let mut ts: i32 = 0;
        for _ in 0..ITERATIONS {
            let events = [
                KeyEvent { code: KEYD_A,        pressed: 1, timestamp: ts },
                KeyEvent { code: KEYD_A,        pressed: 0, timestamp: ts + 1 },
                KeyEvent { code: KEYD_CAPSLOCK, pressed: 1, timestamp: ts + 2 },
                KeyEvent { code: KEYD_CAPSLOCK, pressed: 0, timestamp: ts + 3 },
            ];
            kbd.kbd_process_events(&mut output, &events);
            ts += 4;
        }
        let elapsed = start.elapsed();

        let n_events = (ITERATIONS * 4) as u32;
        println!(
            "kbd_process_events: {n_events} events in {elapsed:?} \
             ({:.1} ns/event, {} keys out, {} layer changes)",
            elapsed.as_secs_f64() * 1e9 / f64::from(n_events),
            output.keys,
            output.layer_changes,
        );

        // Guard against the harness silently measuring nothing.
        assert!(output.keys > 0, "harness produced no output");
    }
}

// ── Property-based fuzzing ───────────────────────────────────────────────────
//
// Replaces the ad-hoc, throwaway fuzzer (never committed — see notes/
// 20260713215804.md) that originally found the swap-cache-hijack and
// chord-slot-aliasing bugs. Each config below reproduces the *shape* of one
// historical bug or a currently-unguarded capacity edge; the invariants
// checked after every run are the same ones the workstream-B debug_asserts
// enforce, so a regression in either the asserts or the state machine itself
// shows up here as a shrinking, reproducible failure instead of a silent
// drop.
#[cfg(test)]
mod proptests {
    use super::*;
    use crate::keys::*;
    use proptest::prelude::*;

    /// A bounded sequence of key events over a small alphabet, with
    /// monotonically increasing timestamps (real device events never go
    /// backwards in time).
    fn key_event_seq(codes: Vec<u8>, max_len: usize) -> impl Strategy<Value = Vec<KeyEvent>> {
        prop::collection::vec(
            (proptest::sample::select(codes), any::<bool>(), 1i32..50i32),
            0..max_len,
        )
        .prop_map(|steps| {
            let mut ts = 0i32;
            steps
                .into_iter()
                .map(|(code, pressed, dt)| {
                    ts += dt;
                    KeyEvent { code, pressed: u8::from(pressed), timestamp: ts }
                })
                .collect()
        })
    }

    /// Checks the same capacity/uniqueness invariants the workstream-B
    /// debug_asserts enforce at the write sites, as a second line of defense
    /// that also gives proptest a clean value to shrink against (rather than
    /// relying solely on a panic partway through processing).
    fn assert_capacity_invariants(kbd: &Keyboard) {
        let mut seen = std::collections::HashSet::new();
        for entry in kbd.cache.iter().flatten() {
            assert!(seen.insert(entry.code), "duplicate code {} aliased in cache", entry.code);
        }
        assert!(
            kbd.chord.queue_sz <= CHORD_QUEUE_LEN,
            "chord queue_sz {} exceeded its {CHORD_QUEUE_LEN}-slot bound",
            kbd.chord.queue_sz
        );
        assert!(
            kbd.nr_timeouts <= TIMEOUT_TABLE_SIZE,
            "nr_timeouts {} exceeded its {TIMEOUT_TABLE_SIZE}-slot bound",
            kbd.nr_timeouts
        );
    }

    /// Builds a config with `MAX_LAYERS - 1` non-main layers, each toggled by
    /// a distinct key on `main`, plus the codes of those trigger keys.
    fn many_layers_config() -> (String, Vec<u8>) {
        const ALPHABET: [(&str, u8); 36] = [
            ("q", KEYD_Q), ("w", KEYD_W), ("e", KEYD_E), ("r", KEYD_R), ("t", KEYD_T),
            ("y", KEYD_Y), ("u", KEYD_U), ("i", KEYD_I), ("o", KEYD_O), ("p", KEYD_P),
            ("a", KEYD_A), ("s", KEYD_S), ("d", KEYD_D), ("f", KEYD_F), ("g", KEYD_G),
            ("h", KEYD_H), ("j", KEYD_J), ("k", KEYD_K), ("l", KEYD_L), ("z", KEYD_Z),
            ("x", KEYD_X), ("c", KEYD_C), ("v", KEYD_V), ("b", KEYD_B), ("n", KEYD_N),
            ("m", KEYD_M), ("1", KEYD_1), ("2", KEYD_2), ("3", KEYD_3), ("4", KEYD_4),
            ("5", KEYD_5), ("6", KEYD_6), ("7", KEYD_7), ("8", KEYD_8), ("9", KEYD_9),
            ("0", KEYD_0),
        ];
        // config_parse_string always adds 6 built-in layers on top of the
        // user-declared ones: main plus 5 modifier-tracking layers (control,
        // shift, alt, meta, altgr) — see config_impl.rs's config_parse_string.
        let n = MAX_LAYERS - 6;
        let mut main = String::from("[ids]\n*\n\n[main]\n");
        let mut layers = String::new();
        let mut codes = Vec::new();
        for &(name, code) in &ALPHABET[..n] {
            main.push_str(&format!("{name} = layer(layer_{name})\n"));
            layers.push_str(&format!("[layer_{name}]\nesc = esc\n\n"));
            codes.push(code);
        }
        (format!("{main}\n{layers}"), codes)
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

        /// Fuzzes the chord queue/resolve/abort state machine with the same
        /// shape of alphabet as the historical chord-slot-aliasing bug (4+
        /// potential simultaneous chords plus a real hardware key, playcd,
        /// immediately past the reserved chord range) — but note this checks
        /// only capacity/uniqueness invariants, not output semantics, so it
        /// cannot by itself catch that specific bug (a wrong output, not a
        /// panic or capacity violation). That bug class — MAX_ACTIVE_CHORDS
        /// hand-edited back to a value larger than the reserved keycode
        /// range — is instead guarded by the `const _: () = assert!(...)` in
        /// keyboard_types.rs, which fails the *build* outright; a runtime
        /// property test cannot catch "the build itself is wrong." This test
        /// still earns its keep by fuzzing the chord queue/state transitions
        /// themselves for other, not-yet-known bugs in a correctly-built
        /// binary.
        #[test]
        fn chord_config_never_panics(events in key_event_seq(
            vec![KEYD_J, KEYD_K, KEYD_H, KEYD_L, KEYD_U, KEYD_I, KEYD_O, KEYD_P, KEYD_PLAYCD],
            40,
        )) {
            let mut cfg = Config::new();
            config_parse_string(&mut cfg,
                "[ids]\n*\n\n[main]\nj+k = a\nh+l = b\nu+i = c\no+p = d\nplaycd = z\n"
            ).unwrap();
            let mut kbd = Keyboard::new(cfg);
            let mut output = TestOutput::new();
            kbd.kbd_process_events(&mut output, &events);
            assert_capacity_invariants(&kbd);
        }

        /// Mirrors the historical swap-cache-hijack bug: overloads that swap
        /// a modifier in on an `extend` layer.
        #[test]
        fn swap_overload_config_never_panics(events in key_event_seq(
            vec![KEYD_S, KEYD_SPACE, KEYD_K, KEYD_E],
            40,
        )) {
            let mut cfg = Config::new();
            config_parse_string(&mut cfg,
                "[ids]\n*\n\n\
                 [main]\n\
                 s = overloadi(s, timeout(overloadt2(alt, s, 200), 500, s), 150)\n\
                 space = overloadi(space, timeout(overloadt2(extend, space, 200), 500, space), 150)\n\
                 k = overloadi(k, timeout(overloadt2(shift, k, 200), 500, k), 150)\n\n\
                 [extend]\n\
                 e = swap(shift)\n"
            ).unwrap();
            let mut kbd = Keyboard::new(cfg);
            let mut output = TestOutput::new();
            kbd.kbd_process_events(&mut output, &events);
            assert_capacity_invariants(&kbd);
        }

        /// Layer count at MAX_LAYERS - 1 (the largest a valid config can
        /// declare), exercising layer activate/deactivate under random input.
        /// Only a handful of the config's trigger keys are used for the event
        /// alphabet (well under CACHE_SIZE) so this stays targeted at the
        /// layer-count boundary rather than re-triggering the separate,
        /// already-known cache-capacity edge (holding 17+ distinct keys at
        /// once — that's a real but unrelated capacity limit, guarded by the
        /// cache_set debug_assert in dispatch.rs, not something this test
        /// is meant to exercise).
        #[test]
        fn many_layers_config_never_panics(events in key_event_seq(many_layers_config().1[..8].to_vec(), 60)) {
            let (config_text, _) = many_layers_config();
            let mut cfg = Config::new();
            config_parse_string(&mut cfg, &config_text).unwrap();
            let mut kbd = Keyboard::new(cfg);
            let mut output = TestOutput::new();
            kbd.kbd_process_events(&mut output, &events);
            assert_capacity_invariants(&kbd);
        }
    }
}
