use crate::macro_types::Macro;

pub const MAX_LAYER_NAME_LEN: usize = 64;
pub const MAX_LAYERS: usize = 32;

pub const ID_EXCLUDED: u8 = 1;
pub const ID_MOUSE: u8 = 2;
pub const ID_KEYBOARD: u8 = 4;
pub const ID_TRACKPAD: u8 = 8;
pub const ID_KEY: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    KeySequence = 1,
    Oneshot,
    OneshotMulti,
    OneshotM,
    OneshotK,
    LayerM,
    Swap,
    SwapM,
    Layer,
    Layout,
    Clear,
    ClearM,
    Overload,
    OverloadTimeout,
    OverloadTimeoutTap,
    OverloadIdleTimeout,
    Toggle,
    ToggleM,
    Repeat,
    Macro,
    Macro2,
    Command,
    Timeout,
    ScrollToggleOn,
    ScrollToggleOff,
    ScrollToggle,
    Scroll,
}

#[derive(Debug, Clone, Copy)]
pub struct DescKeySequence { pub code: u8, pub mods: u8 }
#[derive(Debug, Clone, Copy)]
pub struct DescLayer { pub idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescLayerMulti { pub idx: [i16; 3] }
#[derive(Debug, Clone, Copy)]
pub struct DescMacro { pub macro_idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescCommand { pub cmd_idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescScroll { pub sensitivity: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescLayerMacro { pub idx: i16, pub macro_idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescOverload { pub layer_idx: i16, pub action_idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescOverloadTo { pub layer_idx: i16, pub action_idx: i16, pub timeout: u16 }
#[derive(Debug, Clone, Copy)]
pub struct DescOverloadIdle { pub action1_idx: i16, pub action2_idx: i16, pub timeout: u16 }
#[derive(Debug, Clone, Copy)]
pub struct DescTimeout { pub action1_idx: i16, pub timeout: u16, pub action2_idx: i16 }
#[derive(Debug, Clone, Copy)]
pub struct DescMacro2 { pub delay: u16, pub interval: u16, pub macro_idx: i16 }

#[derive(Debug, Clone, Copy)]
pub enum DescriptorData {
    KeySequence(DescKeySequence),
    Layer(DescLayer),
    LayerMulti(DescLayerMulti),
    MacroOp(DescMacro),
    Command(DescCommand),
    Scroll(DescScroll),
    LayerMacro(DescLayerMacro),
    Overload(DescOverload),
    OverloadTo(DescOverloadTo),
    OverloadIdle(DescOverloadIdle),
    TimeoutOp(DescTimeout),
    Macro2(DescMacro2),
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Descriptor {
    pub op: Op,
    pub data: DescriptorData,
}

#[derive(Debug, Clone, Copy)]
pub struct Chord {
    pub keys: [u8; 8],
    pub sz: usize,
    pub d: Descriptor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerType {
    Normal,
    Layout,
    Composite,
}

pub struct Layer {
    pub name: String,
    pub layer_type: LayerType,
    pub mods: u8,
    pub keymap: [Descriptor; 256],
    pub chords: [Chord; 64],
    pub nr_chords: usize,
    pub nr_constituents: usize,
    pub constituents: [i32; 8],
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            layer_type: LayerType::Normal,
            mods: 0,
            keymap: [Descriptor { op: Op::KeySequence, data: DescriptorData::None }; 256],
            chords: [Chord { keys: [0; 8], sz: 0, d: Descriptor { op: Op::KeySequence, data: DescriptorData::None } }; 64],
            nr_chords: 0,
            nr_constituents: 0,
            constituents: [0; 8],
        }
    }
}

pub struct Command {
    pub cmd: String,
}

pub struct ConfigId {
    pub id: String,
    pub flags: u8,
}

pub struct Config {
    pub path: String,
    pub layers: Vec<Layer>,
    pub descriptors: Vec<Descriptor>,
    pub macros: Vec<Macro>,
    pub commands: Vec<Command>,
    pub aliases: Vec<(String, String)>,
    pub wildcard: u8,
    pub ids: Vec<ConfigId>,
    pub macro_timeout: i64,
    pub macro_sequence_timeout: i64,
    pub macro_repeat_timeout: i64,
    pub oneshot_timeout: i64,
    pub overload_tap_timeout: i64,
    pub chord_interkey_timeout: i64,
    pub chord_hold_timeout: i64,
    pub layer_indicator: u8,
    pub disable_modifier_guard: u8,
    pub default_layout: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            path: String::new(),
            layers: Vec::new(),
            descriptors: Vec::new(),
            macros: Vec::new(),
            commands: Vec::new(),
            aliases: Vec::new(),
            wildcard: 0,
            ids: Vec::new(),
            macro_timeout: 600,
            macro_sequence_timeout: 0,
            macro_repeat_timeout: 50,
            oneshot_timeout: 0,
            overload_tap_timeout: 0,
            chord_interkey_timeout: 50,
            chord_hold_timeout: 0,
            layer_indicator: 0,
            disable_modifier_guard: 0,
            default_layout: String::new(),
        }
    }
}
