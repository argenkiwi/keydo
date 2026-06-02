use evdev::{uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, Key, RelativeAxisType, AbsoluteAxisType, InputEvent, EventType, UinputAbsSetup, AbsInfo};
use std::io::Write;

pub struct Vkbd {
    device: VirtualDevice,
}

impl Vkbd {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        // Add all keys from KEYCODE_TABLE
        for i in 0..256 {
            if let Some(_) = crate::keys::KEYCODE_TABLE[i] {
                keys.insert(Key::new(i as u16));
            }
        }
        // Add mouse buttons
        for i in 0x110..0x118 {
             keys.insert(Key::new(i as u16));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_X);
        rel_axes.insert(RelativeAxisType::REL_Y);
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_HWHEEL);

        let mut abs_axes = AttributeSet::<AbsoluteAxisType>::new();
        abs_axes.insert(AbsoluteAxisType::ABS_X);
        abs_axes.insert(AbsoluteAxisType::ABS_Y);

        let device = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            // TODO: properly setup absolute axes with range
            .build()?;

        Ok(Vkbd { device })
    }

    pub fn send_key(&mut self, code: u16, pressed: bool) -> anyhow::Result<()> {
        let ev = InputEvent::new(EventType::KEY, code, if pressed { 1 } else { 0 });
        self.device.emit(&[ev])?;
        Ok(())
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let mut events = Vec::new();
        if x != 0 {
            events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, x));
        }
        if y != 0 {
            events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y));
        }
        if !events.is_empty() {
            self.device.emit(&events)?;
        }
        Ok(())
    }

    pub fn mouse_scroll(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let mut events = Vec::new();
        if y != 0 {
            events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, y));
        }
        if x != 0 {
            events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_HWHEEL.0, x));
        }
        if !events.is_empty() {
            self.device.emit(&events)?;
        }
        Ok(())
    }
}
