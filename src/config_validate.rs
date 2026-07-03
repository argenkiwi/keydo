//! Semantic validation of a parsed `Config` — catches issues that would cause
//! runtime panics or silent misbehaviour before the daemon starts.

use crate::config::*;

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct ValidationError {
    pub severity: Severity,
    pub message: String,
}

impl ValidationError {
    fn error(msg: impl Into<String>) -> Self {
        Self { severity: Severity::Error, message: msg.into() }
    }
    #[allow(dead_code)]
    fn warning(msg: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, message: msg.into() }
    }
}

/// Validate `config` after parsing. Returns a list of errors and warnings.
/// Any `Severity::Error` should prevent the daemon from loading the config.
pub fn validate(config: &Config) -> Vec<ValidationError> {
    let mut out = Vec::new();
    check_layer_count(config, &mut out);
    check_composite_cycles(config, &mut out);
    check_descriptor_index_bounds(config, &mut out);
    out
}

fn check_layer_count(config: &Config, out: &mut Vec<ValidationError>) {
    if config.layers.len() > MAX_LAYERS {
        out.push(ValidationError::error(format!(
            "config defines {} layers but the maximum is {}; \
             layers beyond index {} will cause a panic at runtime",
            config.layers.len(), MAX_LAYERS, MAX_LAYERS - 1,
        )));
    }
}

fn check_composite_cycles(config: &Config, out: &mut Vec<ValidationError>) {
    let n = config.layers.len();
    let mut visited = vec![false; n];
    let mut in_stack = vec![false; n];

    for start in 0..n {
        if !visited[start] && dfs_has_cycle(config, start, &mut visited, &mut in_stack) {
            out.push(ValidationError::error(format!(
                "layer '{}' is part of a composite-layer cycle; \
                 activating it will cause infinite recursion",
                config.layers[start].name,
            )));
        }
    }
}

fn dfs_has_cycle(
    config: &Config,
    node: usize,
    visited: &mut Vec<bool>,
    in_stack: &mut Vec<bool>,
) -> bool {
    if in_stack[node] { return true; }
    if visited[node]  { return false; }

    visited[node]  = true;
    in_stack[node] = true;

    if config.layers[node].layer_type == LayerType::Composite {
        for ci in 0..config.layers[node].nr_constituents {
            let child = config.layers[node].constituents[ci] as usize;
            if child < config.layers.len() && dfs_has_cycle(config, child, visited, in_stack) {
                in_stack[node] = false;
                return true;
            }
        }
    }

    in_stack[node] = false;
    false
}

