use std::cell::Cell;
use web_time::Instant;

thread_local! {
    static LABEL: Cell<&'static str> = const { Cell::new("frame") };
    static SINCE: Cell<Option<Instant>> = const { Cell::new(None) };
    static WORST: Cell<(&'static str, f32)> = const { Cell::new(("frame", 0.0)) };
}

fn close_segment() {
    let (Some(since), label) = (SINCE.with(|s| s.get()), LABEL.with(|l| l.get())) else {
        return;
    };
    let ms = since.elapsed().as_secs_f32() * 1000.0;
    WORST.with(|w| {
        if ms > w.get().1 || w.get().0 == "frame" {
            w.set((label, ms));
        }
    });
    SINCE.with(|s| s.set(Some(Instant::now())));
}

/// Marks the start of a named block. The frame is reported under the block that
/// actually held the CPU, not under the last label touched before the frame ended.
pub fn label(name: &'static str) {
    close_segment();
    LABEL.with(|l| l.set(name));
    SINCE.with(|s| s.set(Some(Instant::now())));
}

pub fn current() -> &'static str {
    LABEL.with(|l| l.get())
}

fn reset() {
    LABEL.with(|l| l.set("frame"));
    SINCE.with(|s| s.set(None));
    WORST.with(|w| w.set(("frame", 0.0)));
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hitch {
    pub frame: u64,
    pub ms: f32,
    pub label: &'static str,
    pub segment_ms: f32,
}

pub struct Watchdog {
    threshold_ms: f32,
    pub worst: Hitch,
    pub hitches: u64,
    pub total: u64,
    pub log: bool,
}

impl Watchdog {
    pub fn new(threshold_ms: f32, log: bool) -> Self {
        Watchdog {
            threshold_ms,
            worst: Hitch {
                frame: 0,
                ms: 0.0,
                label: "frame",
                segment_ms: 0.0,
            },
            hitches: 0,
            total: 0,
            log,
        }
    }

    pub fn from_env() -> Self {
        let get = |k: &str| std::env::var(k).ok();
        let threshold = get("UME_HITCH_MS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(50.0);
        Watchdog::new(threshold, get("UME_WATCHDOG").is_some_and(|v| v != "0"))
    }

    pub fn record(&mut self, ms: f32) -> Option<Hitch> {
        close_segment();
        let (label, segment_ms) = WORST.with(|w| w.get());
        reset();
        self.total += 1;
        let hitch = Hitch {
            frame: self.total,
            ms,
            label,
            segment_ms,
        };
        if ms > self.worst.ms {
            self.worst = hitch.clone();
        }
        if ms < self.threshold_ms {
            return None;
        }
        self.hitches += 1;
        if self.log {
            eprintln!(
                "[watchdog] frame {} took {:.1} ms ({:.1} ms in '{}')",
                hitch.frame, ms, segment_ms, label
            );
        }
        Some(hitch)
    }

    pub fn summary(&self) -> String {
        format!(
            "frames {} hitches {} worst {:.1} ms in '{}' ({:.1} ms of it) (frame {})",
            self.total,
            self.hitches,
            self.worst.ms,
            self.worst.label,
            self.worst.segment_ms,
            self.worst.frame
        )
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
        label("a");
        w.record(30.0);
        label("b");
        w.record(20.0);
        assert_eq!(w.worst.label, "a");
    }

    #[test]
    fn reports_the_block_that_held_the_cpu_not_the_last_label() {
        label("load:character");
        std::thread::sleep(std::time::Duration::from_millis(25));
        label("frame-tail");
        let h = Watchdog::new(0.1, false).record(30.0).expect("hitch");
        assert_eq!(h.label, "load:character");
        assert!(h.segment_ms >= 20.0, "segment was {} ms", h.segment_ms);
    }
}
