use crate::keyboard_types::*;

impl Keyboard {
    pub(super) fn calculate_main_loop_timeout(&mut self, time: i64) -> i64 {
        let mut earliest: i64 = 0;
        let mut n = 0;
        for i in 0..self.nr_timeouts {
            if self.timeouts[i] > time {
                if earliest == 0 || self.timeouts[i] < earliest {
                    earliest = self.timeouts[i];
                }
                self.timeouts[n] = self.timeouts[i];
                n += 1;
            }
        }
        self.nr_timeouts = n;
        if earliest > 0 { earliest - time } else { 0 }
    }

    pub(super) fn schedule_timeout(&mut self, deadline_ms: i64) {
        if self.nr_timeouts < self.timeouts.len() {
            self.timeouts[self.nr_timeouts] = deadline_ms;
            self.nr_timeouts += 1;
        }
    }
}
