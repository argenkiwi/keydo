//! Descriptor and macro expression parsing — converts config value strings into `Descriptor` structs.

use crate::config::*;
use crate::macro_types::*;
use crate::keys::*;
use crate::macro_parse::{macro_parse, str_escape};

#[derive(Default)]
pub struct ParseCtx {
    pub current_line: usize,
    pub current_file: Option<String>,
    pub nr_warnings: usize,
}

impl ParseCtx {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn config_warn(ctx: &mut ParseCtx, msg: &str) {
    if let Some(ref file) = ctx.current_file {
        eprintln!("\tw{{WARNING:}} b{{{file}}}:r{{{}}}: {msg}", ctx.current_line + 1);
    } else {
        eprintln!("\tw{{WARNING:}} {msg}");
    }
    ctx.nr_warnings += 1;
}

pub fn config_lookup_keycode(name: &str) -> u8 {
    for (i, ent) in KEYCODE_TABLE.iter().enumerate() {
        if let Some(n) = ent.name && (n == name || ent.alt_name == Some(name)) {
            return i as u8;
        }
    }
    0
}

pub fn parse_fn(s: &str) -> Option<(String, Vec<String>)> {
    let open_paren = s.find('(')?;
    let close_paren = s.rfind(')')?;
    if close_paren < open_paren {
        return None;
    }

    let name = s[..open_paren].trim().to_string();
    let args_str = &s[open_paren + 1..close_paren];

    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut paren_level = 0;
    let mut chars = args_str.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next_c) = chars.next() {
                    current_arg.push('\\');
                    current_arg.push(next_c);
                } else {
                    current_arg.push('\\');
                }
            }
            '(' => {
                paren_level += 1;
                current_arg.push(c);
            }
            ')' => {
                paren_level -= 1;
                current_arg.push(c);
            }
            ',' if paren_level == 0 => {
                args.push(current_arg.trim().to_string());
                current_arg = String::new();
            }
            _ => {
                current_arg.push(c);
            }
        }
    }
    if !current_arg.trim().is_empty() || args_str.trim().is_empty() && args.is_empty() {
         // Handle empty args case or last arg
         if !args_str.trim().is_empty() {
            args.push(current_arg.trim().to_string());
         }
    }

    Some((name, args))
}

pub fn config_parse_macro_expression(s: &str) -> Result<Macro, String> {
    if s.starts_with("macro(") && s.ends_with(')') {
        let inner = &s[6..s.len()-1];
        let escaped = str_escape(inner);
        return macro_parse(&escaped);
    } else if let Some((_code, _mods)) = parse_key_sequence(s) {
        return macro_parse(s);
    } else if s.chars().count() == 1 {
        return macro_parse(s);
    }
    Err("Invalid macro".to_string())
}

pub fn config_parse_command(s: &str) -> Result<Command, String> {
    if s.starts_with("command(") && s.ends_with(')') {
        let cmd = &s[8..s.len()-1];
        return Ok(Command { cmd: str_escape(cmd) });
    }
    Err("Not a command expression".to_string())
}

pub fn config_get_layer_index(config: &Config, name: &str) -> Option<usize> {
    config.layers.iter().position(|l| l.name == name)
}

