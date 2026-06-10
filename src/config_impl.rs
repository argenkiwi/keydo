//! Config loading and matching — parses `.conf` files into `Config` structs and checks device IDs.

use crate::config::*;
use crate::config_parse::*;
use crate::ini::*;
use crate::keys::*;

pub fn config_get_layer_index(config: &Config, name: &str) -> Option<usize> {
    config.layers.iter().position(|l| l.name == name)
}

fn create_layer(config: &mut Config, name: &str, layer_type: LayerType) -> usize {
    if let Some(idx) = config_get_layer_index(config, name) {
        return idx;
    }
    let idx = config.layers.len();
    config.layers.push(Layer::new(name.to_string()));
    config.layers[idx].layer_type = layer_type;
    idx
}

pub fn set_layer_entry(config: &mut Config, layer_idx: usize, key: &str, d: Descriptor) {
    if key.contains('+') {
        let mut keys = [0u8; 8];
        let mut sz = 0;
        for part in key.split('+') {
            if sz >= 8 { break; }
            if let Some((code, _)) = parse_key_sequence(part) {
                keys[sz] = code;
                sz += 1;
            } else {
                // Try alias
                let mut found_alias = false;
                for alias in &config.aliases {
                    if alias.0 == part && let Some((code, _)) = parse_key_sequence(&alias.1) {
                        keys[sz] = code;
                        sz += 1;
                        found_alias = true;
                        break;
                    }
                }
                if !found_alias {
                    // C-style alias lookup by name
                    for (i, ent) in KEYCODE_TABLE.iter().enumerate() {
                        if let Some(name) = ent.name && (name == part || ent.alt_name == Some(part)) {
                            keys[sz] = i as u8;
                            sz += 1;
                            break;
                        }
                    }
                }
            }
        }
        let nr_chords = config.layers[layer_idx].nr_chords;
        if nr_chords < 64 {
            config.layers[layer_idx].chords[nr_chords] = Chord {
                keys,
                sz,
                d,
            };
            config.layers[layer_idx].nr_chords += 1;
        }
    } else {
        let mut found = false;
        // Check exact aliases first (like C's strcpy(config->aliases[code], name))
        // Actually C checks all aliases: for (i = 0; i < 256; i++) if (!strcmp(config->aliases[i], key)) ...
        // Our aliases are (alias_name, target_name).
        let aliases_to_check: Vec<String> = config.aliases.iter()
            .filter(|(name, _target)| name == key)
            .map(|(_name, target)| target.clone())
            .collect();
        
        for target in aliases_to_check {
            if let Some((code, _)) = parse_key_sequence(&target) {
                config.layers[layer_idx].keymap[code as usize] = d;
                found = true;
            }
        }

        if !found {
            if let Some((code, _)) = parse_key_sequence(key) {
                config.layers[layer_idx].keymap[code as usize] = d;
            } else {
                // Try one more time with general alias lookup (target matches key)
                for alias in &config.aliases {
                    if alias.1 == key && let Some((code, _)) = parse_key_sequence(&alias.0) {
                        config.layers[layer_idx].keymap[code as usize] = d;
                    }
                }
            }
        }
    }
}