fn check_descriptor_index_bounds(config: &Config, out: &mut Vec<ValidationError>) {
    let nl = config.layers.len();
    let nm = config.macros.len();
    let nd = config.descriptors.len();
    let nc = config.commands.len();

    let check_desc = |d: &Descriptor, ctx: &str, out: &mut Vec<ValidationError>| {
        match d.data {
            DescriptorData::Layer(l) => {
                if l.idx < 0 || l.idx as usize >= nl {
                    out.push(ValidationError::error(format!(
                        "{ctx}: layer index {} is out of range (0..{nl})", l.idx
                    )));
                }
            }
            DescriptorData::LayerMulti(lm) => {
                for &idx in lm.idx.iter().filter(|&&i| i != -1) {
                    if idx < 0 || idx as usize >= nl {
                        out.push(ValidationError::error(format!(
                            "{ctx}: layer index {idx} is out of range (0..{nl})"
                        )));
                    }
                }
            }
            DescriptorData::MacroOp(m) => {
                if m.macro_idx < 0 || m.macro_idx as usize >= nm {
                    out.push(ValidationError::error(format!(
                        "{ctx}: macro index {} is out of range (0..{nm})", m.macro_idx
                    )));
                }
            }
            DescriptorData::Command(c) => {
                if c.cmd_idx < 0 || c.cmd_idx as usize >= nc {
                    out.push(ValidationError::error(format!(
                        "{ctx}: command index {} is out of range (0..{nc})", c.cmd_idx
                    )));
                }
            }
            DescriptorData::LayerMacro(lm) => {
                if lm.idx < 0 || lm.idx as usize >= nl {
                    out.push(ValidationError::error(format!(
                        "{ctx}: layer index {} is out of range (0..{nl})", lm.idx
                    )));
                }
                if lm.macro_idx < 0 || lm.macro_idx as usize >= nm {
                    out.push(ValidationError::error(format!(
                        "{ctx}: macro index {} is out of range (0..{nm})", lm.macro_idx
                    )));
                }
            }
            DescriptorData::Overload(ov) => {
                if ov.layer_idx < 0 || ov.layer_idx as usize >= nl {
                    out.push(ValidationError::error(format!(
                        "{ctx}: layer index {} is out of range (0..{nl})", ov.layer_idx
                    )));
                }
                if ov.action_idx < 0 || ov.action_idx as usize >= nd {
                    out.push(ValidationError::error(format!(
                        "{ctx}: descriptor index {} is out of range (0..{nd})", ov.action_idx
                    )));
                }
            }
            DescriptorData::OverloadTo(ov) => {
                if ov.layer_idx < 0 || ov.layer_idx as usize >= nl {
                    out.push(ValidationError::error(format!(
                        "{ctx}: layer index {} is out of range (0..{nl})", ov.layer_idx
                    )));
                }
                if ov.action_idx < 0 || ov.action_idx as usize >= nd {
                    out.push(ValidationError::error(format!(
                        "{ctx}: descriptor index {} is out of range (0..{nd})", ov.action_idx
                    )));
                }
            }
            DescriptorData::OverloadIdle(ov) => {
                for idx in [ov.action1_idx, ov.action2_idx] {
                    if idx < 0 || idx as usize >= nd {
                        out.push(ValidationError::error(format!(
                            "{ctx}: descriptor index {idx} is out of range (0..{nd})"
                        )));
                    }
                }
            }
            DescriptorData::TimeoutOp(to) => {
                for idx in [to.action1_idx, to.action2_idx] {
                    if idx < 0 || idx as usize >= nd {
                        out.push(ValidationError::error(format!(
                            "{ctx}: descriptor index {idx} is out of range (0..{nd})"
                        )));
                    }
                }
            }
            DescriptorData::Macro2(m) => {
                if m.macro_idx < 0 || m.macro_idx as usize >= nm {
                    out.push(ValidationError::error(format!(
                        "{ctx}: macro index {} is out of range (0..{nm})", m.macro_idx
                    )));
                }
            }
            _ => {}
        }
    };

    for (_li, layer) in config.layers.iter().enumerate() {
        for (ki, d) in layer.keymap.iter().enumerate() {
            if d.op != Op::KeySequence || !matches!(d.data, DescriptorData::None) {
                let ctx = format!("layer '{}' key {ki}", layer.name);
                check_desc(d, &ctx, out);
            }
        }
        for ci in 0..layer.nr_chords {
            let ctx = format!("layer '{}' chord {ci}", layer.name);
            check_desc(&layer.chords[ci].d, &ctx, out);
        }
    }

    for (di, d) in config.descriptors.iter().enumerate() {
        let ctx = format!("descriptor[{di}]");
        check_desc(d, &ctx, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> Config {
        Config::new()
    }

    #[test]
    fn validates_clean_config_with_no_errors() {
        let config = empty_config();
        let errors = validate(&config);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn detects_too_many_layers() {
        let mut config = empty_config();
        for i in 0..=MAX_LAYERS {
            config.layers.push(Layer::new(format!("layer_{i}")));
        }
        let errors = validate(&config);
        assert!(errors.iter().any(|e| matches!(e.severity, Severity::Error) && e.message.contains("maximum")));
    }

    #[test]
    fn detects_out_of_range_layer_index_in_keymap() {
        use crate::config::*;
        let mut config = empty_config();
        config.layers.push(Layer::new("main".to_string()));
        // Point a keymap entry at layer index 99, which doesn't exist.
        config.layers[0].keymap[10] = Descriptor {
            op: Op::Layer,
            data: DescriptorData::Layer(DescLayer { idx: 99 }),
        };
        let errors = validate(&config);
        assert!(errors.iter().any(|e| matches!(e.severity, Severity::Error) && e.message.contains("out of range")));
    }
}
