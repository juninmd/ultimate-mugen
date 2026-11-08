pub fn workers(jobs: usize) -> usize {
    if cfg!(target_arch = "wasm32") {
        return 1;
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(jobs)
        .clamp(1, 32)
}

pub fn map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    if items.is_empty() {
        return Vec::new();
    }
    let n = workers(items.len() / 2);
    if n <= 1 {
        return items.iter().map(f).collect();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let parts: Vec<Vec<R>> = std::thread::scope(|sc| {
            let handles: Vec<_> = (0..n)
                .map(|t| {
                    let start = t * items.len() / n;
                    let end = (t + 1) * items.len() / n;
                    let f_ref = &f;
                    sc.spawn(move || (start..end).map(|i| f_ref(&items[i])).collect::<Vec<R>>())
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
                .collect()
        });
        let mut out = Vec::with_capacity(items.len());
        for part in parts {
            out.extend(part);
        }
        out
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
