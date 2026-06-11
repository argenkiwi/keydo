use crate::keyboard_types::*;

impl Keyboard {
    pub(super) fn cache_set(&mut self, code: u8, ent: Option<CacheEntry>) -> bool {
        let mut slot = None;
        for i in 0..16 {
            if let Some(c) = self.cache[i] {
                if c.code == code {
                    slot = Some(i);
                    break;
                }
            } else if slot.is_none() {
                slot = Some(i);
            }
        }

        if let Some(i) = slot {
            if let Some(mut e) = ent {
                e.code = code;
                self.cache[i] = Some(e);
            } else {
                self.cache[i] = None;
            }
            true
        } else {
            false
        }
    }

    pub(super) fn cache_get(&self, code: u8) -> Option<CacheEntry> {
        for i in 0..16 {
            if let Some(c) = self.cache[i].filter(|c| c.code == code) {
                return Some(c);
            }
        }
        None
    }
}
