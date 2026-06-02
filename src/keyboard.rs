use crate::config::{Config, Action, LayerType, Layer};
use crate::keys::{KeyCode, get_key_name, modifiers};
use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub code: u16,
    pub action: Action,
    pub dl: i16, // Layer index where it was found
    pub layer: i16,
}

#[derive(Debug, Default)]
pub struct LayerState {
    pub activation_time: u128,
    pub active: u8,
    pub toggled: bool,
    pub oneshot_depth: u8,
}

pub struct PendingTimeout {
    pub code: u16,
    pub dl: usize,
    pub spontaneous: bool,
    pub expiration: u128,
    pub activation_time: u128,
    pub action1: Action,
    pub action2: Action,
}

pub struct PendingOverload {
    pub code: u16,
    pub dl: usize,
    pub expiration: u128,
    pub resolve_on_interrupt: bool,
    pub queue: Vec<KeyEvent>,
    pub action1: Action,
    pub action2: Action,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChordState {
    Inactive,
    PendingDisambiguation,
    PendingHoldTimeout,
    Resolving,
}

pub struct ActiveChord {
    pub active: bool,
    pub keys: Vec<u16>,
    pub action: Action,
    pub layer: usize,
}

pub struct Keyboard {
    pub config: Config,
    pub cache: HashMap<u16, CacheEntry>,
    pub layer_states: Vec<LayerState>,
    pub keystate: [bool; 256],
    pub last_pressed_output_code: u16,
    pub last_pressed_code: u16,
    pub last_simple_key_time: u128,
    pub last_repeatable_action: Option<Action>,
    pub current_time: u128,
    pub pending_timeout: Option<PendingTimeout>,
    pub pending_overload: Option<PendingOverload>,
    
    pub oneshot_latch: bool,
    pub oneshot_timeout: u128,

    pub chord_state: ChordState,
    pub chord_queue: Vec<KeyEvent>,
    pub chord_match: Option<(usize, usize)>, // layer_idx, chord_idx
    pub chord_last_code_time: u128,
    pub active_chords: Vec<ActiveChord>,
}

impl Keyboard {
    pub fn new(config: Config) -> Self {
        let nr_layers = config.layers.len();
        let mut layer_states = Vec::with_capacity(nr_layers);
        for _ in 0..nr_layers {
            layer_states.push(LayerState::default());
        }
        // Main layer is always active
        layer_states[0].active = 1;

        Keyboard {
            config,
            cache: HashMap::new(),
            layer_states,
            keystate: [false; 256],
            last_pressed_output_code: 0,
            last_pressed_code: 0,
            last_simple_key_time: 0,
            last_repeatable_action: None,
            current_time: 0,
            pending_timeout: None,
            pending_overload: None,
            oneshot_latch: false,
            oneshot_timeout: 0,
            chord_state: ChordState::Inactive,
            chord_queue: Vec::new(),
            chord_match: None,
            chord_last_code_time: 0,
            active_chords: Vec::new(),
        }
    }

    pub fn set_time(&mut self, time: u128) {
        self.current_time = time;
    }

    fn get_time_ms(&self) -> u128 {
        self.current_time
    }

    pub fn process_event(&mut self, code: u16, pressed: bool) -> Vec<KeyEvent> {
        let mut events = Vec::new();
        let time = self.get_time_ms();

        if self.handle_chord(code, pressed, time, &mut events) {
            return events;
        }

        if self.handle_pending_timeout(code, pressed, time, &mut events) {
            return events;
        }

        if self.handle_pending_overload(code, pressed, time, &mut events) {
            return events;
        }

        if self.oneshot_timeout > 0 && time >= self.oneshot_timeout {
            self.clear_oneshot(&mut events);
        }

        if code != 0 {
            if pressed {
                self.oneshot_latch = false;
                if self.cache.contains_key(&code) {
                    return events;
                }

                let (action, dl) = self.lookup_descriptor(code);
                self.cache.insert(code, CacheEntry {
                    code,
                    action,
                    dl: dl as i16,
                    layer: 0,
                });

                self.execute_action(code, action, dl, true, time, &mut events);
            } else {
                if let Some(ce) = self.cache.remove(&code) {
                    self.execute_action(code, ce.action, ce.dl as usize, false, time, &mut events);
                }
            }
        }

        events
    }