pub fn config_parse_string(config: &mut Config, content: &str) -> Result<usize, String> {
    let ini = ini_parse_string(content, None).ok_or("Failed to parse INI")?;
    let mut ctx = ParseCtx::new();

    // First pass: identify layers
    for section in &ini.sections {
        if section.name == "ids" || section.name == "global" || section.name == "aliases" {
            continue;
        }

        if section.name.contains('+') {
             create_layer(config, &section.name, LayerType::Composite);
        } else {
             create_layer(config, &section.name, LayerType::Normal);
        }
    }

    // Default layers — main is always a layout layer (mirrors C's `[main:layout]` default)
    let main_idx = create_layer(config, "main", LayerType::Normal);
    config.layers[main_idx].layer_type = LayerType::Layout;
    let control_idx = create_layer(config, "control", LayerType::Normal);
    config.layers[control_idx].mods = MOD_CTRL;
    let shift_idx = create_layer(config, "shift", LayerType::Normal);
    config.layers[shift_idx].mods = MOD_SHIFT;
    let alt_idx = create_layer(config, "alt", LayerType::Normal);
    config.layers[alt_idx].mods = MOD_ALT;
    let meta_idx = create_layer(config, "meta", LayerType::Normal);
    config.layers[meta_idx].mods = MOD_SUPER;
    let altgr_idx = create_layer(config, "altgr", LayerType::Normal);
    config.layers[altgr_idx].mods = MOD_ALT_GR;

    // Default aliases
    config.aliases.push(("leftshift".to_string(), "shift".to_string()));
    config.aliases.push(("rightshift".to_string(), "shift".to_string()));
    config.aliases.push(("leftalt".to_string(), "alt".to_string()));
    config.aliases.push(("rightalt".to_string(), "altgr".to_string()));
    config.aliases.push(("leftmeta".to_string(), "meta".to_string()));
    config.aliases.push(("rightmeta".to_string(), "meta".to_string()));
    config.aliases.push(("leftcontrol".to_string(), "control".to_string()));
    config.aliases.push(("rightcontrol".to_string(), "control".to_string()));

    // Default mappings
    let defaults = [
        ("shift", Op::Layer, shift_idx as i16),
        ("alt", Op::Layer, alt_idx as i16),
        ("altgr", Op::Layer, altgr_idx as i16),
        ("meta", Op::Layer, meta_idx as i16),
        ("control", Op::Layer, control_idx as i16),
    ];

    for (name, op, idx) in defaults {
        let desc = Descriptor {
            op,
            data: DescriptorData::Layer(DescLayer { idx }),
        };
        set_layer_entry(config, main_idx, name, desc);
    }

    // Second pass: parse content
    for section in &ini.sections {
        ctx.current_line = section.lnum - 1;

        if section.name == "ids" {
            for entry in &section.entries {
                let s = &entry.key;
                if s == "*" {
                    config.wildcard = 1;
                } else if let Some(id) = s.strip_prefix("m:") {
                    config.ids.push(ConfigId { id: id.to_string(), flags: ID_MOUSE });
                } else if let Some(id) = s.strip_prefix("k:") {
                    config.ids.push(ConfigId { id: id.to_string(), flags: ID_KEYBOARD | ID_KEY });
                } else if let Some(id) = s.strip_prefix('-') {
                    config.ids.push(ConfigId { id: id.to_string(), flags: ID_EXCLUDED });
                } else {
                    config.ids.push(ConfigId { id: s.clone(), flags: ID_KEYBOARD | ID_KEY | ID_MOUSE });
                }
            }
        } else if section.name == "global" {
            for entry in &section.entries {
                if let Some(ref val) = entry.val {
                    match entry.key.as_str() {
                        "macro_timeout" => config.macro_timeout = val.parse().unwrap_or(0),
                        "macro_sequence_timeout" => config.macro_sequence_timeout = val.parse().unwrap_or(0),
                        "macro_repeat_timeout" => config.macro_repeat_timeout = val.parse().unwrap_or(0),
                        "oneshot_timeout" => config.oneshot_timeout = val.parse().unwrap_or(0),
                        "overload_tap_timeout" => config.overload_tap_timeout = val.parse().unwrap_or(0),
                        "chord_interkey_timeout" | "chord_timeout" => config.chord_interkey_timeout = val.parse().unwrap_or(0),
                        "chord_hold_timeout" => config.chord_hold_timeout = val.parse().unwrap_or(0),
                        "layer_indicator" => config.layer_indicator = val.parse().unwrap_or(0),
                        "disable_modifier_guard" => config.disable_modifier_guard = val.parse().unwrap_or(0),
                        "default_layout" => config.default_layout.clone_from(val),
                        _ => config_warn(&mut ctx, &format!("Unknown global option: {}", entry.key)),
                    }
                }
            }
        } else if section.name == "aliases" {
            for entry in &section.entries {
                if let Some(ref val) = entry.val {
                    if let Some((code, _)) = parse_key_sequence(&entry.key) && let Some((alias_code, _)) = parse_key_sequence(val) {
                        config.layers[main_idx].keymap[code as usize] = Descriptor {
                            op: Op::KeySequence,
                            data: DescriptorData::KeySequence(DescKeySequence { code: alias_code, mods: 0 }),
                        };
                    }
                    config.aliases.push((entry.key.clone(), val.clone()));
                }
            }
        } else {
            let layer_idx = config_get_layer_index(config, &section.name)
                .ok_or_else(|| format!("internal error: layer '{}' missing after first pass", section.name))?;
            
            // Handle composite layer constituents
            if section.name.contains('+') {
                let parts: Vec<&str> = section.name.split('+').collect();
                let mut constituents = [0; 8];
                for (i, part) in parts.iter().enumerate() {
                    if i >= 8 { break; }
                    if let Some(idx) = config_get_layer_index(config, part) {
                        constituents[i] = idx as i32;
                    } else {
                        constituents[i] = create_layer(config, part, LayerType::Normal) as i32;
                    }
                }
                config.layers[layer_idx].nr_constituents = parts.len();
                config.layers[layer_idx].constituents = constituents;
            }

            for entry in &section.entries {
                ctx.current_line = entry.lnum - 1;
                
                if let Some(ref val) = entry.val {
                    let desc = config_parse_descriptor(val, config, &mut ctx)?;
                    set_layer_entry(config, layer_idx, &entry.key, desc);
                }
            }
        }
    }

    // Merge constituent keymaps into composite layers (explicit entries take priority).
    // Collect first to avoid borrow conflicts, then apply.
    let mut merges: Vec<(usize, usize, Descriptor)> = Vec::new();
    for i in 0..config.layers.len() {
        if config.layers[i].layer_type != LayerType::Composite {
            continue;
        }
        for ci in 0..config.layers[i].nr_constituents {
            let const_idx = config.layers[i].constituents[ci] as usize;
            for key in 0..256usize {
                let ce = &config.layers[i].keymap[key];
                let unset = ce.op == Op::KeySequence && matches!(ce.data, DescriptorData::None);
                if unset {
                    let fe = config.layers[const_idx].keymap[key];
                    let has_entry = fe.op != Op::KeySequence
                        || !matches!(fe.data, DescriptorData::None);
                    if has_entry {
                        merges.push((i, key, fe));
                    }
                }
            }
        }
    }
    for (ci, key, desc) in merges {
        config.layers[ci].keymap[key] = desc;
    }

    Ok(ctx.nr_warnings)
}

