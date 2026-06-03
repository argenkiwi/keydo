use crate::config::*;
use crate::ini::parse_ini_string;
use crate::keys::lookup_keycode;
use crate::macro_parser::parse_macro;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

pub struct ConfigParser {
    config: Config,
    layer_map: HashMap<String, usize>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            config: Config {
                path: String::new(),
                layers: Vec::new(),
                descriptors: Vec::new(),
                macros: Vec::new(),
                commands: Vec::new(),
                aliases: HashMap::new(),
                wildcard: false,
                ids: Vec::new(),
                macro_timeout: 600,
                macro_sequence_timeout: 0,
                macro_repeat_timeout: 50,
                oneshot_timeout: 0,
                overload_tap_timeout: 0,
                chord_interkey_timeout: 50,
                chord_hold_timeout: 0,
                layer_indicator: false,
                disable_modifier_guard: false,
                default_layout: String::new(),
            },
            layer_map: HashMap::new(),
        }
    }

    pub fn parse(&mut self, path: &Path) -> anyhow::Result<Config> {
        let content = fs::read_to_string(path)?;
        self.config.path = path.to_string_lossy().to_string();
        
        // Initial setup similar to config_init in config.c
        self.init_default_config();

        let ini = parse_ini_string(&content, None).ok_or_else(|| anyhow::anyhow!("Failed to parse INI"))?;

        // First pass: create layers
        for section in &ini.sections {
            match section.name.as_str() {
                "ids" | "aliases" | "global" => {}
                _ => {
                    self.add_layer(&section.name)?;
                }
            }
        }

        // Second pass: populate everything
        for section in &ini.sections {
            match section.name.as_str() {
                "ids" => self.parse_id_section(section),
                "aliases" => self.parse_alias_section(section),
                "global" => self.parse_global_section(section),
                _ => {
                    let layer_name = section.name.split(':').next().unwrap();
                    let layer_idx = *self.layer_map.get(layer_name).unwrap();
                    for entry in &section.entries {
                        let action = self.parse_action(&entry.val.as_deref().unwrap_or(""))?;
                        
                        if entry.key.contains('+') {
                            let mut keys = Vec::new();
                            for k in entry.key.split('+') {
                                keys.push(self.resolve_key(k.trim()).ok_or_else(|| anyhow::anyhow!("Invalid key in chord: '{}' in section [{}]", k, section.name))?);
                            }
                            self.config.layers[layer_idx].chords.push(Chord {
                                keys,
                                action,
                            });
                        } else {
                            let code = self.resolve_key(&entry.key).ok_or_else(|| anyhow::anyhow!("Invalid key: '{}' in section [{}]", entry.key, section.name))?;
                            self.config.layers[layer_idx].keymap.insert(
                                code,
                                action,
                            );
                        }
                    }
                }
            }
        }

        Ok(self.config.clone())
    }

    fn resolve_key(&self, s: &str) -> Option<u16> {
        if let Some(code) = lookup_keycode(s) {
            return Some(code);
        }
        // Check aliases
        for (&code, alias) in &self.config.aliases {
            if alias == s {
                return Some(code);
            }
        }
        None
    }

    fn init_default_config(&mut self) {
        // Port of default_config string in config.c
        let default_layers = [
            ("main", LayerType::Layout, 0),
            ("control", LayerType::Normal, crate::keys::MOD_CTRL),
            ("shift", LayerType::Normal, crate::keys::MOD_SHIFT),
            ("meta", LayerType::Normal, crate::keys::MOD_SUPER),
            ("alt", LayerType::Normal, crate::keys::MOD_ALT),
            ("altgr", LayerType::Normal, crate::keys::MOD_ALT_GR),
        ];

        for (name, ltype, mods) in default_layers {
            let idx = self.config.layers.len();
            self.config.layers.push(Layer {
                name: name.to_string(),
                layer_type: ltype,
                mods,
                keymap: HashMap::new(),
                chords: Vec::new(),
                constituents: Vec::new(),
            });
            self.layer_map.insert(name.to_string(), idx);
        }

        // Default modifier aliases
        let aliases = [
            ("leftshift", "shift"),
            ("rightshift", "shift"),
            ("leftalt", "alt"),
            ("rightalt", "altgr"),
            ("leftmeta", "meta"),
            ("rightmeta", "meta"),
            ("leftcontrol", "control"),
            ("rightcontrol", "control"),
        ];

        for (key, alias) in aliases {
            if let Some(code) = lookup_keycode(key) {
                self.config.aliases.insert(code, alias.to_string());
            }
        }
        
        // Default layer bindings (handled automatically if we map them to their layers)
    }

    fn add_layer(&mut self, s: &str) -> anyhow::Result<()> {
        let mut parts = s.splitn(2, ':');
        let name = parts.next().unwrap();
        let ltype_str = parts.next();

        if self.layer_map.contains_key(name) {
            return Ok(());
        }

        let (ltype, mods) = if let Some(t) = ltype_str {
            if t == "layout" {
                (LayerType::Layout, 0)
            } else {
                let mut mods = 0u8;
                // Parse modset like C-S
                for m in t.split('-') {
                    match m {
                        "C" => mods |= crate::keys::MOD_CTRL,
                        "M" => mods |= crate::keys::MOD_SUPER,
                        "A" => mods |= crate::keys::MOD_ALT,
                        "S" => mods |= crate::keys::MOD_SHIFT,
                        "G" => mods |= crate::keys::MOD_ALT_GR,
                        _ => {}
                    }
                }
                (LayerType::Normal, mods)
            }
        } else {
            (LayerType::Normal, 0)
        };

        let idx = self.config.layers.len();
        let mut constituents = Vec::new();
        let final_ltype = if name.contains('+') {
            for part in name.split('+') {
                if let Some(&c_idx) = self.layer_map.get(part) {
                    constituents.push(c_idx);
                } else {
                    anyhow::bail!("Layer {} not found for composite layer {}", part, name);
                }
            }
            LayerType::Composite
        } else {
            ltype
        };

        self.config.layers.push(Layer {
            name: name.to_string(),
            layer_type: final_ltype,
            mods,
            keymap: HashMap::new(),
            chords: Vec::new(),
            constituents,
        });
        self.layer_map.insert(name.to_string(), idx);

        Ok(())
    }

    fn parse_id_section(&mut self, section: &crate::ini::IniSection) {
        for entry in &section.entries {
            if entry.key == "*" {
                self.config.wildcard = true;
            } else if entry.key.starts_with('-') {
                self.config.ids.push(DeviceId {
                    id: entry.key[1..].to_string(),
                    flags: ID_EXCLUDED,
                });
            } else {
                 let (prefix, id) = if entry.key.starts_with("m:") {
                    (ID_MOUSE, &entry.key[2..])
                } else if entry.key.starts_with("k:") {
                    (ID_KEYBOARD | ID_KEY, &entry.key[2..])
                } else {
                    (ID_KEYBOARD | ID_KEY | ID_MOUSE, entry.key.as_str())
                };
                self.config.ids.push(DeviceId {
                    id: id.to_string(),
                    flags: prefix,
                });
            }
        }
    }

    fn parse_alias_section(&mut self, section: &crate::ini::IniSection) {
        for entry in &section.entries {
            if let Some(code) = lookup_keycode(&entry.key) {
                if let Some(val) = &entry.val {
                    self.config.aliases.insert(code, val.clone());
                }
            }
        }
    }

    fn parse_global_section(&mut self, section: &crate::ini::IniSection) {
        for entry in &section.entries {
            let val = entry.val.as_deref().unwrap_or("0");
            match entry.key.as_str() {
                "macro_timeout" => self.config.macro_timeout = val.parse().unwrap_or(600),
                "macro_sequence_timeout" => self.config.macro_sequence_timeout = val.parse().unwrap_or(0),
                "macro_repeat_timeout" => self.config.macro_repeat_timeout = val.parse().unwrap_or(50),
                "oneshot_timeout" => self.config.oneshot_timeout = val.parse().unwrap_or(0),
                "overload_tap_timeout" => self.config.overload_tap_timeout = val.parse().unwrap_or(0),
                "chord_timeout" => self.config.chord_interkey_timeout = val.parse().unwrap_or(50),
                "chord_hold_timeout" => self.config.chord_hold_timeout = val.parse().unwrap_or(0),
                "layer_indicator" => self.config.layer_indicator = val == "1",
                "disable_modifier_guard" => self.config.disable_modifier_guard = val == "1",
                "default_layout" => self.config.default_layout = val.to_string(),
                _ => {}
            }
        }
    }

    fn parse_action(&mut self, s: &str) -> anyhow::Result<Action> {
        // This is a port of parse_descriptor from config.c
        if s.is_empty() {
            return Ok(Action::Noop);
        }

        // Try to parse as key sequence first
        if let Some((code, mods)) = self.parse_key_sequence_helper(s) {
            return Ok(Action::KeySequence(code, mods));
        }

        // Handle functions: layer(nav), oneshot(control), etc.
        if let Some(idx) = s.find('(') {
            if s.ends_with(')') {
                let name = &s[..idx];
                let args_str = &s[idx + 1..s.len() - 1];
                let args = self.parse_args(args_str);

                match name {
                    "layer" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        return Ok(Action::Layer(l_idx as i16));
                    }
                    "oneshot" => {
                        let l1 = self.get_layer_idx(&args[0])?;
                        let l2 = if args.len() > 1 { self.get_layer_idx(&args[1])? } else { -1 };
                        let l3 = if args.len() > 2 { self.get_layer_idx(&args[2])? } else { -1 };
                        return Ok(Action::Oneshot(l1 as i16, l2 as i16, l3 as i16));
                    }
                    "overload" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let action_idx = self.parse_and_store_descriptor(&args[1])?;
                        return Ok(Action::Overload(l_idx as i16, action_idx as i16));
                    }
                    "overloadt" | "overload2" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let action_idx = self.parse_and_store_descriptor(&args[1])?;
                        let timeout = args[2].parse().unwrap_or(0);
                        return Ok(Action::OverloadTimeout(l_idx as i16, action_idx as i16, timeout));
                    }
                    "overloadt2" | "overload3" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let action_idx = self.parse_and_store_descriptor(&args[1])?;
                        let timeout = args[2].parse().unwrap_or(0);
                        return Ok(Action::OverloadTimeoutTap(l_idx as i16, action_idx as i16, timeout));
                    }
                    "timeout" => {
                         let a1 = self.parse_and_store_descriptor(&args[0])?;
                         let timeout = args[1].parse().unwrap_or(0);
                         let a2 = self.parse_and_store_descriptor(&args[2])?;
                         return Ok(Action::Timeout(a1 as i16, timeout, a2 as i16));
                    }
                    "overloadi" => {
                         let a1 = self.parse_and_store_descriptor(&args[0])?;
                         let a2 = self.parse_and_store_descriptor(&args[1])?;
                         let timeout = args[2].parse().unwrap_or(0);
                         return Ok(Action::OverloadIdleTimeout(a1 as i16, a2 as i16, timeout));
                    }
                    "lettermod" => {
                        let layer_idx = self.get_layer_idx(&args[0])?;
                        let _key_code = self.resolve_key(&args[1]).ok_or_else(|| anyhow::anyhow!("Invalid key: {}", args[1]))?;
                        let idle_timeout = args[2].parse().unwrap_or(0);
                        let hold_timeout = args[3].parse().unwrap_or(0);
                        
                        let a1_idx = self.parse_and_store_descriptor(&args[1])?;
                        let a2_idx = {
                             let action = Action::OverloadTimeoutTap(layer_idx, a1_idx as i16, hold_timeout);
                             let idx = self.config.descriptors.len();
                             self.config.descriptors.push(action);
                             idx
                        };
                        return Ok(Action::OverloadIdleTimeout(a1_idx as i16, a2_idx as i16, idle_timeout));
                    }
                    "oneshotk" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let d_idx = self.parse_and_store_descriptor(&args[1])?;
                        return Ok(Action::OneshotKey(l_idx as i16, d_idx as i16));
                    }
                    "oneshotm" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let m_idx = self.parse_and_store_macro(&args[1])?;
                        return Ok(Action::OneshotMacro(l_idx as i16, m_idx as i16));
                    }
                    "layerm" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        let m_idx = self.parse_and_store_macro(&args[1])?;
                        return Ok(Action::LayerMacro(l_idx as i16, m_idx as i16));
                    }
                    "toggle" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        return Ok(Action::Toggle(l_idx as i16));
                    }
                    "swap" => {
                        let l_idx = self.get_layer_idx(&args[0])?;
                        return Ok(Action::Swap(l_idx as i16));
                    }
                    "macro" => {
                         let m_idx = self.parse_and_store_macro(args_str)?; // use the whole string inside macro()
                         return Ok(Action::Macro(m_idx as i16));
                    }
                    // TODO: Implement more actions
                    _ => {}
                }
            }
        }

        // Default to Noop or error
        Ok(Action::Noop)
    }

    fn parse_args(&self, s: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut escape = false;

        for c in s.chars() {
            if escape {
                current.push(c);
                escape = false;
                continue;
            }

            match c {
                '\\' => {
                    current.push(c);
                    escape = true;
                }
                '(' => {
                    depth += 1;
                    current.push(c);
                }
                ')' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    args.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        if !current.is_empty() {
            args.push(current.trim().to_string());
        }
        args
    }

    fn get_layer_idx(&self, name: &str) -> anyhow::Result<i16> {
        self.layer_map.get(name).map(|&i| i as i16).ok_or_else(|| anyhow::anyhow!("Layer {} not found", name))
    }

    fn parse_and_store_descriptor(&mut self, s: &str) -> anyhow::Result<usize> {
        let action = self.parse_action(s)?;
        let idx = self.config.descriptors.len();
        self.config.descriptors.push(action);
        Ok(idx)
    }

    fn parse_and_store_macro(&mut self, s: &str) -> anyhow::Result<usize> {
        let m = parse_macro(s).map_err(|e| anyhow::anyhow!(e))?;
        let idx = self.config.macros.len();
        self.config.macros.push(m);
        Ok(idx)
    }

    fn parse_key_sequence_helper(&self, s: &str) -> Option<(u16, u8)> {
        // Simplification for now, should use same logic as macro parser
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
                 return Some((code, mods));
            }
        } else {
            if let Some(code) = lookup_keycode(s) {
                return Some((code, mods));
            }
        }

        None
    }
}
