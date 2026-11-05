pub fn workers(jobs: usize) -> usize {
    if cfg!(target_arch = "wasm32") { return 1; }
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(jobs).clamp(1, 32)
}

pub fn map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let n = workers(items.len() / 4);
    if n <= 1 { return items.iter().map(f).collect(); }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let next = AtomicUsize::new(0);
        let chunk = (items.len() / (n * 8)).max(1);
        let mut parts: Vec<Vec<(usize, R)>> = std::thread::scope(|sc| {
            let handles: Vec<_> = (0..n).map(|_| sc.spawn(|| {
                let mut out = Vec::new();
                loop {
                    let start = next.fetch_add(chunk, Ordering::Relaxed);
                    if start >= items.len() { break out; }
                    let end = (start + chunk).min(items.len());
                    out.extend((start..end).map(|i| (i, f(&items[i]))));
                }
            })).collect();
            handles.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
        });
        let mut slots: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
        for (i, r) in parts.iter_mut().flat_map(std::mem::take) { slots[i] = Some(r); }
        slots.into_iter().map(|r| r.expect("every index is produced exactly once")).collect()
    }
    #[cfg(target_arch = "wasm32")]
    unreachable!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn keeps_order_and_covers_every_item() {
        let v: Vec<u32> = (0..10_000).collect();
        let out = super::map(&v, |x| x * 2);
        assert_eq!(out, v.iter().map(|x| x * 2).collect::<Vec<_>>());
        assert!(super::map(&[] as &[u32], |x| *x).is_empty());
    }
}
