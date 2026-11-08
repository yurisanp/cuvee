//! Time-based motion. Everything reads `now` and a start instant, so a screen's entrance
//! cascade, the bubbles and the breathing dot need no per-widget state. With reduced
//! motion every helper answers its final value.

use std::time::{Duration, Instant};

/// The CSS curve the mockups use: cubic-bezier(.2,.7,.2,1), close to an ease-out quint.
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(4)
}

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub now: Instant,
    pub reduce_motion: bool,
}

impl Clock {
    /// 0 → 1 over `duration`, starting `delay` after `start`.
    pub fn progress(&self, start: Instant, delay: Duration, duration: Duration) -> f32 {
        if self.reduce_motion {
            return 1.0;
        }
        let elapsed = self.now.saturating_duration_since(start);
        if elapsed <= delay {
            return 0.0;
        }
        ease_out((elapsed - delay).as_secs_f32() / duration.as_secs_f32())
    }

    /// The entrance of the i-th item on a screen: fades and rises over 0.8 s, 100 ms apart.
    pub fn rise(&self, start: Instant, index: usize) -> Rise {
        let p = self.progress(
            start,
            Duration::from_millis(100 * index as u64),
            Duration::from_millis(800),
        );
        Rise {
            opacity: p,
            offset: 14.0 * (1.0 - p),
        }
    }

    /// Seconds since `start`, 0 under reduced motion.
    pub fn seconds(&self, start: Instant) -> f32 {
        if self.reduce_motion {
            0.0
        } else {
            self.now.saturating_duration_since(start).as_secs_f32()
        }
    }

    /// 0 → 1 → 0 with `period`, for a breathing dot.
    pub fn breathe(&self, start: Instant, period: f32) -> f32 {
        if self.reduce_motion {
            return 0.0;
        }
        let t = self.seconds(start) % period / period;
        (0.5 - 0.5 * (t * std::f32::consts::TAU).cos()).max(0.0)
    }

    pub fn is_settled(&self, start: Instant, items: usize) -> bool {
        self.reduce_motion
            || self.now.saturating_duration_since(start)
                > Duration::from_millis(800 + 100 * items as u64 + 200)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rise {
    pub opacity: f32,
    /// Pixels still to travel up: used as extra top padding.
    pub offset: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_ends_at_one() {
        assert_eq!(ease_out(0.0), 0.0);
        assert!((ease_out(1.0) - 1.0).abs() < 1e-6);
        assert!(ease_out(0.5) > 0.5);
    }

    #[test]
    fn reduced_motion_is_instant() {
        let c = Clock {
            now: Instant::now(),
            reduce_motion: true,
        };
        let r = c.rise(c.now, 5);
        assert_eq!(r.opacity, 1.0);
        assert_eq!(r.offset, 0.0);
    }
}