pub fn config_parse_descriptor(
    s: &str,
    config: &mut Config,
    ctx: &mut ParseCtx,
) -> Result<Descriptor, String> {
    if s.is_empty() {
        return Ok(Descriptor { op: Op::KeySequence, data: DescriptorData::None });
    }

    if let Some((code, mods)) = parse_key_sequence(s) {
        let layer = match code {
            KEYD_LEFTSHIFT | KEYD_RIGHTSHIFT => Some("shift"),
            KEYD_LEFTCTRL | KEYD_RIGHTCTRL => Some("control"),
            KEYD_LEFTMETA | KEYD_RIGHTMETA => Some("meta"),
            KEYD_LEFTALT => Some("alt"),
            KEYD_RIGHTALT => Some("altgr"),
            _ => None,
        };

        if let Some(layer_name) = layer {
            let key_name = KEYCODE_TABLE[code as usize].name.unwrap_or("UNKNOWN");
            config_warn(ctx, &format!("You should use layer({layer_name}) instead of assigning to {key_name} directly."));
            if let Some(idx) = config_get_layer_index(config, layer_name) {
                return Ok(Descriptor {
                    op: Op::Layer,
                    data: DescriptorData::Layer(DescLayer { idx: idx as i16 }),
                });
            }
        }

        return Ok(Descriptor {
            op: Op::KeySequence,
            data: DescriptorData::KeySequence(DescKeySequence { code, mods }),
        });
    }

    if let Ok(cmd) = config_parse_command(s) {
        let idx = config.commands.len();
        config.commands.push(cmd);
        return Ok(Descriptor {
            op: Op::Command,
            data: DescriptorData::Command(DescCommand { cmd_idx: idx as i16 }),
        });
    }

    if let Ok(m) = config_parse_macro_expression(s) {
        let idx = config.macros.len();
        config.macros.push(m);
        return Ok(Descriptor {
            op: Op::Macro,
            data: DescriptorData::MacroOp(DescMacro { macro_idx: idx as i16 }),
        });
    }

    // Zero-argument keywords accepted without parentheses (C syntax: `x = clear`).
    match s.trim() {
        "clear"     => return Ok(Descriptor { op: Op::Clear,           data: DescriptorData::None }),
        "repeat"    => return Ok(Descriptor { op: Op::Repeat,          data: DescriptorData::None }),
        "scrolloff" => return Ok(Descriptor { op: Op::ScrollToggleOff, data: DescriptorData::None }),
        _ => {}
    }

    if let Some((fn_name, args)) = parse_fn(s) {
        // Handle lettermod special case
        if fn_name == "lettermod" {
            if args.len() != 4 {
                return Err("lettermod requires 4 arguments".to_string());
            }
            let expanded = format!("overloadi({}, overloadt2({}, {}, {}), {})", args[1], args[0], args[1], args[3], args[2]);
            return config_parse_descriptor(&expanded, config, ctx);
        }

        if fn_name == "oneshot" && args.len() >= 2 {
             if args.len() > 3 {
                 return Err("oneshot supports at most 3 layers".to_string());
             }
             let mut idxs = [-1; 3];
             for (i, arg) in args.iter().enumerate() {
                 if arg == "main" {
                     return Err("the main layer cannot be toggled".to_string());
                 }
                 if let Some(idx) = config_get_layer_index(config, arg) {
                     if config.layers[idx].layer_type == LayerType::Layout {
                         return Err(format!("{arg} is not a valid layer"));
                     }
                     idxs[i] = idx as i16;
                 } else {
                     return Err(format!("{arg} is not a valid layer"));
                 }
             }
             return Ok(Descriptor {
                 op: Op::OneshotMulti,
                 data: DescriptorData::LayerMulti(DescLayerMulti { idx: idxs }),
             });
        }

        // Generic action handling
        let actions = [
            ("swap", Op::Swap, vec!["layer"]),
            ("clear", Op::Clear, vec![]),
            ("oneshot", Op::Oneshot, vec!["layer"]),
            ("toggle", Op::Toggle, vec!["layer"]),
            ("clearm", Op::ClearM, vec!["macro"]),
            ("swapm", Op::SwapM, vec!["layer", "macro"]),
            ("togglem", Op::ToggleM, vec!["layer", "macro"]),
            ("layerm", Op::LayerM, vec!["layer", "macro"]),
            ("oneshotm", Op::OneshotM, vec!["layer", "macro"]),
            ("oneshotk", Op::OneshotK, vec!["layer", "keysequence_descriptor"]),
            ("layer", Op::Layer, vec!["layer"]),
            ("overload", Op::Overload, vec!["layer", "descriptor"]),
            ("overloadt", Op::OverloadTimeout, vec!["layer", "descriptor", "timeout"]),
            ("overloadt2", Op::OverloadTimeoutTap, vec!["layer", "descriptor", "timeout"]),
            ("overloadi", Op::OverloadIdleTimeout, vec!["descriptor", "descriptor", "timeout"]),
            ("timeout", Op::Timeout, vec!["descriptor", "timeout", "descriptor"]),
            ("macro2", Op::Macro2, vec!["timeout", "timeout", "macro"]),
            ("setlayout", Op::Layout, vec!["layout"]),
            ("repeat", Op::Repeat, vec![]),
            ("scrollon", Op::ScrollToggleOn, vec!["sensitivity"]),
            ("scrolloff", Op::ScrollToggleOff, vec![]),
            ("scrollt", Op::ScrollToggle, vec!["sensitivity"]),
            ("scroll", Op::Scroll, vec!["sensitivity"]),
            // Deprecated
            ("overload2", Op::OverloadTimeout, vec!["layer", "descriptor", "timeout"]),
            ("overload3", Op::OverloadTimeoutTap, vec!["layer", "descriptor", "timeout"]),
            ("toggle2", Op::ToggleM, vec!["layer", "macro"]),
            ("swap2", Op::SwapM, vec!["layer", "macro"]),
        ];

        for (name, op, arg_types) in actions {
            if fn_name == name {
                if args.len() != arg_types.len() {
                    return Err(format!("{name} requires {} arguments", arg_types.len()));
                }
                
                // We need to parse arguments. This is tricky because we might need to mutate config (for nested descriptors/macros)
                // In Rust, we'll collect the results then build the Descriptor.
                
                let mut parsed_args = Vec::new();
                for (i, arg_type) in arg_types.iter().enumerate() {
                    let arg_str = &args[i];
                    match *arg_type {
                        "layer" => {
                            if arg_str == "main" {
                                return Err("the main layer cannot be toggled".to_string());
                            }
                            if let Some(idx) = config_get_layer_index(config, arg_str) {
                                if config.layers[idx].layer_type == LayerType::Layout {
                                    return Err(format!("{arg_str} is not a valid layer"));
                                }
                                parsed_args.push(idx as i16);
                            } else {
                                return Err(format!("{arg_str} is not a valid layer"));
                            }
                        }
                        "layout" => {
                            if let Some(idx) = config_get_layer_index(config, arg_str) {
                                if idx != 0 && config.layers[idx].layer_type != LayerType::Layout {
                                    return Err(format!("{arg_str} is not a valid layout"));
                                }
                                parsed_args.push(idx as i16);
                            } else {
                                return Err(format!("{arg_str} is not a valid layout"));
                            }
                        }
                        "descriptor" | "keysequence_descriptor" => {
                            let desc = config_parse_descriptor(arg_str, config, ctx)?;
                            if *arg_type == "keysequence_descriptor" && desc.op != Op::KeySequence {
                                return Err(format!("{arg_str} is not a valid keysequence"));
                            }
                            let idx = config.descriptors.len();
                            config.descriptors.push(desc);
                            parsed_args.push(idx as i16);
                        }
                        "timeout" | "sensitivity" => {
                            parsed_args.push(arg_str.parse::<i16>().map_err(|_| format!("Invalid number: {arg_str}"))?);
                        }
                        "macro" => {
                            let m = config_parse_macro_expression(arg_str)?;
                            let idx = config.macros.len();
                            config.macros.push(m);
                            parsed_args.push(idx as i16);
                        }
                        _ => unreachable!(),
                    }
                }

                let data = match op {
                    Op::Swap | Op::Oneshot | Op::Toggle | Op::Layer | Op::Layout => DescriptorData::Layer(DescLayer { idx: parsed_args[0] }),
                    Op::ClearM => DescriptorData::MacroOp(DescMacro { macro_idx: parsed_args[0] }),
                    Op::SwapM | Op::ToggleM | Op::LayerM | Op::OneshotM => DescriptorData::LayerMacro(DescLayerMacro { idx: parsed_args[0], macro_idx: parsed_args[1] }),
                    Op::OneshotK | Op::Overload => DescriptorData::Overload(DescOverload { layer_idx: parsed_args[0], action_idx: parsed_args[1] }),
                    Op::OverloadTimeout | Op::OverloadTimeoutTap => DescriptorData::OverloadTo(DescOverloadTo { layer_idx: parsed_args[0], action_idx: parsed_args[1], timeout: parsed_args[2] as u16 }),
                    Op::OverloadIdleTimeout => DescriptorData::OverloadIdle(DescOverloadIdle { action1_idx: parsed_args[0], action2_idx: parsed_args[1], timeout: parsed_args[2] as u16 }),
                    Op::Timeout => DescriptorData::TimeoutOp(DescTimeout { action1_idx: parsed_args[0], timeout: parsed_args[1] as u16, action2_idx: parsed_args[2] }),
                    Op::Macro2 => DescriptorData::Macro2(DescMacro2 { delay: parsed_args[0] as u16, interval: parsed_args[1] as u16, macro_idx: parsed_args[2] }),
                    Op::ScrollToggleOn | Op::ScrollToggle | Op::Scroll => DescriptorData::Scroll(DescScroll { sensitivity: parsed_args[0] }),
                    _ => DescriptorData::None,
                };

                return Ok(Descriptor { op, data });
            }
        }
    }

    Err("invalid key or action".to_string())
}
