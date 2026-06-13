use crate::config::*;
use crate::keyboard_types::*;
use crate::keys::*;

impl Keyboard {
    pub(super) fn execute_descriptor<O: Output>(&mut self, output: &mut O, d: Descriptor, code: u8, layer: i32, pressed: u8, time: i64) {
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
                    } else {
                        self.send_key(output, code, 0);
                        self.update_mods(output, -1, 0);
                    }
                    self.last_simple_key_time = time;
                }
            }

            Op::Layer | Op::LayerM => {
                let idx = match d.data {
                    DescriptorData::Layer(l) => l.idx as usize,
                    DescriptorData::LayerMacro(lm) => lm.idx as usize,
                    _ => return,
                };
                if pressed != 0 {
                    if d.op == Op::LayerM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                    self.activate_layer(output, code, idx, time);
                } else {
                    self.deactivate_layer(output, idx);
                }
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
                        self.deactivate_layer(output, dl);
                        self.layer_state[dl].toggled = 0;
                        self.activate_layer(output, 0, idx, time);
                        self.layer_state[idx].toggled = 1;
                        self.update_mods(output, -1, 0);
                    } else if self.layer_state[dl].oneshot_depth > 0 {
                        self.deactivate_layer(output, dl);
                        self.layer_state[dl].oneshot_depth -= 1;
                        self.activate_layer(output, 0, idx, time);
                        self.layer_state[idx].oneshot_depth += 1;
                        self.update_mods(output, -1, 0);
                    } else {
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
                    self.execute_descriptor(output, ra, code, layer, 1, time);
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
                        if pt.code == code && time == pt.activation_time {
                            pt.spontaneous = 1;
                        }
                    }
                }
            }

            Op::Oneshot | Op::OneshotM | Op::OneshotK => {
                let (idx, nested_idx) = match d.data {
                    DescriptorData::Layer(l)       => (l.idx as usize, None),
                    DescriptorData::LayerMacro(lm) => (lm.idx as usize, None),
                    DescriptorData::Overload(ov)   => (ov.layer_idx as usize, Some(ov.action_idx as usize)),
                    _ => return,
                };

                if pressed != 0 {
                    if d.op == Op::OneshotM && let DescriptorData::LayerMacro(lm) = d.data {
                        self.play_macro_init_async(output, layer, lm.macro_idx as usize, time);
                    }
                    if let Some(ai) = nested_idx {
                        let nested = self.config.descriptors[ai];
                        self.execute_descriptor(output, nested, code, layer, 1, time);
                    }
                    self.activate_layer(output, code, idx, time);
                    self.update_mods(output, layer, 0);
                    self.oneshot_latch = 1;
                } else {
                    if let Some(ai) = nested_idx {
                        let nested = self.config.descriptors[ai];
                        self.execute_descriptor(output, nested, code, layer, 0, time);
                    }

                    if self.oneshot_latch != 0 {
                        self.layer_state[idx].oneshot_depth += 1;
                        let ot = self.config.oneshot_timeout;
                        if ot != 0 {
                            let deadline = time + ot;
                            self.oneshot_timeout = deadline;
                            self.schedule_timeout(deadline);
                        }
                    } else {
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
