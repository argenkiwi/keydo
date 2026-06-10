use crate::config::*;
use crate::config_impl::*;
use crate::config_parse::{config_parse_descriptor, config_parse_macro_expression, ParseCtx};
use crate::keys::*;
use crate::macro_types::MacroEntryType;

fn make_base_config() -> Config {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    cfg
}

// ── config_parse_descriptor ───────────────────────────────────────────────────

#[test]
fn descriptor_returns_keysequence_op_for_simple_key() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("a", &mut cfg, &mut ctx).unwrap();
    assert_eq!(d.op, Op::KeySequence);
}

#[test]
fn descriptor_keysequence_has_correct_keycode() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("a", &mut cfg, &mut ctx).unwrap();
    if let DescriptorData::KeySequence(ks) = d.data {
        assert_eq!(ks.code, KEYD_A);
    } else {
        panic!("expected KeySequence data");
    }
}

#[test]
fn descriptor_keysequence_has_zero_mods_for_bare_key() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("a", &mut cfg, &mut ctx).unwrap();
    if let DescriptorData::KeySequence(ks) = d.data {
        assert_eq!(ks.mods, 0);
    } else {
        panic!("expected KeySequence data");
    }
}

#[test]
fn descriptor_ctrl_a_returns_keysequence_op() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("C-a", &mut cfg, &mut ctx).unwrap();
    assert_eq!(d.op, Op::KeySequence);
}

#[test]
fn descriptor_ctrl_a_has_ctrl_modifier_set() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("C-a", &mut cfg, &mut ctx).unwrap();
    if let DescriptorData::KeySequence(ks) = d.data {
        assert!(ks.mods & MOD_CTRL != 0, "expected MOD_CTRL in mods");
    } else {
        panic!("expected KeySequence data");
    }
}

#[test]
fn descriptor_overload_returns_overload_op() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let d = config_parse_descriptor("overload(control, a)", &mut cfg, &mut ctx).unwrap();
    assert_eq!(d.op, Op::Overload);
}

#[test]
fn descriptor_overload_references_correct_layer_index() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let control_idx = config_get_layer_index(&cfg, "control").unwrap();
    let d = config_parse_descriptor("overload(control, a)", &mut cfg, &mut ctx).unwrap();
    if let DescriptorData::Overload(ov) = d.data {
        assert_eq!(ov.layer_idx, control_idx as i16);
    } else {
        panic!("expected Overload data");
    }
}

#[test]
fn descriptor_returns_error_for_invalid_key() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let res = config_parse_descriptor("notavalidkey", &mut cfg, &mut ctx);
    assert!(res.is_err(), "expected error for invalid key");
}

#[test]
fn descriptor_accepts_deprecated_overload2_syntax() {
    let mut cfg = make_base_config();
    let mut ctx = ParseCtx::new();
    let res = config_parse_descriptor("overload2(control, a, 300)", &mut cfg, &mut ctx);
    assert!(res.is_ok(), "deprecated overload2 should still parse successfully");
}

// ── config_parse_macro_expression ─────────────────────────────────────────────

#[test]
fn macro_expression_produces_keysequence_entry_for_modifier_chord() {
    let m = config_parse_macro_expression("C-h").unwrap();
    assert!(m.sz > 0, "macro should have at least one entry");
    assert_eq!(m.entries[0].entry_type, MacroEntryType::KeySequence);
}

#[test]
fn macro_expression_accepts_unicode_character() {
    let m = config_parse_macro_expression("😄").unwrap();
    assert!(m.sz > 0, "macro for unicode should have entries");
}

// ── config_parse_string ───────────────────────────────────────────────────────

#[test]
fn config_maps_source_key_to_target_key() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\na = b\n").unwrap();
    let main_idx = config_get_layer_index(&cfg, "main").unwrap();
    if let DescriptorData::KeySequence(ks) = cfg.layers[main_idx].keymap[KEYD_A as usize].data {
        assert_eq!(ks.code, KEYD_B, "a should map to b");
    } else {
        panic!("expected KeySequence for a→b mapping");
    }
}

#[test]
fn config_sets_chord_timeout_from_global_section() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[global]\nchord_timeout = 75\n\n[main]\n").unwrap();
    assert_eq!(cfg.chord_interkey_timeout, 75);
}

#[test]
fn config_new_has_default_chord_interkey_timeout() {
    assert_eq!(Config::new().chord_interkey_timeout, 50);
}

#[test]
fn config_new_has_default_macro_timeout() {
    assert_eq!(Config::new().macro_timeout, 600);
}

#[test]
fn config_new_has_default_macro_repeat_timeout() {
    assert_eq!(Config::new().macro_repeat_timeout, 50);
}

#[test]
fn config_marks_main_layer_as_layout() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    let main_idx = config_get_layer_index(&cfg, "main").unwrap();
    assert_eq!(cfg.layers[main_idx].layer_type, LayerType::Layout);
}

#[test]
fn config_wildcard_id_sets_wildcard_flag() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    assert_eq!(cfg.wildcard, 1);
}

#[test]
fn config_wildcard_id_produces_no_entries_in_ids_list() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    assert!(cfg.ids.is_empty());
}

#[test]
fn config_exclusion_id_sets_wildcard_flag() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n-1234:5678\n\n[main]\n").unwrap();
    assert_eq!(cfg.wildcard, 1);
}

