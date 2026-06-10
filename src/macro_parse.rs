use crate::keys::*;
use crate::macro_types::*;
use crate::unicode::unicode_lookup_index;

pub fn str_escape(s: &str) -> String {
    let mut res = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some(&'n') => { res.push('\n'); chars.next(); }
                Some(&'t') => { res.push('\t'); chars.next(); }
                Some(&'\\') => { res.push('\\'); chars.next(); }
                Some(&')') => { res.push(')'); chars.next(); }
                Some(&'(') => { res.push('('); chars.next(); }
                Some(_) | None => { res.push('\\'); }
            }
        } else {
            res.push(c);
        }
    }
    res
}

pub fn is_timeval(s: &str) -> bool {
    let s = s.trim_end_matches('\n');
    if s.len() < 3 || !s.ends_with("ms") {
        return false;
    }
    s[..s.len()-2].chars().all(|c| c.is_ascii_digit())
}

pub fn macro_parse(s: &str) -> Result<Macro, String> {
    let mut macro_obj = Macro::new();

    for tok in s.split_whitespace() {
        if let Some((code, mods)) = parse_key_sequence(tok) {
            add_entry(&mut macro_obj, MacroEntryType::KeySequence, ((mods as u16) << 8) | (code as u16))?;
        } else if tok.contains('+') {
            let parts = tok.split('+');
            for key in parts {
                if is_timeval(key) {
                    let timeout = key[..key.len()-2].parse::<u16>().map_err(|_| format!("Invalid timeout: {key}"))?;
                    add_entry(&mut macro_obj, MacroEntryType::Timeout, timeout)?;
                } else if let Some((code, _)) = parse_key_sequence(key) {
                    add_entry(&mut macro_obj, MacroEntryType::Hold, code as u16)?;
                } else {
                    return Err(format!("{key} is not a valid key"));
                }
            }
            add_entry(&mut macro_obj, MacroEntryType::Release, 0)?;
        } else if is_timeval(tok) {
            let timeout = tok[..tok.len()-2].parse::<u16>().map_err(|_| format!("Invalid timeout: {tok}"))?;
            add_entry(&mut macro_obj, MacroEntryType::Timeout, timeout)?;
        } else {
            for c in tok.chars() {
                let mut found = false;
                if c.is_ascii() && (c as u32) < 128 {
                    let c_str = c.to_string();
                    for (i, ent) in KEYCODE_TABLE.iter().enumerate() {
                        if ent.name.filter(|&n| n == c_str).is_some() {
                            add_entry(&mut macro_obj, MacroEntryType::KeySequence, i as u16)?;
                            found = true;
                            break;
                        }
                        if ent.shifted_name.filter(|&s| s == c_str).is_some() {
                            add_entry(&mut macro_obj, MacroEntryType::KeySequence, ((MOD_SHIFT as u16) << 8) | (i as u16))?;
                            found = true;
                            break;
                        }
                    }
                }
                if let Some(idx) = (!found).then(|| unicode_lookup_index(c as u32)).flatten() {
                    add_entry(&mut macro_obj, MacroEntryType::Unicode, idx as u16)?;
                    found = true;
                }
                if !found {
                    // C code just skips if not found or errors?
                    // "utf8_read_char" loop just continues.
                }
            }
        }
    }

    Ok(macro_obj)
}

fn add_entry(m: &mut Macro, t: MacroEntryType, d: u16) -> Result<(), String> {
    if m.sz as usize >= MAX_MACRO_ENTRIES {
        return Err(format!("maximum macro size ({MAX_MACRO_ENTRIES}) exceeded"));
    }
    m.entries[m.sz as usize] = MacroEntry { entry_type: t, data: d };
    m.sz += 1;
    Ok(())
}
