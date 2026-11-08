use super::*;


/// Counters shared with the boot screen while the roster is catalogued on a worker thread.
#[derive(Default)]
pub struct ScanProgress {
    pub(crate) files: std::sync::atomic::AtomicUsize,
    pub(crate) chars: std::sync::atomic::AtomicUsize,
    pub(crate) stages: std::sync::atomic::AtomicUsize,
}

impl ScanProgress {
    pub fn counts(&self) -> (usize, usize, usize) {
        use std::sync::atomic::Ordering::Relaxed;
        (self.files.load(Relaxed), self.chars.load(Relaxed), self.stages.load(Relaxed))
    }
    pub(super) fn add(&self, key: &std::sync::atomic::AtomicUsize, n: usize) {
        use std::sync::atomic::Ordering::Relaxed;
        key.fetch_add(n, Relaxed);
    }
}

/// Resident memory of the process, for the headless startup report.
pub fn rss_mb() -> f64 {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Counters {
            cb: u32,
            fault: u32,
            peak_ws: usize,
            ws: usize,
            _rest: [usize; 6],
        }
        extern "system" {
            fn GetCurrentProcess() -> isize;
            fn K32GetProcessMemoryInfo(p: isize, c: *mut Counters, cb: u32) -> i32;
        }
        let mut c = Counters { cb: std::mem::size_of::<Counters>() as u32, fault: 0, peak_ws: 0, ws: 0, _rest: [0; 6] };
        let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
        if ok == 0 { 0.0 } else { c.ws as f64 / (1024.0 * 1024.0) }
    }
    #[cfg(not(windows))]
    {
        0.0
    }
}