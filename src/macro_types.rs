pub const MAX_MACRO_ENTRIES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MacroEntryType {
    #[default]
    KeySequence,
    Hold,
    Release,
    Unicode,
    Timeout,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MacroEntry {
    pub entry_type: MacroEntryType,
    pub data: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Macro {
    pub entries: [MacroEntry; MAX_MACRO_ENTRIES],
    pub sz: u32,
}

impl Default for Macro {
    fn default() -> Self {
        Self {
            entries: [MacroEntry { entry_type: MacroEntryType::KeySequence, data: 0 }; MAX_MACRO_ENTRIES],
            sz: 0,
        }
    }
}

impl Macro {
    pub fn new() -> Self {
        Self::default()
    }
}
