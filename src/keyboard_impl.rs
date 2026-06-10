//! Keyboard state machine — processes raw key events through layers, chords, macros, and overloads.

use crate::config::*;
use crate::keyboard_types::*;
use crate::keys::*;

impl Keyboard {
    /// Create a new keyboard instance from a parsed config.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cache: [None; 16],
            last_pressed_output_code: 0,
            last_pressed_code: 0,
            oneshot_latch: 0,
            inhibit_modifier_guard: 0,
            macro_play: MacroPlayState {
                active_idx: None,
                entry_idx: 0,
                hold_start_idx: None,
                is_repeating: false,
                layer: -1,
                timeout: 0,
                repeat_interval: 0,
            },
            overload_last_layer_code: -1,
            oneshot_timeout: 0,
            overload_start_time: 0,
            last_simple_key_time: 0,
            timeouts: [0; 128],
            nr_timeouts: 0,
            active_chords: {
                const E: ActiveChord = ActiveChord {
                    active: 0,
                    chord: Chord {
                        keys: [0; 8], sz: 0,
                        d: Descriptor { op: Op::KeySequence, data: DescriptorData::None },
                    },
                    layer: 0,
                };
                vec![E; 8]
            },
            chord: ChordState {
                queue: [KeyEvent { code: 0, pressed: 0, timestamp: 0 }; 32],
                queue_sz: 0,
                match_idx: None,
                match_layer: -1,
                start_code: 0,
                last_code_time: 0,
                state: ChordStatus::Inactive,
            },
            pending_timeout: None,
            pending_overload: None,
            layer_state: {
                let mut ls = [LayerState { activation_time: 0, active: 0, toggled: 0, oneshot_depth: 0 }; MAX_LAYERS];
                ls[0].active = 1; // main layer (index 0) is always active
                ls
            },
            last_repeatable_action: Descriptor { op: Op::KeySequence, data: DescriptorData::None },
            keystate: [0; 256],
            scroll: ScrollState { x: 0, y: 0, sensitivity: 0, active: 0 },
        }
    }

    fn cache_set(&mut self, code: u8, ent: Option<CacheEntry>) -> bool {
        let mut slot = None;
        for i in 0..16 {
            if let Some(c) = self.cache[i] {
                if c.code == code {
                    slot = Some(i);
                    break;
                }
            } else if slot.is_none() {
                slot = Some(i);
            }
        }

        if let Some(i) = slot {
            if let Some(mut e) = ent {
                e.code = code;
                self.cache[i] = Some(e);
            } else {
                self.cache[i] = None;
            }
            true
        } else {
            false
        }
    }

    fn cache_get(&self, code: u8) -> Option<CacheEntry> {
        for i in 0..16 {
            if let Some(c) = self.cache[i].filter(|c| c.code == code) {
                return Some(c);
            }
        }
        None
    }

    fn clear_oneshot<O: Output>(&mut self, output: &mut O) {
        for i in 0..self.config.layers.len() {
            while self.layer_state[i].oneshot_depth > 0 {
                self.deactivate_layer(output, i);
                self.layer_state[i].oneshot_depth -= 1;
            }
        }
        self.oneshot_latch = 0;
        self.oneshot_timeout = 0;
    }

    fn calculate_main_loop_timeout(&mut self, time: i64) -> i64 {
        // Full implementation in Phase 5 (timeout scheduling).
        // Prune expired timeouts and return ms until next one; 0 = none pending.
        let mut earliest: i64 = 0;
        let mut n = 0;
        for i in 0..self.nr_timeouts {
            if self.timeouts[i] > time {
                if earliest == 0 || self.timeouts[i] < earliest {
                    earliest = self.timeouts[i];
                }
                self.timeouts[n] = self.timeouts[i];
                n += 1;
            }
        }
        self.nr_timeouts = n;
        if earliest > 0 { earliest - time } else { 0 }
    }

    fn schedule_timeout(&mut self, deadline_ms: i64) {
        if self.nr_timeouts < self.timeouts.len() {
            self.timeouts[self.nr_timeouts] = deadline_ms;
            self.nr_timeouts += 1;
        }
    }

    /// Returns `true` if the event was consumed by the pending overload state machine.
    fn handle_pending_overload<O: Output>(
        &mut self, output: &mut O, code: u8, pressed: u8, time: i64,
    ) -> bool {
        if self.pending_overload.is_none() {
            return false;
        }

        // Let through key-up events for keys that were already held *before* the
        // pending overload started (they won't be in the queue and aren't the overload key).
        if code != 0 && pressed == 0 {
            let known = if let Some(po) = self.pending_overload.as_ref() {
                code == po.code
                    || po.queue[..po.queue_sz].iter().any(|e| e.code == code)
            } else {
                return false;
            };
            if !known {
                return false;
            }
        }

        // Enqueue real (non-synthetic) events.
        if code != 0 {
            if let Some(po) = self.pending_overload.as_mut() {
                if po.queue_sz < po.queue.len() {
                    po.queue[po.queue_sz] = KeyEvent { code, pressed, timestamp: time as i32 };
                    po.queue_sz += 1;
                }
            }
        }

        // Decide if we can resolve now.
        let resolve: Option<Descriptor> = if let Some(po) = self.pending_overload.as_ref() {
            if time >= po.expiration {
                Some(po.action2)                               // timeout → hold action
            } else if code == po.code && pressed == 0 {
                Some(po.action1)                               // overload key released → tap
            } else if po.resolve_on_interrupt != 0 && pressed == 0 {
                Some(po.action2)                               // overloadt2: any key-up → hold
            } else {
                None
            }
        } else {
            None
        };

        if let Some(action) = resolve {
            // Snapshot the queue before mutating self.
            let (overload_code, dl, queue_snap, queue_sz) = if let Some(po) = self.pending_overload.as_ref() {
                let sz = po.queue_sz;
                let mut q = [KeyEvent { code: 0, pressed: 0, timestamp: 0 }; 32];
                q[..sz].copy_from_slice(&po.queue[..sz]);
                (po.code, po.dl as i32, q, sz)
            } else {
                return true; // pending_overload was cleared concurrently — nothing to replay
            };

            self.pending_overload = None;

            // Lock in the resolved action for this key's cache entry.
            self.cache_set(overload_code, Some(CacheEntry {
                code: overload_code, d: action, dl, layer: 0,
            }));
            self.execute_descriptor(output, action, overload_code, dl, 1, time);

            // Replay queued events (e.g. the overload key-release and any interleaved keys).
            if queue_sz > 0 {
                self.kbd_process_events(output, &queue_snap[..queue_sz]);
            }
        }

        true
    }

    fn reset_keystate<O: Output>(&mut self, output: &mut O) {
        for i in 0..256 {
            if self.keystate[i] != 0 {
                output.send_key(i as u8, 0);
                self.keystate[i] = 0;
            }
        }
    }

    fn activate_layer<O: Output>(&mut self, output: &mut O, code: u8, idx: usize, time: i64) {
        self.layer_state[idx].activation_time = time;
        self.layer_state[idx].active += 1;
        // Update the cache entry for the activating key so its layer reflects the new layer.
        for i in 0..16 {
            if let Some(ce) = self.cache[i].as_mut().filter(|ce| ce.code == code) {
                ce.layer = idx as i32;
                break;
            }
        }
        output.on_layer_change(self, idx, 1);
    }

    fn deactivate_layer<O: Output>(&mut self, output: &mut O, idx: usize) {
        debug_assert!(self.layer_state[idx].active > 0, "deactivate_layer called on inactive layer {idx}");
        if self.layer_state[idx].active > 0 {
            self.layer_state[idx].active -= 1;
        }
        output.on_layer_change(self, idx, 0);
    }

    fn clear<O: Output>(&mut self, output: &mut O) {
        self.clear_oneshot(output);
        for i in 1..self.config.layers.len() {
            if self.config.layers[i].layer_type != LayerType::Layout && self.layer_state[i].toggled != 0 {
                self.layer_state[i].toggled = 0;
                self.deactivate_layer(output, i);
            }
        }
        self.macro_play.active_idx = None;
        self.reset_keystate(output);
    }

    fn setlayout<O: Output>(&mut self, output: &mut O, idx: usize) {
        self.clear(output);
        for i in 1..self.config.layers.len() {
            if self.config.layers[i].layer_type == LayerType::Layout {
                self.layer_state[i].active = 0;
            }
        }
        // Setting layout to main (idx=0) just clears all other layouts.
        if idx != 0 {
            self.layer_state[idx].activation_time = 1;
            self.layer_state[idx].active = 1;
        }
        output.on_layer_change(self, idx, 1);
    }

    fn send_key<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8) {
        if code == KEYD_NOOP || code == KEYD_EXTERNAL_MOUSE_BUTTON {
            return;
        }

        if pressed != 0 {
            self.last_pressed_output_code = code;
        }

        if self.keystate[code as usize] != pressed {
            self.keystate[code as usize] = pressed;
            output.send_key(code, pressed);
        }
    }

    fn clear_mod<O: Output>(&mut self, output: &mut O, code: u8) {
        let guard = (self.last_pressed_output_code == code) &&
                    (code == KEYD_LEFTMETA || code == KEYD_LEFTALT || code == KEYD_RIGHTALT) &&
                    self.inhibit_modifier_guard == 0 &&
                    self.config.disable_modifier_guard == 0;

        if guard && self.keystate[KEYD_LEFTCTRL as usize] == 0 {
            self.send_key(output, KEYD_LEFTCTRL, 1);
            self.send_key(output, code, 0);
            self.send_key(output, KEYD_LEFTCTRL, 0);
        } else {
            self.send_key(output, code, 0);
        }
    }

    fn set_mods<O: Output>(&mut self, output: &mut O, mods: u8) {
        for m in &MODIFIERS {
            if (m.mask & mods) != 0 {
                if self.keystate[m.key as usize] == 0 {
                    self.send_key(output, m.key, 1);
                }
            } else if self.keystate[m.key as usize] != 0 {
                self.clear_mod(output, m.key);
            }
        }
    }

    fn update_mods<O: Output>(&mut self, output: &mut O, excluded_layer_idx: i32, mods: u8) -> u8 {
        let mut final_mods = mods;
        for i in 0..self.config.layers.len() {
            if self.layer_state[i].active != 0 && (excluded_layer_idx == -1 || i != excluded_layer_idx as usize) {
                final_mods |= self.config.layers[i].mods;
            }
        }
        self.set_mods(output, final_mods);
        final_mods
    }

    fn handle_pending_timeout<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) {
        // Snapshot fields we need; return early if nothing pending or if this is the
        // same-tick release (Op::Timeout release handler will set spontaneous=1 instead).
        let (pt_code, pt_dl, pt_spontaneous, pt_expiration, pt_action1, pt_action2) =
            match self.pending_timeout.as_ref() {
                None => return,
                Some(pt) => {
                    if pressed == 0 && pt.code == code && time == pt.activation_time {
                        return;
                    }
                    (pt.code, pt.dl, pt.spontaneous, pt.expiration, pt.action1, pt.action2)
                }
            };

        // Determine if and how to resolve.
        let resolve: Option<(Descriptor, bool)> = if pt_spontaneous != 0 {
            // Key was released in the same tick as pressed.
            // Resolve once we see any subsequent event OR the deadline passes.
            if time >= pt_expiration || code != 0 {
                let action = if time >= pt_expiration { pt_action2 } else { pt_action1 };
                Some((action, true)) // true → fire both press AND release (key already up)
            } else {
                None
            }
        } else if time >= pt_expiration
            || (code != 0 && (pressed != 0 || code == pt_code))
        {
            // Normal resolution: timeout expired, or another key arrived while held.
            let action = if time >= pt_expiration { pt_action2 } else { pt_action1 };
            Some((action, false)) // false → fire press only; release comes on key-up
        } else {
            None
        };

        if let Some((action, both)) = resolve {
            let dl = pt_dl as i32;
            self.pending_timeout = None;

            if both {
                // Spontaneous tap: fire press + release immediately.
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
                self.execute_descriptor(output, action, pt_code, dl, 0, time);
            } else {
                // Hold: write cache so the eventual key-up releases correctly.
                self.cache_set(pt_code, Some(CacheEntry { code: pt_code, d: action, dl, layer: 0 }));
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
            }
        }
    }

    // ── Macro execution ───────────────────────────────────────────────────────

    fn play_macro_init_async<O: Output>(&mut self, output: &mut O, layer: i32, macro_idx: usize, time: i64) {
        let mac = self.config.macros[macro_idx];
        if mac.sz == 1 && mac.entries[0].entry_type == crate::macro_types::MacroEntryType::KeySequence {
            let entry = mac.entries[0];
            let code = (entry.data & 0xFF) as u8;
            let mods = (entry.data >> 8) as u8;
            self.update_mods(output, layer, mods);
            self.send_key(output, code, 1);
            self.send_key(output, code, 0);
            self.update_mods(output, -1, 0);
        } else {
            self.update_mods(output, -1, 0);
            self.macro_play.active_idx = Some(macro_idx);
            self.macro_play.layer = layer;
            self.macro_play.entry_idx = 0;
            self.macro_play.hold_start_idx = None;
            self.macro_play.is_repeating = false;
            self.macro_play.repeat_interval = 0;
            self.macro_play.timeout = time;
            self.play_macro_step(output, time);
        }
    }

    fn play_macro_step<O: Output>(&mut self, output: &mut O, time: i64) {
        use crate::macro_types::MacroEntryType;

        let Some(macro_idx) = self.macro_play.active_idx else { return };

        let mac = self.config.macros[macro_idx];
        let seq_timeout_ms = self.config.macro_sequence_timeout;

        while self.macro_play.entry_idx < mac.sz as usize {
            let ent = mac.entries[self.macro_play.entry_idx];
            self.macro_play.entry_idx += 1;

            match ent.entry_type {
                MacroEntryType::Hold => {
                    if self.macro_play.hold_start_idx.is_none() {
                        self.macro_play.hold_start_idx = Some(self.macro_play.entry_idx - 1);
                    }
                    output.send_key(ent.data as u8, 1);
                }
                MacroEntryType::Release => {
                    if let Some(start) = self.macro_play.hold_start_idx.take() {
                        for j in start..self.macro_play.entry_idx - 1 {
                            output.send_key(mac.entries[j].data as u8, 0);
                        }
                    }
                }
                MacroEntryType::Unicode => {
                    let codes = crate::unicode::unicode_get_sequence(ent.data as usize);
                    for &c in &codes {
                        if c != 0 {
                            output.send_key(c, 1);
                            output.send_key(c, 0);
                        }
                    }
                }
                MacroEntryType::KeySequence => {
                    let code = (ent.data & 0xFF) as u8;
                    let mods = (ent.data >> 8) as u8;
                    for md in &MODIFIERS {
                        if mods & md.mask != 0 {
                            output.send_key(md.key, 1);
                        }
                    }
                    output.send_key(code, 1);
                    output.send_key(code, 0);
                    for md in &MODIFIERS {
                        if mods & md.mask != 0 {
                            output.send_key(md.key, 0);
                        }
                    }
                }
                MacroEntryType::Timeout => {
                    let ms = ent.data as i64;
                    if ms > 0 {
                        let deadline = time + ms;
                        self.macro_play.timeout = deadline;
                        self.schedule_timeout(deadline);
                        return;
                    }
                }
            }

            if seq_timeout_ms > 0 && self.macro_play.entry_idx < mac.sz as usize {
                let deadline = time + seq_timeout_ms;
                self.macro_play.timeout = deadline;
                self.schedule_timeout(deadline);
                return;
            }
        }

        // Macro finished one full run.
        if self.macro_play.repeat_interval > 0 {
            self.macro_play.is_repeating = true;
            self.macro_play.entry_idx = 0;
            let deadline = time + self.macro_play.repeat_interval;
            self.macro_play.timeout = deadline;
            self.schedule_timeout(deadline);
        } else {
            self.macro_play.active_idx = None;
            self.update_mods(output, -1, 0);
        }
    }

    /// Public entry point for IPC/macro command execution (blocks, but used outside daemon loop).
    pub fn macro_execute_blocking<O: Output>(output: &mut O, mac: &crate::macro_types::Macro, seq_timeout_us: u64) -> i64 {
        use crate::macro_types::MacroEntryType;
        let mut hold_start: Option<usize> = None;
        let mut elapsed_ms: i64 = 0;

        for i in 0..mac.sz as usize {
            let ent = mac.entries[i];
            match ent.entry_type {
                MacroEntryType::Hold => {
                    if hold_start.is_none() { hold_start = Some(i); }
                    output.send_key(ent.data as u8, 1);
                }
                MacroEntryType::Release => {
                    if let Some(start) = hold_start.take() {
                        for j in start..i {
                            output.send_key(mac.entries[j].data as u8, 0);
                        }
                    }
                }
                MacroEntryType::Unicode => {
                    let codes = crate::unicode::unicode_get_sequence(ent.data as usize);
                    for &c in &codes {
                        if c != 0 { output.send_key(c, 1); output.send_key(c, 0); }
                    }
                }
                MacroEntryType::KeySequence => {
                    let code = (ent.data & 0xFF) as u8;
                    let mods = (ent.data >> 8) as u8;
                    for md in &MODIFIERS { if mods & md.mask != 0 { output.send_key(md.key, 1); } }
                    if mods != 0 && seq_timeout_us > 0 {
                        std::thread::sleep(std::time::Duration::from_micros(seq_timeout_us));
                    }
                    output.send_key(code, 1);
                    output.send_key(code, 0);
                    for md in &MODIFIERS { if mods & md.mask != 0 { output.send_key(md.key, 0); } }
                }
                MacroEntryType::Timeout => {
                    let ms = ent.data as u64;
                    if ms > 0 { std::thread::sleep(std::time::Duration::from_millis(ms)); }
                    elapsed_ms += ms as i64;
                }
            }
            if seq_timeout_us > 0 {
                std::thread::sleep(std::time::Duration::from_micros(seq_timeout_us));
                elapsed_ms += (seq_timeout_us / 1000) as i64;
            }
        }

        elapsed_ms
    }

    // ── Chord helpers ─────────────────────────────────────────────────────────

    /// Returns 0 = no match, 1 = partial, 2 = exact.
    fn chord_event_match(chord: &Chord, events: &[KeyEvent]) -> i32 {
        if events.is_empty() { return 0; }
        let mut n = 0usize;
        let mut npressed = 0usize;
        for ev in events {
            if ev.pressed != 0 {
                npressed += 1;
                if !chord.keys[..chord.sz].contains(&ev.code) {
                    return 0;
                }
                n += 1;
            }
        }
        if npressed == 0 { 0 } else if n == chord.sz { 2 } else { 1 }
    }

    /// Returns (ret, chord_idx, match_layer).
    /// ret: 0=none, 1=partial, 2=unambiguous full, 3=ambiguous.
    fn check_chord_match(&self) -> (i32, Option<usize>, i32) {
        let queue = &self.chord.queue[..self.chord.queue_sz];
        let mut full = false;
        let mut partial = false;
        let mut best_ci: Option<usize> = None;
        let mut best_layer: i32 = -1;
        let mut max_ts: i64 = -1;

        for li in 0..self.config.layers.len() {
            if self.layer_state[li].active == 0 { continue; }
            let layer = &self.config.layers[li];
            for ci in 0..layer.nr_chords {
                let r = Self::chord_event_match(&layer.chords[ci], queue);
                if r == 2 && self.layer_state[li].activation_time >= max_ts {
                    best_ci = Some(ci);
                    best_layer = li as i32;
                    full = true;
                    max_ts = self.layer_state[li].activation_time;
                } else if r == 1 {
                    partial = true;
                }
            }
        }

        let ret = i32::from(full) * 2 + i32::from(partial);
        (ret, best_ci, best_layer)
    }

    fn enqueue_chord_event(&mut self, code: u8, pressed: u8, time: i64) {
        if code == 0 { return; }
        let sz = self.chord.queue_sz;
        if sz < self.chord.queue.len() {
            self.chord.queue[sz] = KeyEvent { code, pressed, timestamp: time as i32 };
            self.chord.queue_sz += 1;
        }
    }

    fn resolve_chord<O: Output>(&mut self, output: &mut O) -> bool {
        self.chord.state = ChordStatus::Resolving;

        let match_idx   = self.chord.match_idx;
        let match_layer = self.chord.match_layer as usize;
        let last_time   = self.chord.last_code_time;
        let queue_sz    = self.chord.queue_sz;

        let mut queue_offset = 0usize;

        if let Some(ci) = match_idx {
            let chord = self.config.layers[match_layer].chords[ci]; // Copy
            let mut chord_code = 0u8;
            for i in 0..self.active_chords.len() {
                if self.active_chords[i].active == 0 {
                    self.active_chords[i] = ActiveChord { active: 1, chord, layer: match_layer as i32 };
                    chord_code = KEYD_CHORD_1 + i as u8;
                    break;
                }
            }
            if chord_code != 0 {
                queue_offset = chord.sz;
                self.process_event(output, chord_code, 1, last_time);
            }
        }

        // Snapshot events to flush before mutating chord state.
        let flush_sz = queue_sz.saturating_sub(queue_offset);
        let flush: Vec<KeyEvent> = self.chord.queue[queue_offset..queue_offset + flush_sz].to_vec();

        self.chord.queue_sz = 0;
        self.chord.match_idx = None;

        // Flush with state still Resolving so nested handle_chord returns immediately.
        if flush_sz > 0 {
            self.kbd_process_events(output, &flush);
        }

        self.chord.state = ChordStatus::Inactive;
        true
    }

    fn abort_chord<O: Output>(&mut self, output: &mut O) -> bool {
        self.chord.match_idx = None;
        self.resolve_chord(output)
    }

    fn handle_chord<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) -> bool {
        let interkey_timeout = self.config.chord_interkey_timeout;
        let hold_timeout     = self.config.chord_hold_timeout;

        // Release of a key belonging to an already-resolved active chord.
        if code != 0 && pressed == 0 {
            for i in 0..self.active_chords.len() {
                if self.active_chords[i].active == 0 { continue; }
                let chord_code = KEYD_CHORD_1 + i as u8;
                let mut found = false;
                let mut nremaining = 0usize;
                for j in 0..self.active_chords[i].chord.sz {
                    if self.active_chords[i].chord.keys[j] == code {
                        self.active_chords[i].chord.keys[j] = 0;
                        found = true;
                    }
                    if self.active_chords[i].chord.keys[j] != 0 { nremaining += 1; }
                }
                if found {
                    if nremaining == 0 {
                        self.active_chords[i].active = 0;
                        self.process_event(output, chord_code, 0, time);
                    }
                    return true;
                }
            }
        }

        let state = self.chord.state;
        match state {
            ChordStatus::Resolving => false,

            ChordStatus::Inactive => {
                self.chord.queue_sz = 0;
                self.chord.match_idx = None;
                self.chord.start_code = code;
                if code == 0 { return false; }
                self.enqueue_chord_event(code, pressed, time);
                let (ret, mi, ml) = self.check_chord_match();
                match ret {
                    0 => false,
                    1 | 3 => {
                        self.chord.match_idx = mi;
                        self.chord.match_layer = ml;
                        self.chord.state = ChordStatus::PendingDisambiguation;
                        self.chord.last_code_time = time;
                        self.schedule_timeout(time + interkey_timeout);
                        true
                    }
                    _ => {
                        self.chord.match_idx = mi;
                        self.chord.match_layer = ml;
                        self.chord.last_code_time = time;
                        if hold_timeout > 0 {
                            self.chord.state = ChordStatus::PendingHoldTimeout;
                            self.schedule_timeout(time + hold_timeout);
                            true
                        } else {
                            self.resolve_chord(output)
                        }
                    }
                }
            }

            ChordStatus::PendingDisambiguation => {
                if code == 0 {
                    if (time - self.chord.last_code_time) >= interkey_timeout {
                        if self.chord.match_idx.is_some() {
                            let timeleft = hold_timeout - interkey_timeout;
                            if timeleft > 0 {
                                self.schedule_timeout(time + timeleft);
                                self.chord.state = ChordStatus::PendingHoldTimeout;
                            } else {
                                return self.resolve_chord(output);
                            }
                        } else {
                            return self.abort_chord(output);
                        }
                        return true;
                    }
                    return false;
                }
                self.enqueue_chord_event(code, pressed, time);
                if pressed == 0 { return self.abort_chord(output); }
                let (ret, mi, ml) = self.check_chord_match();
                match ret {
                    0 => self.abort_chord(output),
                    1 | 3 => {
                        self.chord.match_idx = mi;
                        self.chord.match_layer = ml;
                        self.chord.last_code_time = time;
                        self.chord.state = ChordStatus::PendingDisambiguation;
                        self.schedule_timeout(time + interkey_timeout);
                        true
                    }
                    _ => {
                        self.chord.match_idx = mi;
                        self.chord.match_layer = ml;
                        self.chord.last_code_time = time;
                        if hold_timeout > 0 {
                            self.chord.state = ChordStatus::PendingHoldTimeout;
                            self.schedule_timeout(time + hold_timeout);
                            true
                        } else {
                            self.resolve_chord(output)
                        }
                    }
                }
            }

            ChordStatus::PendingHoldTimeout => {
                if code == 0 {
                    if (time - self.chord.last_code_time) >= hold_timeout {
                        return self.resolve_chord(output);
                    }
                    return false;
                }
                self.enqueue_chord_event(code, pressed, time);
                if pressed == 0 {
                    let is_chord_key = if let Some(ci) = self.chord.match_idx {
                        let li = self.chord.match_layer as usize;
                        let chord = &self.config.layers[li].chords[ci];
                        chord.keys[..chord.sz].contains(&code)
                    } else { false };
                    if is_chord_key { return self.abort_chord(output); }
                }
                true
            }
        }
    }

    fn resolve_descriptor(&self, code: u8) -> (Descriptor, i32) {
        // Virtual chord-key codes: look up the active chord slot.
        if code >= KEYD_CHORD_1 {
            let slot = (code - KEYD_CHORD_1) as usize;
            if slot < self.active_chords.len() && self.active_chords[slot].active != 0 {
                return (self.active_chords[slot].chord.d, self.active_chords[slot].layer);
            }
        }

        // Walk active layers in activation order (most recently activated first)
        let mut active_layers: Vec<usize> = (0..self.config.layers.len())
            .filter(|&i| self.layer_state[i].active != 0)
            .collect();
        
        active_layers.sort_by_key(|&i| std::cmp::Reverse(self.layer_state[i].activation_time));

        for &i in &active_layers {
            let d = &self.config.layers[i].keymap[code as usize];
            if d.op != Op::KeySequence || if let DescriptorData::KeySequence(ref ks) = d.data { ks.code != 0 } else { false } {
                return (*d, i as i32);
            }
        }

        // Fallback to main layer
        let main_idx = 0; // Assuming main is always at index 0
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

    fn process_event<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) -> i64 {
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
                // Guard against successive key-down for the same code (e.g. two devices
                // mapped to the same config both sending the same key).
                if self.cache_get(code).is_some() {
                    return self.calculate_main_loop_timeout(time);
                }
                let (d, layer) = self.resolve_descriptor(code);
                // Store in cache before executing so re-entrant lookups work correctly.
                self.cache_set(code, Some(CacheEntry { code, d, dl: layer, layer }));
                self.execute_descriptor(output, d, code, layer, pressed, time);
            } else if let Some(entry) = self.cache_get(code) {
                let d = entry.d;
                let layer = entry.layer;
                // Clear cache before executing so the key is no longer seen as held.
                self.cache_set(code, None);
                self.execute_descriptor(output, d, code, layer, pressed, time);
                // Silently ignore releases with no matching press (already cleaned up).
            }
        }

        self.calculate_main_loop_timeout(time)
    }

    fn execute_descriptor<O: Output>(&mut self, output: &mut O, d: Descriptor, code: u8, layer: i32, pressed: u8, time: i64) {
        match d.op {
            Op::KeySequence => {
                if let DescriptorData::KeySequence(ks) = d.data {
                    let new_code = ks.code;
                    let mods = ks.mods;
                    if pressed != 0 {
                        if self.keystate[new_code as usize] != 0 {
                            self.send_key(output, new_code, 0);
                        }
                        let active_mods = self.update_mods(output, layer, mods);
                        let mut ra = d;
                        if let DescriptorData::KeySequence(ref mut ra_ks) = ra.data {
                            ra_ks.mods = active_mods;
                        }
                        self.last_repeatable_action = ra;
                        self.send_key(output, new_code, 1);
                        self.clear_oneshot(output);
                    } else {
                        self.send_key(output, new_code, 0);
                        self.update_mods(output, -1, 0);
                    }
                    if mods == 0 || mods == MOD_SHIFT {
                        self.last_simple_key_time = time;
                    }
                } else {
                    // Passthrough (DescriptorData::None)
                    if pressed != 0 {
                        self.update_mods(output, layer, 0);
                        self.last_repeatable_action = d;
                        self.send_key(output, code, 1);
                        self.clear_oneshot(output);
                        self.last_simple_key_time = time;
                    } else {
                        self.send_key(output, code, 0);
                        self.update_mods(output, -1, 0);
                    }
                }
            }

            Op::Layer | Op::LayerM => {
                let idx = match d.data {
                    DescriptorData::Layer(l) => l.idx as usize,
                    DescriptorData::LayerMacro(lm) => lm.idx as usize,
                    _ => return,
                };
                if pressed != 0 {
                    // LayerM: execute macro before activating the layer.
                    if d.op == Op::LayerM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                    self.activate_layer(output, code, idx, time);
                } else {
                    self.deactivate_layer(output, idx);
                }
                // If this is a solo tap of a modifier-like key, suppress the spurious
                // modifier tap the OS might register.
                if self.last_pressed_code == code {
                    self.inhibit_modifier_guard = 1;
                    self.update_mods(output, -1, 0);
                    self.inhibit_modifier_guard = 0;
                } else {
                    self.update_mods(output, -1, 0);
                }
            }

            Op::Layout => {
                if pressed != 0 && let DescriptorData::Layer(l) = d.data {
                    self.setlayout(output, l.idx as usize);
                }
            }

            Op::Toggle | Op::ToggleM => {
                let idx = match d.data {
                    DescriptorData::Layer(l) => l.idx as usize,
                    DescriptorData::LayerMacro(lm) => lm.idx as usize,
                    _ => return,
                };
                if pressed != 0 {
                    self.layer_state[idx].toggled ^= 1;
                    if self.layer_state[idx].toggled != 0 {
                        self.activate_layer(output, code, idx, time);
                    } else {
                        self.deactivate_layer(output, idx);
                    }
                    self.update_mods(output, -1, 0);
                    self.clear_oneshot(output);
                    if d.op == Op::ToggleM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                }
            }

            Op::Swap | Op::SwapM => {
                let idx = match d.data {
                    DescriptorData::Layer(l) => l.idx as usize,
                    DescriptorData::LayerMacro(lm) => lm.idx as usize,
                    _ => return,
                };
                let dl = layer as usize;
                if pressed != 0 {
                    if self.layer_state[dl].toggled != 0 {
                        // Source layer was toggled: swap the toggle to target layer.
                        self.deactivate_layer(output, dl);
                        self.layer_state[dl].toggled = 0;
                        self.activate_layer(output, 0, idx, time);
                        self.layer_state[idx].toggled = 1;
                        self.update_mods(output, -1, 0);
                    } else if self.layer_state[dl].oneshot_depth > 0 {
                        // Source layer was a oneshot: swap the oneshot to target layer.
                        self.deactivate_layer(output, dl);
                        self.layer_state[dl].oneshot_depth -= 1;
                        self.activate_layer(output, 0, idx, time);
                        self.layer_state[idx].oneshot_depth += 1;
                        self.update_mods(output, -1, 0);
                    } else {
                        // Normal case: find the cache entry that is holding dl active,
                        // patch its descriptor to layer(idx), then swap activation.
                        let mut ce_code: Option<u8> = None;
                        for i in 0..16 {
                            if let Some(ce) = self.cache[i].as_mut().filter(|ce| {
                                ce.layer == dl as i32
                                    && ce.layer != 0
                                    && self.config.layers[ce.layer as usize].layer_type == LayerType::Normal
                            }) {
                                ce.d = Descriptor {
                                    op: Op::Layer,
                                    data: DescriptorData::Layer(DescLayer { idx: idx as i16 }),
                                };
                                ce_code = Some(ce.code);
                                break;
                            }
                        }
                        if let Some(activating_code) = ce_code {
                            self.deactivate_layer(output, dl);
                            self.activate_layer(output, activating_code, idx, time);
                            self.update_mods(output, -1, 0);
                        }
                    }
                    if d.op == Op::SwapM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                } else if d.op == Op::SwapM {
                    // On release: if macro is a single keysequence, send the key-up.
                    if let DescriptorData::LayerMacro(lm) = d.data {
                        let mac = self.config.macros[lm.macro_idx as usize];
                        if mac.sz == 1 && mac.entries[0].entry_type == crate::macro_types::MacroEntryType::KeySequence {
                            let c = (mac.entries[0].data & 0xFF) as u8;
                            self.send_key(output, c, 0);
                            self.update_mods(output, -1, 0);
                        }
                    }
                }
            }

            Op::Clear => {
                if pressed != 0 {
                    self.clear(output);
                }
            }

            Op::ClearM => {
                if pressed != 0 {
                    self.clear(output);
                    if let DescriptorData::MacroOp(m) = d.data {
                        self.play_macro_init_async(output, layer, m.macro_idx as usize, time);
                    }
                }
            }

            Op::Repeat => {
                if pressed != 0 {
                    let ra = self.last_repeatable_action;
                    // Re-execute the last repeatable action as a press+release.
                    self.execute_descriptor(output, ra, code, layer, 1, time);
                    // Patch the cache entry so the release undoes the repeated action.
                    for i in 0..16 {
                        if let Some(ce) = self.cache[i].as_mut().filter(|ce| ce.code == code) {
                            ce.d = ra;
                            break;
                        }
                    }
                }
            }

            Op::Overload => {
                if let DescriptorData::Overload(ov) = d.data {
                    let layer_idx  = ov.layer_idx as usize;
                    let action_desc = self.config.descriptors[ov.action_idx as usize];
                    if pressed != 0 {
                        self.overload_start_time = time;
                        self.activate_layer(output, code, layer_idx, time);
                        self.update_mods(output, -1, 0);
                    } else {
                        self.deactivate_layer(output, layer_idx);
                        self.update_mods(output, -1, 0);
                        let tap_timeout = self.config.overload_tap_timeout;
                        let is_tap = self.last_pressed_code == code
                            && (tap_timeout == 0
                                || (time - self.overload_start_time) < tap_timeout);
                        if is_tap {
                            self.execute_descriptor(output, action_desc, code, layer, 1, time);
                            self.execute_descriptor(output, action_desc, code, layer, 0, time);
                        }
                    }
                }
            }

            Op::OverloadTimeout | Op::OverloadTimeoutTap => {
                if let DescriptorData::OverloadTo(ov) = d.data && pressed != 0 {
                    // action1 = the tap descriptor; action2 = layer activation.
                    let action1 = self.config.descriptors[ov.action_idx as usize];
                    let action2 = Descriptor {
                        op: Op::Layer,
                        data: DescriptorData::Layer(DescLayer { idx: ov.layer_idx }),
                    };
                    let expiration = time + ov.timeout as i64;
                    self.pending_overload = Some(OverloadState {
                        code,
                        dl: layer as u8,
                        expiration,
                        resolve_on_interrupt: i32::from(d.op == Op::OverloadTimeoutTap),
                        queue: [KeyEvent { code: 0, pressed: 0, timestamp: 0 }; 32],
                        queue_sz: 0,
                        action1,
                        action2,
                    });
                    self.schedule_timeout(expiration);
                }
            }

            Op::OverloadIdleTimeout => {
                if let DescriptorData::OverloadIdle(ov) = d.data && pressed != 0 {
                    let idle = time - self.last_simple_key_time;
                    let action = if idle >= ov.timeout as i64 {
                        self.config.descriptors[ov.action2_idx as usize]
                    } else {
                        self.config.descriptors[ov.action1_idx as usize]
                    };
                    self.execute_descriptor(output, action, code, layer, 1, time);
                    // Patch cache so release uses the resolved action.
                    for i in 0..16 {
                        if let Some(ce) = self.cache[i].as_mut().filter(|ce| ce.code == code) {
                            ce.d = action;
                            break;
                        }
                    }
                }
            }

            Op::Timeout => {
                if let DescriptorData::TimeoutOp(to) = d.data {
                    if pressed != 0 {
                        let action1     = self.config.descriptors[to.action1_idx as usize];
                        let action2     = self.config.descriptors[to.action2_idx as usize];
                        let expiration  = time + to.timeout as i64;
                        self.pending_timeout = Some(TimeoutState {
                            code,
                            dl: layer as u8,
                            spontaneous: 0,
                            expiration,
                            activation_time: time,
                            action1,
                            action2,
                        });
                        self.schedule_timeout(expiration);
                    } else if let Some(ref mut pt) = self.pending_timeout {
                        // Release at the same tick as press → defer resolution.
                        if pt.code == code && time == pt.activation_time {
                            pt.spontaneous = 1;
                        }
                    }
                }
            }

            Op::Oneshot | Op::OneshotM | Op::OneshotK => {
                // Extract the layer index and, for OneshotK, the nested key descriptor index.
                let (idx, nested_idx) = match d.data {
                    DescriptorData::Layer(l)       => (l.idx as usize, None),
                    DescriptorData::LayerMacro(lm) => (lm.idx as usize, None), // macro: Phase 9
                    DescriptorData::Overload(ov)   => (ov.layer_idx as usize, Some(ov.action_idx as usize)),
                    _ => return,
                };

                if pressed != 0 {
                    // OneshotM: execute macro before activating the layer.
                    if d.op == Op::OneshotM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                    // OneshotK: also fire the nested key descriptor on press.
                    if let Some(ai) = nested_idx {
                        let nested = self.config.descriptors[ai];
                        self.execute_descriptor(output, nested, code, layer, 1, time);
                    }
                    self.activate_layer(output, code, idx, time);
                    self.update_mods(output, layer, 0);
                    self.oneshot_latch = 1;
                } else {
                    // OneshotK: also fire the nested key descriptor on release.
                    if let Some(ai) = nested_idx {
                        let nested = self.config.descriptors[ai];
                        self.execute_descriptor(output, nested, code, layer, 0, time);
                    }

                    if self.oneshot_latch != 0 {
                        // Tapped (released before any other key pressed) → schedule oneshot.
                        self.layer_state[idx].oneshot_depth += 1;
                        let ot = self.config.oneshot_timeout;
                        if ot != 0 {
                            let deadline = time + ot;
                            self.oneshot_timeout = deadline;
                            self.schedule_timeout(deadline);
                        }
                    } else {
                        // Another key was pressed while held → deactivate immediately.
                        self.deactivate_layer(output, idx);
                        self.update_mods(output, -1, 0);
                    }
                }
            }

            Op::OneshotMulti => {
                if let DescriptorData::LayerMulti(lm) = d.data {
                    if pressed != 0 {
                        for &i in lm.idx.iter().filter(|&&i| i != -1) {
                            self.activate_layer(output, code, i as usize, time);
                        }
                        self.update_mods(output, layer, 0);
                        self.oneshot_latch = 1;
                    } else if self.oneshot_latch != 0 {
                        for &i in lm.idx.iter().filter(|&&i| i != -1) {
                            self.layer_state[i as usize].oneshot_depth += 1;
                        }
                        let ot = self.config.oneshot_timeout;
                        if ot != 0 {
                            let deadline = time + ot;
                            self.oneshot_timeout = deadline;
                            self.schedule_timeout(deadline);
                        }
                    } else {
                        for &i in lm.idx.iter().filter(|&&i| i != -1) {
                            self.deactivate_layer(output, i as usize);
                        }
                        self.update_mods(output, -1, 0);
                    }
                }
            }

            Op::Macro | Op::Macro2 => {
                if pressed != 0 {
                    let (macro_idx, repeat_interval) = match d.data {
                        DescriptorData::MacroOp(m) => (
                            m.macro_idx as usize,
                            self.config.macro_repeat_timeout,
                        ),
                        DescriptorData::Macro2(m) => (
                            m.macro_idx as usize,
                            m.interval as i64,
                        ),
                        _ => return,
                    };
                    self.clear_oneshot(output);
                    self.play_macro_init_async(output, layer, macro_idx, time);
                    // play_macro_init_async might have set repeat_interval to 0, so we update it if needed.
                    if self.macro_play.active_idx == Some(macro_idx) {
                        self.macro_play.repeat_interval = repeat_interval;
                    }
                    self.last_repeatable_action = d;
                }
            }

            Op::Command => {
                if pressed != 0 && let DescriptorData::Command(cmd_d) = d.data {
                    let cmd = self.config.commands[cmd_d.cmd_idx as usize].cmd.clone();
                    let _ = std::process::Command::new("/bin/sh")
                        .args(["-c", &cmd])
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    self.clear_oneshot(output);
                    self.update_mods(output, -1, 0);
                }
            }

            Op::Scroll => {
                if let DescriptorData::Scroll(s) = d.data {
                    self.scroll.sensitivity = s.sensitivity as i32;
                    self.scroll.active = i32::from(pressed != 0);
                }
            }
            Op::ScrollToggleOn => {
                if let DescriptorData::Scroll(s) = d.data {
                    self.scroll.sensitivity = s.sensitivity as i32;
                    self.scroll.active = 1;
                }
            }
            Op::ScrollToggleOff => {
                if pressed != 0 { self.scroll.active = 0; }
            }
            Op::ScrollToggle => {
                if let DescriptorData::Scroll(s) = d.data {
                    self.scroll.sensitivity = s.sensitivity as i32;
                    if pressed != 0 { self.scroll.active ^= 1; }
                }
            }

        }

        // Track the last physically pressed key (used by inhibit_modifier_guard, overload tap, etc.)
        if pressed != 0 {
            self.last_pressed_code = code;
        }
    }
}
