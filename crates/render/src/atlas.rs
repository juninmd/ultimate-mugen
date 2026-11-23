#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect { pub x: u32, pub y: u32, pub w: u32, pub h: u32 }

#[derive(Clone, Copy, Debug)]
struct Shelf { y: u32, h: u32, x: u32 }

#[derive(Clone, Debug, Default)]
struct Zone { shelves: Vec<Shelf>, used_h: u32 }

#[derive(Clone, Debug)]
pub struct Atlas { pub width: u32, pub height: u32, pad: u32, fixed: Zone, transient: Zone }

const TRANSIENT_SHARE: u32 = 70;

fn shelf_height(h: u32) -> u32 {
    let step = (h.next_power_of_two() / 8).max(4);
    h.div_ceil(step) * step
}

impl Atlas {
    pub fn new(width: u32, height: u32) -> Self { Atlas { width, height, pad: 1, fixed: Zone::default(), transient: Zone::default() } }

    pub fn alloc(&mut self, w: u32, h: u32) -> Option<Rect> { self.place(w, h, false) }

    /// Sprites that can be re-uploaded on demand live in a zone that grows from the bottom and clears on its own.
    pub fn alloc_transient(&mut self, w: u32, h: u32) -> Option<Rect> { self.place(w, h, true) }

    pub fn transient_used(&self) -> u32 { self.transient.used_h }

    fn place(&mut self, w: u32, h: u32, transient: bool) -> Option<Rect> {
        let (pw, ph) = (w + self.pad * 2, h + self.pad * 2);
        if pw > self.width || ph > self.height { return None; }
        let (other_used, limit) = if transient { (self.fixed.used_h, self.height * TRANSIENT_SHARE / 100) } else { (self.transient.used_h, self.height) };
        let zone = if transient { &mut self.transient } else { &mut self.fixed };
        let mut best: Option<usize> = None;
        for (i, s) in zone.shelves.iter().enumerate() {
            if s.h >= ph && self.width - s.x >= pw && best.is_none_or(|b| zone.shelves[b].h > s.h) { best = Some(i); }
        }
        let i = match best {
            Some(i) => i,
            None => {
                let room = (limit - zone.used_h.min(limit)).min(self.height - zone.used_h - other_used);
                let sh = shelf_height(ph).min(room);
                if sh < ph { return None; }
                let y = if transient { self.height - zone.used_h - sh } else { zone.used_h };
                zone.shelves.push(Shelf { y, h: sh, x: 0 });
                zone.used_h += sh;
                zone.shelves.len() - 1
            }
        };
        let s = &mut zone.shelves[i];
        let r = Rect { x: s.x + self.pad, y: s.y + self.pad, w, h };
        s.x += pw;
        Some(r)
    }

    pub fn clear(&mut self) {
        self.fixed.shelves.clear();
        self.fixed.used_h = 0;
        self.transient.shelves.clear();
        self.transient.used_h = 0;
    }

    pub fn clear_transient(&mut self) {
        self.transient.shelves.clear();
        self.transient.used_h = 0;
    }

    pub fn occupancy(&self) -> f32 {
        let used: u64 = self.fixed.shelves.iter().chain(&self.transient.shelves).map(|s| s.x as u64 * s.h as u64).sum();
        used as f32 / (self.width as u64 * self.height as u64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlaps(a: &Rect, b: &Rect) -> bool { a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h }

    #[test]
    fn packs_without_overlap_and_inside_bounds() {
        let mut a = Atlas::new(256, 256);
        let mut rs = vec![];
        let mut seed = 7u32;
        for _ in 0..200 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let (w, h) = (4 + (seed >> 8) % 28, 4 + (seed >> 16) % 28);
            if let Some(r) = a.alloc(w, h) { rs.push(r); }
        }
        assert!(rs.len() > 50);
        for (i, r) in rs.iter().enumerate() {
            assert!(r.x + r.w < 256 && r.y + r.h < 256);
            for q in &rs[i + 1..] { assert!(!overlaps(r, q)); }
        }
        assert!(a.occupancy() > 0.5);
    }

    #[test]
    fn varied_heights_share_shelves_instead_of_opening_one_each() {
        let mut a = Atlas::new(1024, 1024);
        let mut placed = 0;
        for i in 0..4000u32 {
            let (w, h) = (96 + (i * 37) % 64, 150 + (i * 53) % 100);
            if a.alloc(w, h).is_none() { break; }
            placed += 1;
        }
        assert!(placed > 20, "placed {placed}");
    }

    #[test]
    fn transient_zone_clears_without_touching_the_fixed_zone() {
        let mut a = Atlas::new(256, 256);
        let fixed = a.alloc(30, 30).unwrap();
        let mut moved = vec![];
        while let Some(r) = a.alloc_transient(40, 40) { moved.push(r); }
        assert!(moved.len() >= 4);
        assert!(moved.iter().all(|r| !overlaps(r, &fixed)));
        for (i, r) in moved.iter().enumerate() { for q in &moved[i + 1..] { assert!(!overlaps(r, q)); } }
        a.clear_transient();
        assert_eq!(a.transient_used(), 0);
        let again = a.alloc_transient(40, 40).unwrap();
        assert!(!overlaps(&again, &fixed));
        assert!(a.alloc(30, 30).is_some());
    }

    #[test]
    fn rejects_too_big_and_reuses_after_clear() {
        let mut a = Atlas::new(64, 64);
        assert!(a.alloc(64, 1).is_none());
        assert!(a.alloc(62, 62).is_some());
        assert!(a.alloc(8, 8).is_none());
        a.clear();
        assert_eq!(a.alloc(8, 8), Some(Rect { x: 1, y: 1, w: 8, h: 8 }));
    }
}
