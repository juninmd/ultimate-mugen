#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Copy, Debug)]
struct Shelf {
    y: u32,
    h: u32,
    x: u32,
}

#[derive(Clone, Debug)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pad: u32,
    shelves: Vec<Shelf>,
    used_h: u32,
}

impl Atlas {
    pub fn new(width: u32, height: u32) -> Self {
        Atlas {
            width,
            height,
            pad: 1,
            shelves: vec![],
            used_h: 0,
        }
    }

    pub fn alloc(&mut self, w: u32, h: u32) -> Option<Rect> {
        let (pw, ph) = (w + self.pad * 2, h + self.pad * 2);
        if pw > self.width || ph > self.height {
            return None;
        }
        let mut best: Option<usize> = None;
        for (i, s) in self.shelves.iter().enumerate() {
            if s.h >= ph && self.width - s.x >= pw && best.is_none_or(|b| self.shelves[b].h > s.h) {
                best = Some(i);
            }
        }
        let i = match best {
            Some(i) => i,
            None => {
                if self.height - self.used_h < ph {
                    return None;
                }
                self.shelves.push(Shelf {
                    y: self.used_h,
                    h: ph,
                    x: 0,
                });
                self.used_h += ph;
                self.shelves.len() - 1
            }
        };
        let s = &mut self.shelves[i];
        let r = Rect {
            x: s.x + self.pad,
            y: s.y + self.pad,
            w,
            h,
        };
        s.x += pw;
        Some(r)
    }

    pub fn clear(&mut self) {
        self.shelves.clear();
        self.used_h = 0;
    }

    pub fn occupancy(&self) -> f32 {
        let used: u64 = self.shelves.iter().map(|s| s.x as u64 * s.h as u64).sum();
        used as f32 / (self.width as u64 * self.height as u64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlaps(a: &Rect, b: &Rect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    #[test]
    fn packs_without_overlap_and_inside_bounds() {
        let mut a = Atlas::new(256, 256);
        let mut rs = vec![];
        let mut seed = 7u32;
        for _ in 0..200 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let (w, h) = (4 + (seed >> 8) % 28, 4 + (seed >> 16) % 28);
            if let Some(r) = a.alloc(w, h) {
                rs.push(r);
            }
        }
        assert!(rs.len() > 50);
        for (i, r) in rs.iter().enumerate() {
            assert!(r.x + r.w < 256 && r.y + r.h < 256);
            for q in &rs[i + 1..] {
                assert!(!overlaps(r, q));
            }
        }
        assert!(a.occupancy() > 0.5);
    }

    #[test]
    fn rejects_too_big_and_reuses_after_clear() {
        let mut a = Atlas::new(64, 64);
        assert!(a.alloc(64, 1).is_none());
        assert!(a.alloc(62, 62).is_some());
        assert!(a.alloc(8, 8).is_none());
        a.clear();
        assert_eq!(
            a.alloc(8, 8),
            Some(Rect {
                x: 1,
                y: 1,
                w: 8,
                h: 8
            })
        );
    }
}
