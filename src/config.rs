use crate::keys::{KeyCode, MOD_SHIFT};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    KeySequence(u16, u8),
    Oneshot(i16, i16, i16), // Layer indices
    OneshotMacro(i16, i16), // Layer index, macro index
    OneshotKey(i16, i16),   // Layer index, descriptor index
    LayerMacro(i16, i16),   // Layer index, macro index
    Swap(i16),
    SwapMacro(i16, i16),    // Layer index, macro index
    Layer(i16),
    Layout(i16),
    Clear,
    ClearMacro(i16),        // Macro index
    Overload(i16, i16),     // Layer index, descriptor index
    OverloadTimeout(i16, i16, u16), // Layer index, descriptor index, timeout
    OverloadTimeoutTap(i16, i16, u16), // Layer index, descriptor index, timeout
    OverloadIdleTimeout(i16, i16, u16), // Descriptor index 1, descriptor index 2, timeout
    Toggle(i16),
    ToggleMacro(i16, i16),  // Layer index, macro index
    Repeat,
    Macro(i16),             // Macro index
    Macro2(u16, u16, i16),  // Timeout, repeat timeout, macro index
    Command(i16),           // Command index
    Timeout(i16, u16, i16), // Descriptor index 1, timeout, descriptor index 2
    ScrollToggleOn(i16),
    ScrollToggleOff,
    ScrollToggle(i16),
    Scroll(i16),
    Noop,
}

#[derive(Debug, Clone)]
pub struct Chord {
    pub keys: Vec<u16>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerType {
    Normal,
    Layout,
    Composite,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub layer_type: LayerType,
    pub mods: u8,
    pub keymap: HashMap<u16, Action>,
    pub chords: Vec<Chord>,
    pub constituents: Vec<usize>, // For composite layers
}

#[derive(Debug, Clone)]
pub struct MacroEntry {
    pub entry_type: MacroEntryType,
    pub data: u16,
}

#[derive(Debug, Clone)]
pub enum MacroEntryType {
    KeySequence,
    Hold,
    Release,
    Unicode,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct Macro {
    pub entries: Vec<MacroEntry>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub cmd: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub layers: Vec<Layer>,
    pub descriptors: Vec<Action>,
    pub macros: Vec<Macro>,
    pub commands: Vec<Command>,
    pub aliases: HashMap<u16, String>,
    pub wildcard: bool,
    pub ids: Vec<DeviceId>,
    pub macro_timeout: u32,
    pub macro_sequence_timeout: u32,
    pub macro_repeat_timeout: u32,
    pub oneshot_timeout: u32,
    pub overload_tap_timeout: u32,
    pub chord_interkey_timeout: u32,
    pub chord_hold_timeout: u32,
    pub layer_indicator: bool,
    pub disable_modifier_guard: bool,
    pub default_layout: String,
}

#[derive(Debug, Clone)]
pub struct DeviceId {
    pub id: String,
    pub flags: u8,
}

pub const ID_EXCLUDED: u8 = 1;
pub const ID_MOUSE: u8 = 2;
pub const ID_KEYBOARD: u8 = 4;
pub const ID_TRACKPAD: u8 = 8;
pub const ID_KEY: u8 = 16;
