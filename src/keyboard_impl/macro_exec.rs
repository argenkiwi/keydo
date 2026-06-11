use crate::keyboard_types::*;
use crate::keys::*;

impl Keyboard {
    pub(super) fn play_macro_init_async<O: Output>(&mut self, output: &mut O, layer: i32, macro_idx: usize, time: i64) {
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

    pub(super) fn play_macro_step<O: Output>(&mut self, output: &mut O, time: i64) {
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
}
