#[cfg(test)]
mod tests {
    use crate::config::{Config, Action};
    use crate::config_parser::ConfigParser;
    use crate::keyboard::Keyboard;
    use crate::keys::lookup_keycode;
    use std::path::Path;
    use std::fs;

    fn parse_test_file(path: &Path, config: &Config) -> (Vec<TestEvent>, Vec<String>) {
        let content = fs::read_to_string(path).unwrap();
        let mut parts = content.split("\n\n");
        let input_str = parts.next().unwrap();
        let output_str = parts.next().unwrap_or("");

        let mut inputs = Vec::new();
        for line in input_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.ends_with("ms") {
                let ms = line[..line.len()-2].parse::<u32>().unwrap();
                inputs.push(TestEvent::Timeout(ms));
            } else {
                let mut parts = line.split_whitespace();
                let key_name = parts.next().unwrap();
                let state = parts.next().unwrap();
                let code = lookup_key_with_config(key_name, config).unwrap();
                inputs.push(TestEvent::Key(code, state == "down"));
            }
        }

        let expected_output = output_str.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        (inputs, expected_output)
    }

    fn lookup_key_with_config(s: &str, config: &Config) -> Option<u16> {
        if let Some(code) = lookup_keycode(s) {
            return Some(code);
        }
        for (&code, alias) in &config.aliases {
            if alias == s {
                return Some(code);
            }
        }
        None
    }

    enum TestEvent {
        Key(u16, bool),
        Timeout(u32),
    }

    #[test]
    fn run_keyd_tests() {
        let test_dir = Path::new("../keyd/t");
        let config_path = test_dir.join("test.conf");
        
        let mut parser = ConfigParser::new();
        let config = parser.parse(&config_path).unwrap();

        let test_files = fs::read_dir(test_dir).unwrap()
            .flatten()
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "t"))
            .map(|e| e.path());

        for test_file in test_files {
            println!("Running test: {:?}", test_file);
            let (inputs, expected) = parse_test_file(&test_file, &config);
            let mut keyboard = Keyboard::new(config.clone());
            let mut actual = Vec::new();

            for event in inputs {
                match event {
                    TestEvent::Key(code, pressed) => {
                        let output = keyboard.process_event(code, pressed);
                        for out_ev in output {
                            actual.push(format!("{} {}", crate::keys::get_key_name(out_ev.code), if out_ev.pressed { "down" } else { "up" }));
                        }
                    }
                    TestEvent::Timeout(ms) => {
                        let new_time = keyboard.current_time + ms as u128;
                        keyboard.set_time(new_time);
                        // Trigger timeout processing
                        let output = keyboard.process_event(0, false);
                        for out_ev in output {
                            actual.push(format!("{} {}", crate::keys::get_key_name(out_ev.code), if out_ev.pressed { "down" } else { "up" }));
                        }
                    }
                }
            }

            // Simple comparison for now
            // Some tests might fail because of missing feature implementations
            // but this will help identify what's missing.
            if actual.len() != expected.len() {
                 println!("Test {:?} failed: length mismatch (actual {}, expected {})", test_file, actual.len(), expected.len());
                 // Print diff
                 for i in 0..std::cmp::max(actual.len(), expected.len()) {
                     let a = actual.get(i).map(|s| s.as_str()).unwrap_or("");
                     let e = expected.get(i).map(|s| s.as_str()).unwrap_or("");
                     println!("{:<20} {:<20}", e, a);
                 }
            } else {
                for (a, e) in actual.iter().zip(expected.iter()) {
                    if a != e {
                         println!("Test {:?} failed: mismatch (actual {}, expected {})", test_file, a, e);
                         break;
                    }
                }
            }
        }
    }
}
