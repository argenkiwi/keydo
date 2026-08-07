use crate::config::*;
use crate::keyboard_types::*;

impl Keyboard {
    pub(super) fn handle_pending_timeout<O: Output>(&mut self, output: &mut O, code: u8, pressed: u8, time: i64) {
        let (pt_code, pt_dl, pt_spontaneous, pt_expiration, pt_action1, pt_action2) =
            match self.pending_timeout.as_ref() {
                None => return,
                Some(pt) => {
                    if pressed == 0 && pt.code == code && time == pt.activation_time {
                        return;
                    }
                    (pt.code, pt.dl, pt.spontaneous, pt.expiration, pt.action1, pt.action2)
                }
            };

        let resolve: Option<(Descriptor, bool)> = if pt_spontaneous != 0 {
            if time >= pt_expiration || code != 0 {
                let action = if time >= pt_expiration { pt_action2 } else { pt_action1 };
                Some((action, true))
            } else {
                None
            }
        } else if time >= pt_expiration || code != 0 {
            // Any other real key event -- a press *or a release* -- counts
            // as an interrupt, not just a press or this key's own release.
            // Ignoring another key's release let a long, deliberate hold
            // (e.g. holding a home-row mod while a chord-participant key
            // gets pressed and released underneath it) run out this timer's
            // own clock uninterrupted, falling through to its "held alone
            // too long" fallback even though something else was clearly
            // happening at the same time.
            let action = if time >= pt_expiration { pt_action2 } else { pt_action1 };
            Some((action, false))
        } else {
            None
        };

        if let Some((action, both)) = resolve {
            let dl = pt_dl as i32;
            self.pending_timeout = None;
            log::trace!(
                "timeout fire: code {pt_code} resolved via {} (expired={})",
                if both { "action1+action2 (tap)" } else { "single action (hold)" },
                time >= pt_expiration
            );

            if both {
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
                self.execute_descriptor(output, action, pt_code, dl, 0, time);
            } else {
                let cached = self.cache_set(pt_code, Some(CacheEntry { code: pt_code, d: action, dl, layer: 0 }));
                debug_assert!(
                    cached,
                    "cache table full ({CACHE_SIZE} slots) inserting timeout code {pt_code}; \
                     its key-up will be silently dropped"
                );
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
            }
        }
    }
}
