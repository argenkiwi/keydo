use crate::keyboard::OutputEvent;

// ---------------------------------------------------------------------------
// Linux — evdev/uinput backend
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
use evdev::{uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, Key, RelativeAxisType};

#[cfg(target_os = "linux")]
pub struct Vkbd {
    device: VirtualDevice,
}

#[cfg(target_os = "linux")]
impl Vkbd {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        for i in 0..256 {
            if crate::keys::KEYCODE_TABLE[i].is_some() {
                keys.insert(Key::new(i as u16));
            }
        }
        for i in 0x110..0x118 {
            keys.insert(Key::new(i as u16));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_X);
        rel_axes.insert(RelativeAxisType::REL_Y);
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_HWHEEL);

        let device = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            .build()?;

        Ok(Vkbd { device })
    }

    pub fn send_key(&mut self, code: u16, pressed: bool) -> anyhow::Result<()> {
        let ev = InputEvent::new(EventType::KEY, code, if pressed { 1 } else { 0 });
        self.device.emit(&[ev])?;
        Ok(())
    }

    pub fn as_raw_fd(&self) -> i32 {
        use std::os::unix::io::AsRawFd;
        self.device.as_raw_fd()
    }

    pub fn drain_led_events(&mut self) -> Vec<(u16, bool)> {
        let mut out = Vec::new();
        if let Ok(events) = self.device.fetch_events() {
            for ev in events {
                if ev.event_type() == EventType::LED {
                    out.push((ev.code(), ev.value() != 0));
                }
            }
        }
        out
    }

    pub fn send_event(&mut self, ev: &OutputEvent) -> anyhow::Result<()> {
        match ev {
            OutputEvent::Key(code, pressed) => self.send_key(*code, *pressed),
            OutputEvent::Scroll(x, y)       => self.mouse_scroll(*x, *y),
            OutputEvent::Command(cmd) => {
                std::process::Command::new("sh").arg("-c").arg(cmd).spawn().ok();
                Ok(())
            }
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let mut events = Vec::new();
        if x != 0 { events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, x)); }
        if y != 0 { events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y)); }
        if !events.is_empty() { self.device.emit(&events)?; }
        Ok(())
    }

    pub fn mouse_scroll(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        let mut events = Vec::new();
        if y != 0 { events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, y)); }
        if x != 0 { events.push(InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_HWHEEL.0, x)); }
        if !events.is_empty() { self.device.emit(&events)?; }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// macOS — CGEventPost backend
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub struct Vkbd;

#[cfg(target_os = "macos")]
impl Vkbd {
    pub fn new(_name: &str) -> anyhow::Result<Self> {
        crate::macos::vkbd::init();
        Ok(Vkbd)
    }

    pub fn send_event(&mut self, ev: &OutputEvent) -> anyhow::Result<()> {
        crate::macos::vkbd::send_event(ev)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> anyhow::Result<()> {
        crate::macos::vkbd::mouse_move(x, y);
        Ok(())
    }
}
