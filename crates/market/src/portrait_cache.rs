use crate::model::TreeEntry;
use std::collections::HashMap;

enum Slot<T> {
    Loading,
    Failed,
    Ready(T, u64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Lookup<T> {
    Ready(T),
    Pending,
    Start,
}

pub struct PortraitCache<T> {
    slots: HashMap<String, Slot<T>>,
    index: HashMap<String, HashMap<String, TreeEntry>>,
    inflight: usize,
    tick: u64,
    max_inflight: usize,
    max_ready: usize,
}

impl<T: Clone> PortraitCache<T> {
    pub fn new(max_inflight: usize, max_ready: usize) -> Self {
        PortraitCache { slots: HashMap::new(), index: HashMap::new(), inflight: 0, tick: 0, max_inflight, max_ready }
    }

    pub fn index(&self, repo_key: &str) -> Option<&HashMap<String, TreeEntry>> {
        self.index.get(repo_key)
    }

    pub fn set_index(&mut self, repo_key: &str, index: HashMap<String, TreeEntry>) {
        self.index.insert(repo_key.to_string(), index);
    }

    pub fn forget(&mut self, repo_key: &str) {
        self.index.remove(repo_key);
        self.slots.retain(|k, s| !(matches!(s, Slot::Failed) && k.starts_with(repo_key)));
    }

    pub fn lookup(&mut self, key: &str) -> Lookup<T> {
        self.tick += 1;
        let tick = self.tick;
        match self.slots.get_mut(key) {
            Some(Slot::Ready(art, seen)) => {
                *seen = tick;
                Lookup::Ready(art.clone())
            }
            Some(_) => Lookup::Pending,
            None if self.inflight >= self.max_inflight => Lookup::Pending,
            None => {
                self.inflight += 1;
                self.slots.insert(key.to_string(), Slot::Loading);
                Lookup::Start
            }
        }
    }

    pub fn finish(&mut self, key: &str, result: Result<T, String>) {
        self.inflight = self.inflight.saturating_sub(1);
        let slot = match result {
            Ok(art) => Slot::Ready(art, self.tick),
            Err(_) => Slot::Failed,
        };
        self.slots.insert(key.to_string(), slot);
        let ready = self.slots.values().filter(|s| matches!(s, Slot::Ready(..))).count();
        if ready > self.max_ready {
            let oldest = self.slots.iter().filter_map(|(k, s)| if let Slot::Ready(_, t) = s { Some((*t, k.clone())) } else { None }).min();
            if let Some((_, k)) = oldest {
                self.slots.remove(&k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_capped_and_failures_are_not_retried_until_forgotten() {
        let mut c: PortraitCache<u8> = PortraitCache::new(2, 10);
        assert_eq!(c.lookup("r\0a"), Lookup::Start);
        assert_eq!(c.lookup("r\0a"), Lookup::Pending);
        assert_eq!(c.lookup("r\0b"), Lookup::Start);
        assert_eq!(c.lookup("r\0c"), Lookup::Pending);
        c.finish("r\0a", Ok(1));
        c.finish("r\0b", Err("x".into()));
        assert_eq!(c.lookup("r\0a"), Lookup::Ready(1));
        assert_eq!(c.lookup("r\0b"), Lookup::Pending);
        assert_eq!(c.lookup("r\0c"), Lookup::Start);
        c.finish("r\0c", Err("x".into()));
        c.forget("r");
        assert_eq!(c.lookup("r\0b"), Lookup::Start);
        assert_eq!(c.lookup("r\0a"), Lookup::Ready(1));
    }

    #[test]
    fn least_recently_used_ready_entry_is_evicted() {
        let mut c: PortraitCache<u8> = PortraitCache::new(8, 2);
        for (i, k) in ["a", "b", "c"].into_iter().enumerate() {
            assert_eq!(c.lookup(k), Lookup::Start);
            c.finish(k, Ok(i as u8));
            if k == "b" {
                assert_eq!(c.lookup("a"), Lookup::Ready(0));
            }
        }
        assert_eq!(c.lookup("b"), Lookup::Start);
        assert_eq!(c.lookup("a"), Lookup::Ready(0));
    }
}
