use crate::config::*;
use crate::keyboard_types::*;
use crate::keys::*;

impl Keyboard {
    pub(super) fn clear_oneshot<O: Output>(&mut self, output: &mut O) {
        for i in 0..self.config.layers.len() {
            while self.layer_state[i].oneshot_depth > 0 {
                self.deactivate_layer(output, i);
                self.layer_state[i].oneshot_depth -= 1;
            }
        }
        self.oneshot_latch = 0;
        self.oneshot_timeout = 0;
    }

    pub(super) fn reset_keystate<O: Output>(&mut self, output: &mut O) {
        for i in 0..256 {
            if self.keystate[i] != 0 {
                output.send_key(i as u8, 0);
                self.keystate[i] = 0;
            }
        }
    }

    pub(super) fn activate_layer<O: Output>(&mut self, output: &mut O, code: u8, idx: usize, time: i64) {
        self.layer_state[idx].activation_time = time;
        self.layer_state[idx].active += 1;
        for i in 0..16 {
            if let Some(ce) = self.cache[i].as_mut().filter(|ce| ce.code == code) {
                ce.layer = idx as i32;
                break;
            }
        }
        output.on_layer_change(self, idx, 1);
    }

    pub(super) fn deactivate_layer<O: Output>(&mut self, output: &mut O, idx: usize) {
        debug_assert!(self.layer_state[idx].active > 0, "deactivate_layer called on inactive layer {idx}");
        if self.layer_state[idx].active > 0 {
            self.layer_state[idx].active -= 1;
        }
        output.on_layer_change(self, idx, 0);
    }

    pub(super) fn clear<O: Output>(&mut self, output: &mut O) {
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

    pub(super) fn setlayout<O: Output>(&mut self, output: &mut O, idx: usize) {
        self.clear(output);
        for i in 1..self.config.layers.len() {
            if self.config.layers[i].layer_type == LayerType::Layout {
                self.layer_state[i].active = 0;
            }
        }
        if idx != 0 {
            self.layer_state[idx].activation_time = 1;
            self.layer_state[idx].active = 1;
        }
        output.on_layer_change(self, idx, 1);
    }

    pub(super) fn send_key<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8) {
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

    pub(super) fn clear_mod<O: Output>(&mut self, output: &mut O, code: u8) {
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

    pub(super) fn set_mods<O: Output>(&mut self, output: &mut O, mods: u8) {
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

    pub(super) fn update_mods<O: Output>(&mut self, output: &mut O, excluded_layer_idx: i32, mods: u8) -> u8 {
        let mut final_mods = mods;
        for i in 0..self.config.layers.len() {
            if self.layer_state[i].active != 0 && (excluded_layer_idx == -1 || i != excluded_layer_idx as usize) {
                final_mods |= self.config.layers[i].mods;
            }
        }
        self.set_mods(output, final_mods);
        final_mods
    }
}
