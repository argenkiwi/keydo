use crate::config::{DeviceId, ID_EXCLUDED, ID_KEYBOARD, ID_MOUSE, ID_TRACKPAD, ID_KEY};
use evdev::{Device, EventType, InputEvent, Key};
use std::path::Path;
use std::fs;

pub struct EvdevDevice {
    pub device: Device,
    pub path: String,
    pub id: String,
    pub capabilities: u8,
}

impl EvdevDevice {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let device = Device::open(path)?;
        let name = device.name().unwrap_or("Unknown");
        
        let mut capabilities = 0u8;
        
        let keys = device.supported_keys();
        let rel_axes = device.supported_relative_axes();
        let abs_axes = device.supported_absolute_axes();

        if keys.is_some() {
            capabilities |= ID_KEY;
        }

        // Logic from resolve_device_capabilities in device.c
        let is_keyboard = if let Some(keys) = keys {
             let keyboard_keys = [
                Key::KEY_1, Key::KEY_2, Key::KEY_3, Key::KEY_4,
                Key::KEY_5, Key::KEY_6, Key::KEY_7, Key::KEY_8,
                Key::KEY_9, Key::KEY_0, Key::KEY_Q, Key::KEY_W,
                Key::KEY_E, Key::KEY_R, Key::KEY_T, Key::KEY_Y,
            ];
            keyboard_keys.iter().all(|&k| keys.contains(k))
        } else {
            false
        };

        if is_keyboard {
            capabilities |= ID_KEYBOARD;
        }

        if rel_axes.is_some() || abs_axes.is_some() {
            capabilities |= ID_MOUSE;
        }

        if abs_axes.is_some() {
            capabilities |= ID_TRACKPAD;
        }

        let input_id = device.input_id();
        // Generate a unique ID similar to generate_uid in device.c
        // For now, just vendor:product:name_hash
        let id = format!("{:04x}:{:04x}:{:08x}", 
            input_id.vendor(), 
            input_id.product(), 
            fxhash::hash32(name)
        );

        Ok(EvdevDevice {
            device,
            path: path.to_string_lossy().to_string(),
            id,
            capabilities,
        })
    }

    pub fn grab(&mut self) -> std::io::Result<()> {
        self.device.grab()
    }

    pub fn ungrab(&mut self) -> std::io::Result<()> {
        self.device.ungrab()
    }

    pub fn set_led(&mut self, led_code: u16, on: bool) {
        let ev = InputEvent::new(EventType::LED, led_code, if on { 1 } else { 0 });
        let _ = self.device.send_events(&[ev]);
    }

    /// Returns whether this device should be grabbed according to the config.
    /// Mirrors keyd's config_check_match + manage_device logic.
    /// Returns true if the device should be grabbed, false if excluded or unmatched.
    pub fn should_grab(&self, ids: &[DeviceId], wildcard: bool) -> bool {
        let flags = self.capabilities;
        for id_entry in ids {
            if self.id.starts_with(&id_entry.id) {
                if id_entry.flags & ID_EXCLUDED != 0 {
                    return false;
                }
                if id_entry.flags & flags != 0 {
                    return true;
                }
            }
        }
        // Wildcard matches keyboards that are not trackpads.
        if wildcard {
            return (flags & ID_KEYBOARD != 0) && (flags & ID_TRACKPAD == 0);
        }
        false
    }
}

pub fn scan_devices() -> Vec<EvdevDevice> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev/input/") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().unwrap().to_string_lossy().starts_with("event") {
                if let Ok(dev) = EvdevDevice::open(&path) {
                    if dev.capabilities != 0 {
                        devices.push(dev);
                    }
                }
            }
        }
    }
    devices
}
