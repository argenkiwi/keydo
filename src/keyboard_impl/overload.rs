use crate::config::*;
use crate::keyboard_types::*;

impl Keyboard {
    /// Returns `true` if the event was consumed by the pending overload state machine.
    pub(super) fn handle_pending_overload<O: Output>(
        &mut self, output: &mut O, code: u8, pressed: u8, time: i64,
    ) -> bool {
        if self.pending_overload.is_none() {
            return false;
        }

        // Let through key-up events for keys that were already held *before* the
        // pending overload started (they won't be in the queue and aren't the overload key).
        if code != 0 && pressed == 0 {
            let known = if let Some(po) = self.pending_overload.as_ref() {
                code == po.code
                    || po.queue[..po.queue_sz].iter().any(|e| e.code == code)
            } else {
                return false;
            };
            if !known {
                return false;
            }
        }

        // Enqueue real (non-synthetic) events.
        if code != 0 {
            if let Some(po) = self.pending_overload.as_mut() {
                if po.queue_sz < po.queue.len() {
                    po.queue[po.queue_sz] = KeyEvent { code, pressed, timestamp: time as i32 };
                    po.queue_sz += 1;
                }
            }
        }

        // Decide if we can resolve now.
        let resolve: Option<Descriptor> = if let Some(po) = self.pending_overload.as_ref() {
            if time >= po.expiration {
                Some(po.action2)
            } else if code == po.code && pressed == 0 {
                Some(po.action1)
            } else if po.resolve_on_interrupt != 0 && pressed == 0 {
                Some(po.action2)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(action) = resolve {
            let (overload_code, dl, queue_snap, queue_sz) = if let Some(po) = self.pending_overload.as_ref() {
                let sz = po.queue_sz;
                let mut q = [KeyEvent { code: 0, pressed: 0, timestamp: 0 }; 32];
                q[..sz].copy_from_slice(&po.queue[..sz]);
                (po.code, po.dl as i32, q, sz)
            } else {
                return true;
            };

            self.pending_overload = None;

            self.cache_set(overload_code, Some(CacheEntry {
                code: overload_code, d: action, dl, layer: 0,
            }));
            self.execute_descriptor(output, action, overload_code, dl, 1, time);

            if queue_sz > 0 {
                self.kbd_process_events(output, &queue_snap[..queue_sz]);
            }
        }

        true
    }
}
