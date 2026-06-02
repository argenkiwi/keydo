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
    if s.starts_with('=') {
        let (_, val) = s[1..].split_once('=').unwrap_or((&s[1..], ""));
        return ("=".to_string(), Some(val.trim().to_string()));
    }
    let mut parts = s.splitn(2, '=');
    let key = parts.next().unwrap_or("").trim().to_string();
    let val = parts.next().map(|v| v.trim().to_string());
    (key, val)
}

pub fn parse_ini_string(s: &str, default_section_name: Option<&str>) -> Option<Ini> {
    let mut ini = Ini { sections: Vec::new() };
    let mut current_section: Option<usize> = None;

    for (i, line) in s.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = &line[1..line.len() - 1];
            ini.sections.push(IniSection {
                name: section_name.to_string(),
                entries: Vec::new(),
                lnum: i + 1,
            });
            current_section = Some(ini.sections.len() - 1);
            continue;
        }

        let (key, val) = parse_kvp(line);
        if let Some(idx) = current_section {
            ini.sections[idx].entries.push(IniEntry {
                key,
                val,
                lnum: i + 1,
            });
        } else if let Some(name) = default_section_name {
            ini.sections.push(IniSection {
                name: name.to_string(),
                entries: Vec::new(),
                lnum: 0,
            });
            current_section = Some(ini.sections.len() - 1);
            ini.sections[current_section.unwrap()].entries.push(IniEntry {
                key,
                val,
                lnum: i + 1,
            });
        } else {
            return None;
        }
    }

    Some(ini)
}