pub fn config_check_match(config: &Config, id: &str, flags: u8) -> i32 {
    for cfg_id in &config.ids {
        if id.starts_with(cfg_id.id.as_str()) {
            if cfg_id.flags & ID_EXCLUDED != 0 {
                return 0;
            } else if cfg_id.flags & flags != 0 {
                return 2;
            }
        }
    }
    i32::from(config.wildcard != 0)
}

pub fn config_parse(path: &str) -> Result<Config, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {path}: {e}"))?;

    let config_dir = std::path::Path::new(path)
        .parent()
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf);

    let preprocessed = preprocess_includes(&content, &config_dir)?;

    let mut config = Config::new();
    config_parse_string(&mut config, &preprocessed)?;
    config.path = path.to_string();
    Ok(config)
}

fn preprocess_includes(content: &str, config_dir: &std::path::Path) -> Result<String, String> {
    let mut result = String::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("include ") {
            let include_path_str = rest.trim();
            match resolve_include_path(config_dir, include_path_str) {
                Ok(resolved) => {
                    let included = std::fs::read_to_string(&resolved)
                        .map_err(|e| format!("Failed to open include {resolved}: {e}"))?;
                    let included_dir = std::path::Path::new(&resolved)
                        .parent()
                        .map_or_else(|| config_dir.to_path_buf(), std::path::Path::to_path_buf);
                    let nested = preprocess_includes(&included, &included_dir)?;
                    result.push_str(&nested);
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                }
                Err(e) => eprintln!("WARNING: {e}"),
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    Ok(result)
}

fn resolve_include_path(config_dir: &std::path::Path, include_path: &str) -> Result<String, String> {
    let candidate = config_dir.join(include_path);
    if candidate.exists() {
        return Ok(candidate.to_string_lossy().into_owned());
    }
    let data_dir = std::path::Path::new("/usr/share/keyd");
    let candidate = data_dir.join(include_path);
    if candidate.exists() {
        return Ok(candidate.to_string_lossy().into_owned());
    }
    Err(format!("Failed to resolve include path: {include_path}"))
}

pub fn config_add_entry(config: &mut Config, exp: &str) -> Result<(), String> {
    let parts: Vec<&str> = exp.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid entry expression".to_string());
    }
    let key_parts: Vec<&str> = parts[0].trim().split('.').collect();
    if key_parts.len() != 2 {
        return Err("Invalid key part".to_string());
    }
    let layer_name = key_parts[0];
    let key_name = key_parts[1];
    let val = parts[1].trim();

    let layer_idx = config_get_layer_index(config, layer_name).ok_or_else(|| format!("Layer {layer_name} not found"))?;
    let (code, _) = parse_key_sequence(key_name).ok_or_else(|| format!("Invalid key {key_name}"))?;
    
    let mut ctx = ParseCtx::new();
    let desc = config_parse_descriptor(val, config, &mut ctx)?;
    config.layers[layer_idx].keymap[code as usize] = desc;
    Ok(())
}
