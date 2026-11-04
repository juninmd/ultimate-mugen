use std::cell::Cell;

thread_local! { static LABEL: Cell<&'static str> = const { Cell::new("frame") }; }

pub fn label(name: &'static str) { LABEL.with(|l| l.set(name)); }
pub fn current() -> &'static str { LABEL.with(|l| l.get()) }
fn reset() { label("frame"); }

#[derive(Clone, Debug, PartialEq)]
pub struct Hitch { pub frame: u64, pub ms: f32, pub label: &'static str }

pub struct Watchdog { threshold_ms: f32, pub worst: Hitch, pub hitches: u64, pub total: u64, pub log: bool }

impl Watchdog {
    pub fn new(threshold_ms: f32, log: bool) -> Self {
        Watchdog { threshold_ms, worst: Hitch { frame: 0, ms: 0.0, label: "frame" }, hitches: 0, total: 0, log }
    }

    pub fn from_env() -> Self {
        let get = |k: &str| std::env::var(k).ok();
        let threshold = get("UME_HITCH_MS").and_then(|v| v.parse().ok()).unwrap_or(50.0);
        Watchdog::new(threshold, get("UME_WATCHDOG").is_some_and(|v| v != "0"))
    }

    pub fn record(&mut self, ms: f32) -> Option<Hitch> {
        let label = current();
        reset();
        self.total += 1;
        let hitch = Hitch { frame: self.total, ms, label };
        if ms > self.worst.ms { self.worst = hitch.clone(); }
        if ms < self.threshold_ms { return None; }
        self.hitches += 1;
        if self.log { eprintln!("[watchdog] frame {} took {:.1} ms in '{}'", hitch.frame, ms, hitch.label); }
        Some(hitch)
    }

    pub fn summary(&self) -> String {
        format!("frames {} hitches {} worst {:.1} ms in '{}' (frame {})", self.total, self.hitches, self.worst.ms, self.worst.label, self.worst.frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_slow_frames_with_their_label() {
        let mut w = Watchdog::new(50.0, false);
        assert!(w.record(16.0).is_none());
        label("load:stage");
        let h = w.record(120.0).expect("hitch");
        assert_eq!(h.label, "load:stage");
        assert_eq!(current(), "frame");
        assert_eq!((w.hitches, w.total), (1, 2));
        assert_eq!(w.worst.ms, 120.0);
    }

    #[test]
    fn keeps_the_worst_frame() {
        let mut w = Watchdog::new(10.0, false);
        label("a"); w.record(30.0);
        label("b"); w.record(20.0);
        assert_eq!(w.worst.label, "a");
    }
}