#[test]
fn config_exclusion_id_stored_in_ids_list() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n-1234:5678\n\n[main]\n").unwrap();
    assert_eq!(cfg.ids.len(), 1);
}

#[test]
fn config_exclusion_id_has_excluded_flag_and_no_dash_prefix() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n-1234:5678\n\n[main]\n").unwrap();
    assert_eq!(cfg.ids[0].flags, ID_EXCLUDED);
    assert_eq!(cfg.ids[0].id, "1234:5678");
}

#[test]
fn config_composite_layer_has_composite_type() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n\n[nav]\na = b\n\n[nav+control]\n").unwrap();
    let idx = config_get_layer_index(&cfg, "nav+control").unwrap();
    assert_eq!(cfg.layers[idx].layer_type, LayerType::Composite);
}

#[test]
fn config_composite_layer_has_two_constituents() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n\n[nav]\na = b\n\n[nav+control]\n").unwrap();
    let idx = config_get_layer_index(&cfg, "nav+control").unwrap();
    assert_eq!(cfg.layers[idx].nr_constituents, 2);
}

#[test]
fn config_composite_layer_inherits_mappings_from_constituent() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg,
        "[ids]\n*\n\n[main]\n\n[nav]\na = b\n\n[nav+control]\nb = a\n"
    ).unwrap();
    let idx = config_get_layer_index(&cfg, "nav+control").unwrap();
    if let DescriptorData::KeySequence(ks) = cfg.layers[idx].keymap[KEYD_A as usize].data {
        assert_eq!(ks.code, KEYD_B, "composite should inherit a→b from nav constituent");
    } else {
        panic!("composite should inherit a→b from nav constituent");
    }
}

#[test]
fn config_composite_layer_retains_its_own_explicit_mappings() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg,
        "[ids]\n*\n\n[main]\n\n[nav]\na = b\n\n[nav+control]\nb = a\n"
    ).unwrap();
    let idx = config_get_layer_index(&cfg, "nav+control").unwrap();
    if let DescriptorData::KeySequence(ks) = cfg.layers[idx].keymap[KEYD_B as usize].data {
        assert_eq!(ks.code, KEYD_A, "explicit b→a should be present in composite");
    } else {
        panic!("composite should have explicit b→a");
    }
}

#[test]
fn config_include_loads_layer_from_included_file() {
    let dir = std::env::temp_dir();
    let main_path = dir.join("test_keyd_main.conf");
    let inc_path = dir.join("test_keyd_inc.conf");

    std::fs::write(&inc_path, "[extra-layer]\na = b\n").unwrap();
    let main_content = format!("[ids]\n*\n\ninclude {}\n\n[main]\n", inc_path.display());
    std::fs::write(&main_path, &main_content).unwrap();

    let cfg = config_parse(main_path.to_str().unwrap()).unwrap();
    let _ = std::fs::remove_file(&main_path);
    let _ = std::fs::remove_file(&inc_path);

    let idx = config_get_layer_index(&cfg, "extra-layer").unwrap();
    if let DescriptorData::KeySequence(ks) = cfg.layers[idx].keymap[KEYD_A as usize].data {
        assert_eq!(ks.code, KEYD_B, "included layer should have a→b mapping");
    } else {
        panic!("expected a→b in included layer");
    }
}

// ── config_check_match ────────────────────────────────────────────────────────

#[test]
fn config_check_match_returns_positive_when_wildcard_set() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    let r = config_check_match(&cfg, "1234:5678", ID_KEYBOARD | ID_KEY);
    assert!(r > 0, "wildcard should match any device");
}

#[test]
fn config_check_match_returns_2_for_exact_id_match() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n1234:5678\n\n[main]\n").unwrap();
    let r = config_check_match(&cfg, "1234:5678", ID_KEYBOARD | ID_KEY);
    assert_eq!(r, 2, "exact match should return 2");
}

#[test]
fn config_check_match_returns_0_for_unregistered_id() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n1234:5678\n\n[main]\n").unwrap();
    let r = config_check_match(&cfg, "0000:0000", ID_KEYBOARD | ID_KEY);
    assert_eq!(r, 0, "unregistered id should return 0");
}

#[test]
fn config_check_match_returns_0_for_excluded_id() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n-1234:5678\n\n[main]\n").unwrap();
    let r = config_check_match(&cfg, "1234:5678", ID_KEYBOARD | ID_KEY);
    assert_eq!(r, 0, "excluded id should return 0");
}

// ── config_add_entry ──────────────────────────────────────────────────────────

#[test]
fn config_add_entry_updates_keymap_for_valid_expression() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    config_add_entry(&mut cfg, "main.a = b").unwrap();
    let main_idx = config_get_layer_index(&cfg, "main").unwrap();
    if let DescriptorData::KeySequence(ks) = cfg.layers[main_idx].keymap[KEYD_A as usize].data {
        assert_eq!(ks.code, KEYD_B, "a should be bound to b after add_entry");
    } else {
        panic!("expected KeySequence after add_entry");
    }
}

#[test]
fn config_add_entry_returns_error_for_unknown_layer() {
    let mut cfg = Config::new();
    config_parse_string(&mut cfg, "[ids]\n*\n\n[main]\n").unwrap();
    let res = config_add_entry(&mut cfg, "nonexistent_layer.a = b");
    assert!(res.is_err(), "should fail for non-existent layer");
}
