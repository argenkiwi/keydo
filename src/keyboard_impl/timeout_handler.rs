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
        } else if time >= pt_expiration
            || (code != 0 && (pressed != 0 || code == pt_code))
        {
            let action = if time >= pt_expiration { pt_action2 } else { pt_action1 };
            Some((action, false))
        } else {
            None
        };

        if let Some((action, both)) = resolve {
            let dl = pt_dl as i32;
            self.pending_timeout = None;

            if both {
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
                self.execute_descriptor(output, action, pt_code, dl, 0, time);
            } else {
                self.cache_set(pt_code, Some(CacheEntry { code: pt_code, d: action, dl, layer: 0 }));
                self.execute_descriptor(output, action, pt_code, dl, 1, time);
            }
        }
    }
}
