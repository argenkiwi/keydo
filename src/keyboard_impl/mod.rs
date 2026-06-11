//! Keyboard state machine — processes raw key events through layers, chords, macros, and overloads.

use crate::config::*;
use crate::keyboard_types::*;

mod actions;
mod cache;
mod chord;
mod dispatch;
mod layer;
mod macro_exec;
mod overload;
mod timeout;
mod timeout_handler;

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
}
