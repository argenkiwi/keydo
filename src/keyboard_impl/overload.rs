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

        // Phase 2 entry: overloadr only. H released while interrupt keys are still held →
        // don't resolve yet; instead record the timestamp and watch for the interrupt release.
        // queue_sz > 1 means the queue holds more than just the H-release itself.
        if let Some(po) = self.pending_overload.as_ref() {
            if po.release_gap_threshold > 0
                && po.phase2_start == 0
                && code == po.code
                && pressed == 0
                && po.queue_sz > 1
            {
                self.pending_overload.as_mut().unwrap().phase2_start = time;
                return true;
            }
        }

        // Decide if we can resolve now.
        let resolve: Option<Descriptor> = if let Some(po) = self.pending_overload.as_ref() {
            if po.release_gap_threshold > 0 && po.phase2_start > 0 {
                // Phase 2 (overloadr): H already released. Resolve when interrupt key releases
                // or the tap-timeout timer fires. In both cases, compare the elapsed gap against
                // the following_timeout threshold: small gap → HOLD, large gap → TAP.
                let threshold = po.release_gap_threshold as i64;
                if time >= po.expiration {
                    let gap = time - po.phase2_start;
                    if gap <= threshold { Some(po.action2) } else { Some(po.action1) }
                } else if code != 0 && pressed == 0 && code != po.code {
                    let gap = time - po.phase2_start;
                    if gap <= threshold { Some(po.action2) } else { Some(po.action1) }
                } else {
                    None
                }
            } else if time >= po.expiration {
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
