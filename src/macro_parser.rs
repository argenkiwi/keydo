use crate::config::{Macro, MacroEntry, MacroEntryType};
use crate::keys::lookup_keycode;

pub fn parse_macro(s: &str) -> Result<Macro, String> {
    let mut entries = Vec::new();
    let tokens = s.split_whitespace();

    for tok in tokens {
        if let Some(code_mods) = parse_key_sequence_opt(tok) {
            let (code, mods) = code_mods;
            entries.push(MacroEntry {
                entry_type: MacroEntryType::KeySequence,
                data: ((mods as u16) << 8) | code,
            });
        } else if tok.contains('+') {
            let keys: Vec<&str> = tok.split('+').collect();
            let mut hold_keys = Vec::new();

            for key in keys {
                if let Some(ms) = parse_timeout(key) {
                    entries.push(MacroEntry {
                        entry_type: MacroEntryType::Timeout,
                        data: ms,
                    });
                } else if let Some((code, _mods)) = parse_key_sequence_opt(key) {
                    entries.push(MacroEntry {
                        entry_type: MacroEntryType::Hold,
                        data: code,
                    });
                    hold_keys.push(code);
                } else {
                    return Err(format!("{} is not a valid key", key));
                }
            }

            entries.push(MacroEntry {
                entry_type: MacroEntryType::Release,
                data: 0,
            });
        } else if let Some(ms) = parse_timeout(tok) {
            entries.push(MacroEntry {
                entry_type: MacroEntryType::Timeout,
                data: ms,
            });
        } else {
            // Unicode characters
            for c in tok.chars() {
                if c.is_ascii() {
                    // Try to find a keycode for this ASCII char
                    if let Some(code) = lookup_keycode(&c.to_string()) {
                         entries.push(MacroEntry {
                            entry_type: MacroEntryType::KeySequence,
                            data: code,
                        });
                    } else {
                        // Handle shifted keys if necessary, or just use unicode if not found
                        // For simplicity, if not in keycode table, we'd use unicode.
                        // But keyd lookup_keycode handles names like "1", "2", "a", "b".
                        // Symbols like "!" are in shifted_name.
                        // I should probably improve keys.rs to handle this better.
                         entries.push(MacroEntry {
                            entry_type: MacroEntryType::Unicode,
                            data: c as u16, // TODO: Proper unicode table handling
                        });
                    }
                } else {
                    entries.push(MacroEntry {
                        entry_type: MacroEntryType::Unicode,
                        data: c as u16, // TODO: Proper unicode table handling
                    });
                }
            }
        }
    }

    Ok(Macro { entries })
}

fn parse_key_sequence_opt(s: &str) -> Option<(u16, u8)> {
    // Port of parse_key_sequence from keys.c
    // It handles things like C-A-x
    let mut mods = 0u8;
    let mut parts: Vec<&str> = s.split('-').collect();
    
    if parts.len() > 1 {
        let key_part = parts.pop().unwrap();
        for m in parts {
            match m {
                "C" => mods |= crate::keys::MOD_CTRL,
                "M" => mods |= crate::keys::MOD_SUPER,
                "A" => mods |= crate::keys::MOD_ALT,
                "S" => mods |= crate::keys::MOD_SHIFT,
                "G" => mods |= crate::keys::MOD_ALT_GR,
                _ => return None,
            }
        }
        if let Some(code) = lookup_keycode(key_part) {
             // Check if the key itself is a shifted name
             // But parse_key_sequence in C also handles that.
             return Some((code, mods));
        }
    } else {
        if let Some(code) = lookup_keycode(s) {
            return Some((code, mods));
        }
    }

    None
}

fn parse_timeout(s: &str) -> Option<u16> {
    if s.ends_with("ms") {
        s[..s.len() - 2].parse::<u16>().ok()
    } else {
        None
    }
}
