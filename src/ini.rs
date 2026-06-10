pub struct IniEntry {
    pub key: String,
    pub val: Option<String>,
    pub lnum: usize,
}

pub struct IniSection {
    pub name: String,
    pub entries: Vec<IniEntry>,
    pub lnum: usize,
}

pub struct Ini {
    pub sections: Vec<IniSection>,
}

pub fn parse_kvp(s: &str) -> (String, Option<String>) {
    let s = s.trim();
    if s.is_empty() {
        return (String::new(), None);
    }

    let mut key_end = s.len();
    let mut val_start = None;

    let bytes = s.as_bytes();
    let mut i = 0;
    
    // Special case: first character is '='
    if bytes[0] == b'=' {
        i = 1;
    }

    while i < s.len() {
        if bytes[i] == b'=' {
            // Found the separator
            // Work backwards from i to find the key end (stripping trailing whitespace from key)
            let mut k_end = i;
            while k_end > 0 && (bytes[k_end - 1] == b' ' || bytes[k_end - 1] == b'\t') {
                k_end -= 1;
            }
            key_end = k_end;
            
            // Work forwards from i to find the value start
            let mut v_start = i + 1;
            while v_start < s.len() && (bytes[v_start] == b' ' || bytes[v_start] == b'\t') {
                v_start += 1;
            }
            val_start = Some(v_start);
            break;
        }
        i += 1;
    }

    let key = s[..key_end].to_string();
    let val = val_start.map(|start| s[start..].to_string());

    (key, val)
}

pub fn ini_parse_string(s: &str, default_section_name: Option<&str>) -> Option<Ini> {
    let mut ini = Ini { sections: Vec::new() };
    let mut current_section: Option<usize> = None;

    for (i, raw_line) in s.lines().enumerate() {
        let lnum = i + 1;
        let line = raw_line.trim_start();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            let line = line.trim();
            if line.ends_with(']') {
                let name = &line[1..line.len() - 1];
                ini.sections.push(IniSection {
                    name: name.to_string(),
                    entries: Vec::new(),
                    lnum,
                });
                current_section = Some(ini.sections.len() - 1);
                continue;
            }
        }

        if current_section.is_none() {
            if let Some(default_name) = default_section_name {
                ini.sections.push(IniSection {
                    name: default_name.to_string(),
                    entries: Vec::new(),
                    lnum: 0,
                });
                current_section = Some(0);
            } else {
                return None;
            }
        }

        let Some(idx) = current_section else { return None };
        let section = &mut ini.sections[idx];
        let (key, val) = parse_kvp(line);
        section.entries.push(IniEntry {
            key,
            val,
            lnum,
        });
    }

    Some(ini)
}