    fn handle_chord(&mut self, code: u16, pressed: bool, time: u128, events: &mut Vec<KeyEvent>) -> bool {
        if code != 0 && !pressed {
            for ac in &mut self.active_chords {
                if ac.active {
                    let mut found = false;
                    for k in &mut ac.keys {
                        if *k == code {
                            *k = 0;
                            found = true;
                        }
                    }
                    if found {
                        if ac.keys.iter().all(|&k| k == 0) {
                            ac.active = false;
                            let action = ac.action;
                            let layer = ac.layer;
                            self.execute_action(0, action, layer, false, time, events);
                        }
                        return true;
                    }
                }
            }
        }

        match self.chord_state {
            ChordState::Resolving => false,
            ChordState::Inactive => {
                if code == 0 { return false; }
                self.chord_queue.clear();
                self.chord_match = None;
                self.chord_queue.push(KeyEvent { code, pressed });
                match self.check_chord_match() {
                    0 => false,
                    1 | 3 => {
                        self.chord_state = ChordState::PendingDisambiguation;
                        self.chord_last_code_time = time;
                        true
                    }
                    2 => {
                        self.chord_last_code_time = time;
                        if self.config.chord_hold_timeout > 0 {
                            self.chord_state = ChordState::PendingHoldTimeout;
                            true
                        } else {
                            self.resolve_chord(events)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ChordState::PendingDisambiguation => {
                if code == 0 {
                    if (time - self.chord_last_code_time) >= self.config.chord_interkey_timeout as u128 {
                        if self.chord_match.is_some() {
                             if self.config.chord_hold_timeout > self.config.chord_interkey_timeout {
                                 self.chord_state = ChordState::PendingHoldTimeout;
                                 return true;
                             } else {
                                 return self.resolve_chord(events);
                             }
                        } else {
                             return self.abort_chord(events);
                        }
                    }
                    return false;
                }
                self.chord_queue.push(KeyEvent { code, pressed });
                if !pressed { return self.abort_chord(events); }
                
                match self.check_chord_match() {
                    0 => self.abort_chord(events),
                    1 | 3 => {
                        self.chord_last_code_time = time;
                        true
                    }
                    2 => {
                        self.chord_last_code_time = time;
                        if self.config.chord_hold_timeout > 0 {
                            self.chord_state = ChordState::PendingHoldTimeout;
                            true
                        } else {
                            self.resolve_chord(events)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ChordState::PendingHoldTimeout => {
                if code == 0 {
                    if (time - self.chord_last_code_time) >= self.config.chord_hold_timeout as u128 {
                        return self.resolve_chord(events);
                    }
                    return false;
                }
                self.chord_queue.push(KeyEvent { code, pressed });
                if !pressed {
                    if let Some((l_idx, c_idx)) = self.chord_match {
                        let chord = &self.config.layers[l_idx].chords[c_idx];
                        if chord.keys.contains(&code) {
                            return self.abort_chord(events);
                        }
                    }
                }
                true
            }
        }
    }

    fn check_chord_match(&mut self) -> i32 {
        let mut full_match = false;
        let mut partial_match = false;
        let mut maxts = 0u128;

        for (idx, layer) in self.config.layers.iter().enumerate() {
            if self.layer_states[idx].active > 0 {
                for (c_idx, chord) in layer.chords.iter().enumerate() {
                    let ret = self.chord_event_match(chord);
                    if ret == 2 {
                        if self.layer_states[idx].activation_time >= maxts {
                            self.chord_match = Some((idx, c_idx));
                            full_match = true;
                            maxts = self.layer_states[idx].activation_time;
                        }
                    } else if ret == 1 {
                        partial_match = true;
                    }
                }
            }
        }

        if full_match { if partial_match { 3 } else { 2 } }
        else if partial_match { 1 }
        else { 0 }
    }

    fn chord_event_match(&self, chord: &crate::config::Chord) -> i32 {
        let pressed_keys: Vec<u16> = self.chord_queue.iter().filter(|ev| ev.pressed).map(|ev| ev.code).collect();
        if pressed_keys.is_empty() { return 0; }
        
        let all_in_chord = pressed_keys.iter().all(|k| chord.keys.contains(k));
        if !all_in_chord { return 0; }
        
        if pressed_keys.len() == chord.keys.len() { 2 } else { 1 }
    }

    fn resolve_chord(&mut self, events: &mut Vec<KeyEvent>) -> bool {
        self.chord_state = ChordState::Resolving;
        let mut queue_offset = 0;
        if let Some((l_idx, c_idx)) = self.chord_match {
            let chord = &self.config.layers[l_idx].chords[c_idx];
            let action = chord.action;
            self.active_chords.push(ActiveChord {
                active: true,
                keys: chord.keys.clone(),
                action,
                layer: l_idx,
            });
            queue_offset = chord.keys.len();
            self.execute_action(0, action, l_idx, true, self.chord_last_code_time, events);
        }

        let queue: Vec<KeyEvent> = self.chord_queue.drain(queue_offset..).collect();
        self.chord_queue.clear();
        for ev in queue {
            let mut q_events = self.process_event(ev.code, ev.pressed);
            events.append(&mut q_events);
        }
        self.chord_state = ChordState::Inactive;
        true
    }

    fn abort_chord(&mut self, events: &mut Vec<KeyEvent>) -> bool {
        self.chord_match = None;
        self.resolve_chord(events)
    }

    fn handle_pending_timeout(&mut self, code: u16, pressed: bool, time: u128, events: &mut Vec<KeyEvent>) -> bool {
        if let Some(pt) = self.pending_timeout.take() {
            if !pressed && code == pt.code && time == pt.activation_time {
                self.pending_timeout = Some(pt);
                return false;
            }

            if pt.spontaneous {
                if time >= pt.expiration || code != 0 {
                    let action = if time >= pt.expiration { pt.action2 } else { pt.action1 };
                    self.execute_action(pt.code, action, pt.dl, true, time, events);
                    self.execute_action(pt.code, action, pt.dl, false, time, events);
                    return code == 0;
                }
            } else if time >= pt.expiration || (code != 0 && (pressed || code == pt.code)) {
                let action = if time >= pt.expiration { pt.action2 } else { pt.action1 };
                self.cache.insert(pt.code, CacheEntry {
                    code: pt.code,
                    dl: pt.dl as i16,
                    action,
                    layer: 0,
                });
                self.execute_action(pt.code, action, pt.dl, true, time, events);
                return code == 0;
            }
            self.pending_timeout = Some(pt);
        }
        false
    }

    fn handle_pending_overload(&mut self, code: u16, pressed: bool, time: u128, events: &mut Vec<KeyEvent>) -> bool {
        if let Some(mut po) = self.pending_overload.take() {
            if code != 0 {
                if !pressed {
                     if !po.queue.iter().any(|ev| ev.code == code) && code != po.code {
                         self.pending_overload = Some(po);
                         return false;
                     }
                }
                po.queue.push(KeyEvent { code, pressed });
            }

            let action = if time >= po.expiration {
                Some(po.action2)
            } else if code == po.code && !pressed {
                Some(po.action1)
            } else if po.resolve_on_interrupt && !pressed && code != 0 {
                Some(po.action2)
            } else {
                None
            };

            if let Some(act) = action {
                let po_code = po.code;
                let dl = po.dl;
                let queue = po.queue;
                
                self.cache.insert(po_code, CacheEntry {
                    code: po_code,
                    dl: dl as i16,
                    action: act,
                    layer: 0,
                });
                self.execute_action(po_code, act, dl, true, time, events);
                
                // Flush queue
                for ev in queue {
                    let mut q_events = self.process_event(ev.code, ev.pressed);
                    events.append(&mut q_events);
                }
                return true;
            }

            self.pending_overload = Some(po);
        }
        false
    }

    fn lookup_descriptor(&self, code: u16) -> (Action, usize) {
        let mut maxts = 0u128;
        let mut best_action = None;
        let mut best_layer = 0;

        for (i, layer) in self.config.layers.iter().enumerate() {
            if self.layer_states[i].active > 0 {
                if let Some(&action) = layer.keymap.get(&code) {
                    if self.layer_states[i].activation_time >= maxts {
                        maxts = self.layer_states[i].activation_time;
                        best_action = Some(action);
                        best_layer = i;
                    }
                }
            }
        }

        // Composite layers take precedence
        let mut max_constituents = 0;
        for (i, layer) in self.config.layers.iter().enumerate() {
            if layer.layer_type == LayerType::Composite {
                let match_all = layer.constituents.iter().all(|&idx| self.layer_states[idx].active > 0);
                if match_all {
                    if let Some(&action) = layer.keymap.get(&code) {
                        if layer.constituents.len() > max_constituents {
                            max_constituents = layer.constituents.len();
                            best_action = Some(action);
                            best_layer = i;
                        }
                    }
                }
            }
        }

        match best_action {
            Some(action) => (action, best_layer),
            None => (Action::KeySequence(code, 0), 0),
        }
    }

    fn execute_action(&mut self, code: u16, action: Action, dl: usize, pressed: bool, time: u128, events: &mut Vec<KeyEvent>) {
        if pressed {
            self.last_pressed_code = code;
        }

        match action {
            Action::KeySequence(new_code, mods) => {
                if pressed {
                    let _active_mods = self.update_mods(dl, mods, events);
                    self.send_key(new_code, true, events);
                    self.clear_oneshot(events);
                } else {
                    self.send_key(new_code, false, events);
                    self.update_mods(usize::MAX, 0, events);
                }
                if mods == 0 || mods == crate::keys::MOD_SHIFT {
                    self.last_simple_key_time = time;
                }
            }
            Action::Layer(idx) => {
                if pressed {
                    self.activate_layer(idx as usize);
                } else {
                    self.deactivate_layer(idx as usize);
                }
                self.update_mods(usize::MAX, 0, events);
            }
            Action::Oneshot(l1, l2, l3) => {
                if pressed {
                    self.activate_layer(l1 as usize);
                    if l2 != -1 { self.activate_layer(l2 as usize); }
                    if l3 != -1 { self.activate_layer(l3 as usize); }
                    self.update_mods(dl, 0, events);
                    self.oneshot_latch = true;

                    if self.config.oneshot_timeout > 0 {
                        self.oneshot_timeout = time + self.config.oneshot_timeout as u128;
                    }
                } else {
                    if self.oneshot_latch {
                        self.layer_states[l1 as usize].oneshot_depth += 1;
                        if l2 != -1 { self.layer_states[l2 as usize].oneshot_depth += 1; }
                        if l3 != -1 { self.layer_states[l3 as usize].oneshot_depth += 1; }
                    } else {
                        self.deactivate_layer(l1 as usize);
                        if l2 != -1 { self.deactivate_layer(l2 as usize); }
                        if l3 != -1 { self.deactivate_layer(l3 as usize); }
                    }
                    self.update_mods(usize::MAX, 0, events);
                }
            }
            Action::Overload(layer_idx, action_idx) => {
                if pressed {
                    self.activate_layer(layer_idx as usize);
                    self.update_mods(usize::MAX, 0, events);
                } else {
                    self.deactivate_layer(layer_idx as usize);
                    self.update_mods(usize::MAX, 0, events);

                    if self.last_pressed_code == code {
                        let action = self.config.descriptors[action_idx as usize];
                        self.execute_action(code, action, dl, true, time, events);
                        self.execute_action(code, action, dl, false, time, events);
                    }
                }
            }
            Action::Macro(idx) => {
                if pressed {
                    self.clear_oneshot(events);
                    self.execute_macro_by_idx(idx as usize, events);
                }
            }
            Action::Layout(idx) => {
                if pressed {
                    self.set_layout(idx as usize, events);
                }
            }
            Action::Toggle(idx) => {
                if pressed {
                    self.layer_states[idx as usize].toggled = !self.layer_states[idx as usize].toggled;
                    if self.layer_states[idx as usize].toggled {
                        self.activate_layer(idx as usize);
                    } else {
                        self.deactivate_layer(idx as usize);
                    }
                    self.update_mods(usize::MAX, 0, events);
                }
            }
            Action::Swap(idx) => {
                if pressed {
                    if self.layer_states[dl].toggled {
                        self.deactivate_layer(dl);
                        self.layer_states[dl].toggled = false;
                        self.activate_layer(idx as usize);
                        self.layer_states[idx as usize].toggled = true;
                    } else if self.layer_states[dl].oneshot_depth > 0 {
                        self.deactivate_layer(dl);
                        self.layer_states[dl].oneshot_depth -= 1;
                        self.activate_layer(idx as usize);
                        self.layer_states[idx as usize].oneshot_depth += 1;
                    } else {
                        self.deactivate_layer(dl);
                        self.activate_layer(idx as usize);
                    }
                    self.update_mods(usize::MAX, 0, events);
                }
            }
            Action::Clear => {
                if pressed {
                    self.clear_all(events);
                }
            }
            Action::OverloadTimeout(layer_idx, action_idx, timeout) | 
            Action::OverloadTimeoutTap(layer_idx, action_idx, timeout) => {
                if pressed {
                    self.pending_overload = Some(PendingOverload {
                        code,
                        dl,
                        expiration: time + timeout as u128,
                        resolve_on_interrupt: match action { Action::OverloadTimeoutTap(_, _, _) => true, _ => false },
                        queue: Vec::new(),
                        action1: self.config.descriptors[action_idx as usize],
                        action2: Action::Layer(layer_idx),
                    });
                }
            }
            Action::Timeout(act1_idx, timeout, act2_idx) => {
                if pressed {
                    self.pending_timeout = Some(PendingTimeout {
                        code,
                        dl,
                        spontaneous: false,
                        expiration: time + timeout as u128,
                        activation_time: time,
                        action1: self.config.descriptors[act1_idx as usize],
                        action2: self.config.descriptors[act2_idx as usize],
                    });
                }
            }
            Action::OverloadIdleTimeout(act1_idx, act2_idx, timeout) => {
                if pressed {
                    let action = if (time - self.last_simple_key_time) >= timeout as u128 {
                        self.config.descriptors[act2_idx as usize]
                    } else {
                        self.config.descriptors[act1_idx as usize]
                    };
                    self.execute_action(code, action, dl, true, time, events);
                }
            }
            Action::OneshotKey(l_idx, d_idx) => {
                if pressed {
                    self.activate_layer(l_idx as usize);
                    self.execute_action(code, self.config.descriptors[d_idx as usize], dl, true, time, events);
                } else {
                    self.layer_states[l_idx as usize].oneshot_depth += 1;
                    self.execute_action(code, self.config.descriptors[d_idx as usize], dl, false, time, events);
                }
            }
            Action::OneshotMacro(l_idx, m_idx) => {
                if pressed {
                    let m = self.config.macros[m_idx as usize].clone();
                    self.execute_macro(&m, events);
                    self.activate_layer(l_idx as usize);
                } else {
                    self.layer_states[l_idx as usize].oneshot_depth += 1;
                }
            }
            Action::LayerMacro(l_idx, m_idx) => {
                if pressed {
                    let m = self.config.macros[m_idx as usize].clone();
                    self.execute_macro(&m, events);
                    self.activate_layer(l_idx as usize);
                } else {
                    self.deactivate_layer(l_idx as usize);
                }
            }
            _ => {}
        }
    }

    fn set_layout(&mut self, idx: usize, events: &mut Vec<KeyEvent>) {
        self.clear_all(events);
        for i in 1..self.layer_states.len() {
            if self.config.layers[i].layer_type == LayerType::Layout {
                self.layer_states[i].active = 0;
            }
        }
        if idx != 0 {
            self.layer_states[idx].activation_time = 1;
            self.layer_states[idx].active = 1;
        }
        self.update_mods(usize::MAX, 0, events);
    }

    fn clear_all(&mut self, events: &mut Vec<KeyEvent>) {
        self.clear_oneshot(events);
        for i in 1..self.layer_states.len() {
            if self.config.layers[i].layer_type != LayerType::Layout {
                if self.layer_states[i].toggled {
                    self.layer_states[i].toggled = false;
                    self.deactivate_layer(i);
                }
            }
        }
        self.reset_keystate(events);
    }

    fn reset_keystate(&mut self, events: &mut Vec<KeyEvent>) {
        for i in 0..256 {
            if self.keystate[i] {
                self.send_key(i as u16, false, events);
            }
        }
    }

    fn execute_macro(&mut self, m: &crate::config::Macro, events: &mut Vec<KeyEvent>) {
        for entry in &m.entries {
            match entry.entry_type {
                crate::config::MacroEntryType::KeySequence => {
                    let code = entry.data & 0xFF;
                    let mods = (entry.data >> 8) as u8;
                    self.update_mods(usize::MAX, mods, events);
                    self.send_key(code, true, events);
                    self.send_key(code, false, events);
                    self.update_mods(usize::MAX, 0, events);
                }
                _ => {
                    // TODO: Handle hold/release/timeout/unicode in macros
                }
            }
        }
    }

    fn execute_macro_by_idx(&mut self, idx: usize, events: &mut Vec<KeyEvent>) {
        let m = self.config.macros[idx].clone();
        self.execute_macro(&m, events);
    }

    fn clear_oneshot(&mut self, events: &mut Vec<KeyEvent>) {
        for i in 0..self.layer_states.len() {
            while self.layer_states[i].oneshot_depth > 0 {
                self.deactivate_layer(i);
                self.layer_states[i].oneshot_depth -= 1;
            }
        }
        self.oneshot_latch = false;
        self.oneshot_timeout = 0;
        self.update_mods(usize::MAX, 0, events);
    }

    fn update_mods(&mut self, excluded_layer_idx: usize, mut mods: u8, events: &mut Vec<KeyEvent>) -> u8 {
        for i in 0..self.config.layers.len() {
            if self.layer_states[i].active > 0 {
                let is_excluded = if i == excluded_layer_idx {
                    true
                } else if excluded_layer_idx != usize::MAX && self.config.layers[excluded_layer_idx].layer_type == LayerType::Composite {
                    self.config.layers[excluded_layer_idx].constituents.contains(&i)
                } else {
                    false
                };

                if !is_excluded {
                    mods |= self.config.layers[i].mods;
                }
            }
        }

        self.set_mods(mods, events);
        mods
    }

    fn set_mods(&mut self, mods: u8, events: &mut Vec<KeyEvent>) {
        for m in modifiers() {
            if (m.mask & mods) != 0 {
                if !self.keystate[m.key as usize] {
                    self.send_key(m.key as u16, true, events);
                }
            } else {
                if self.keystate[m.key as usize] {
                    self.send_key(m.key as u16, false, events);
                }
            }
        }
    }

    fn send_key(&mut self, code: u16, pressed: bool, events: &mut Vec<KeyEvent>) {
        if self.keystate[code as usize] != pressed {
            self.keystate[code as usize] = pressed;
            if pressed {
                self.last_pressed_output_code = code;
            }
            events.push(KeyEvent { code, pressed });
        }
    }

    fn activate_layer(&mut self, idx: usize) {
        self.layer_states[idx].active += 1;
        self.layer_states[idx].activation_time = self.get_time_ms();
    }

    fn deactivate_layer(&mut self, idx: usize) {
        if self.layer_states[idx].active > 0 {
            self.layer_states[idx].active -= 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyEvent {
    pub code: u16,
    pub pressed: bool,
}
