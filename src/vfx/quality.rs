use std::sync::atomic::{AtomicU8, Ordering};

static LEVEL: AtomicU8 = AtomicU8::new(2);

pub fn set(level: usize) -> u8 {
    let level = level.min(2) as u8;
    LEVEL.store(level, Ordering::Relaxed);
    level
}

pub fn level() -> u8 {
    LEVEL.load(Ordering::Relaxed)
}

pub fn scale() -> f32 {
    match level() {
        0 => 0.35,
        1 => 0.7,
        _ => 1.0,
    }
}

pub fn count(n: usize) -> usize {
    ((n as f32 * scale()).ceil() as usize).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_shrink_with_lower_levels() {
        set(0);
        let low = count(20);
        set(2);
        assert!(low < count(20));
    }
}
