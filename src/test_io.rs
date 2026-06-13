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

    const OVERLOADI_CONFIG: &str = "[ids]\n*\n\n[main]\nspace = overloadi(a, b, 200)\n";

    #[test]
    fn test_overloadi_passthrough_release_resets_idle_clock() {
        // Holding a passthrough key, releasing it, then quickly pressing the
        // overloadi key counts as active typing: the release (not just the
        // press) resets the idle clock, matching keyd.
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, OVERLOADI_CONFIG).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_C,     pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_C,     pressed: 0, timestamp: 1000 }, // long hold
            KeyEvent { code: KEYD_SPACE, pressed: 1, timestamp: 1100 }, // 100ms after release
            KeyEvent { code: KEYD_SPACE, pressed: 0, timestamp: 1150 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_A), "idle measured from the release (100ms < 200ms) → action1 (a)");
        assert!(!codes.contains(&KEYD_B), "idle action must not fire right after a key release");
    }

    #[test]
    fn test_overloadi_resolves_idle_action_after_timeout() {
        let mut cfg = Config::new();
        config_parse_string(&mut cfg, OVERLOADI_CONFIG).unwrap();
        let mut kbd = Keyboard::new(cfg);
        let mut output = TestOutput::new();

        let events = [
            KeyEvent { code: KEYD_C,     pressed: 1, timestamp: 0 },
            KeyEvent { code: KEYD_C,     pressed: 0, timestamp: 1000 },
            KeyEvent { code: KEYD_SPACE, pressed: 1, timestamp: 5000 }, // 4000ms idle
            KeyEvent { code: KEYD_SPACE, pressed: 0, timestamp: 5050 },
        ];
        kbd.kbd_process_events(&mut output, &events);

        let codes: Vec<u8> = output.events.iter().map(|e| e.code).collect();
        assert!(codes.contains(&KEYD_B), "4000ms idle ≥ 200ms timeout → action2 (b)");
        assert!(!codes.contains(&KEYD_A), "active-typing action must not fire after idle timeout");
    }
}
