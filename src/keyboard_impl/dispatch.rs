use crate::config::*;
use crate::keyboard_types::*;

impl Keyboard {
    pub(super) fn resolve_descriptor(&self, code: u8) -> (Descriptor, i32) {
        if code >= crate::keys::KEYD_CHORD_1 {
            let slot = (code - crate::keys::KEYD_CHORD_1) as usize;
            if slot < self.active_chords.len() && self.active_chords[slot].active != 0 {
                return (self.active_chords[slot].chord.d, self.active_chords[slot].layer);
            }
        }

        // Single-pass max-scan over active layers, mirroring keyd's
        // lookup_descriptor: the most recently activated layer wins, with
        // ties going to the highest layer index (>= comparison).
        let mut best: Option<usize> = None;
        let mut max_ts: i64 = 0;
        for i in 0..self.config.layers.len() {
            if self.layer_state[i].active == 0 {
                continue;
            }
            let d = &self.config.layers[i].keymap[code as usize];
            let bound = d.op != Op::KeySequence
                || matches!(d.data, DescriptorData::KeySequence(ref ks) if ks.code != 0);
            if bound && self.layer_state[i].activation_time >= max_ts {
                max_ts = self.layer_state[i].activation_time;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            return (self.config.layers[i].keymap[code as usize], i as i32);
        }

        let main_idx = 0;
        (self.config.layers[main_idx].keymap[code as usize], main_idx as i32)
    }

    /// Process a batch of key events and return the milliseconds until the next pending timeout,
    /// or -1 if no timeout is pending. Pass a single synthetic event with `code = 0` to tick
    /// timeouts without any key input.
    pub fn kbd_process_events<O: Output>(&mut self, output: &mut O, events: &[KeyEvent]) -> i64 {
        let mut i = 0;
        let mut time: i64 = events.first().map_or(0, |e| e.timestamp as i64);

        while i < events.len() {
            let ev = &events[i];
            let ev_ts = ev.timestamp as i64;

            let timeout = self.calculate_main_loop_timeout(time);

            if timeout > 0 && time + timeout <= ev_ts {
                time += timeout;
                self.process_event(output, 0, 0, time);
            } else {
                time = ev_ts;
                self.process_event(output, ev.code, ev.pressed, time);
                i += 1;
            }
        }

        self.calculate_main_loop_timeout(time)
    }

    pub(super) fn process_event<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) -> i64 {
        if self.handle_chord(output, code, pressed, time) {
            return self.calculate_main_loop_timeout(time);
        }

        self.handle_pending_timeout(output, code, pressed, time);

        if self.handle_pending_overload(output, code, pressed, time) {
            return self.calculate_main_loop_timeout(time);
        }

        if self.oneshot_timeout != 0 && time >= self.oneshot_timeout {
            self.clear_oneshot(output);
            self.update_mods(output, -1, 0);
        }

        if self.macro_play.active_idx.is_some() {
            if code != 0 {
                self.macro_play.active_idx = None;
                self.update_mods(output, -1, 0);
            } else if time >= self.macro_play.timeout {
                self.play_macro_step(output, time);
            }
        }

        if code != 0 {
            if pressed != 0 {
                if self.cache_get(code).is_some() {
                    return self.calculate_main_loop_timeout(time);
                }
                let (d, layer) = self.resolve_descriptor(code);
                self.cache_set(code, Some(CacheEntry { code, d, dl: layer, layer }));
                self.execute_descriptor(output, d, code, layer, pressed, time);
            } else if let Some(entry) = self.cache_get(code) {
                let d = entry.d;
                let layer = entry.layer;
                self.cache_set(code, None);
                self.execute_descriptor(output, d, code, layer, pressed, time);
            }
        }

        self.calculate_main_loop_timeout(time)
    }
}
