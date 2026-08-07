use crate::config::*;
use crate::keyboard_types::*;
use crate::keys::*;

impl Keyboard {
    /// Returns 0 = no match, 1 = partial, 2 = exact.
    pub(super) fn chord_event_match(chord: &Chord, events: &[KeyEvent]) -> i32 {
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
    pub(super) fn check_chord_match(&self) -> (i32, Option<usize>, i32) {
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

    pub(super) fn enqueue_chord_event(&mut self, code: u8, pressed: u8, time: i64) {
        if code == 0 { return; }
        let sz = self.chord.queue_sz;
        debug_assert!(
            sz < self.chord.queue.len(),
            "chord queue full ({CHORD_QUEUE_LEN} slots) enqueuing code {code}"
        );
        if sz < self.chord.queue.len() {
            self.chord.queue[sz] = KeyEvent { code, pressed, timestamp: time as i32 };
            self.chord.queue_sz += 1;
        } else {
            log::trace!("chord queue full ({CHORD_QUEUE_LEN} slots): dropping code {code}");
        }
    }

    pub(super) fn resolve_chord<O: Output>(&mut self, output: &mut O) -> bool {
        self.chord.state = ChordStatus::Resolving;

        let match_idx   = self.chord.match_idx;
        let match_layer = self.chord.match_layer as usize;
        let last_time   = self.chord.last_code_time;
        let queue_sz    = self.chord.queue_sz;

        let mut queue_offset = 0usize;

        if let Some(ci) = match_idx {
            let chord = self.config.layers[match_layer].chords[ci];
            let mut chord_code = 0u8;
            for i in 0..self.active_chords.len() {
                if self.active_chords[i].active == 0 {
                    self.active_chords[i] = ActiveChord { active: 1, chord, layer: match_layer as i32 };
                    chord_code = KEYD_CHORD_1 + i as u8;
                    break;
                }
            }
            if chord_code != 0 {
                log::trace!("chord resolve: matched layer {match_layer} chord {ci} -> synthetic code {chord_code}");
                queue_offset = chord.sz;
                self.process_event(output, chord_code, 1, last_time);
            } else {
                log::trace!(
                    "chord resolve: matched layer {match_layer} chord {ci} but no free active_chords \
                     slot ({MAX_ACTIVE_CHORDS} in use) — falling back to raw passthrough"
                );
            }
        } else {
            log::trace!("chord resolve: no match, flushing {queue_sz} queued event(s) as raw passthrough");
        }

        // Snapshot onto the stack: kbd_process_events needs &mut self, and the
        // queue is a fixed 32 slots, so there is nothing to allocate for.
        let flush_sz = queue_sz.saturating_sub(queue_offset);
        let mut flush = [KeyEvent { code: 0, pressed: 0, timestamp: 0 }; CHORD_QUEUE_LEN];
        flush[..flush_sz].copy_from_slice(&self.chord.queue[queue_offset..queue_offset + flush_sz]);

        self.chord.queue_sz = 0;
        self.chord.match_idx = None;

        if flush_sz > 0 {
            self.kbd_process_events(output, &flush[..flush_sz]);
        }

        self.chord.state = ChordStatus::Inactive;
        true
    }

    pub(super) fn abort_chord<O: Output>(&mut self, output: &mut O) -> bool {
        log::trace!("chord abort: discarding match, flushing queue as raw passthrough");
        self.chord.match_idx = None;
        self.resolve_chord(output)
    }

    pub(super) fn handle_chord<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) -> bool {
        // With no chords configured no slot can ever activate and the state
        // machine can never leave Inactive, so skip the per-event walk over
        // every active layer that check_chord_match would otherwise do.
        if !self.has_chords {
            debug_assert_eq!(self.chord.state, ChordStatus::Inactive);
            return false;
        }

        let interkey_timeout = self.config.chord_interkey_timeout;
        let hold_timeout     = self.config.chord_hold_timeout;

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

        // Events replayed from a pending-overload's deferred queue were
        // already set aside waiting on an unrelated key's tap-vs-hold
        // decision; letting them start or extend a fresh chord
        // disambiguation here would let a key that has nothing to do with
        // the chord get folded into `chord.queue` and reordered ahead of
        // whatever's genuinely mid-chord. Already-active chords (handled
        // above) are unaffected -- only new disambiguation is skipped.
        if self.chord_suppressed {
            return false;
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
}
