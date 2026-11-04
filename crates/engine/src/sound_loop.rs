#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoopSpec {
    pub start: i32,
    pub end: i32,
    pub count: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoopChange {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub count: Option<i32>,
}

impl LoopChange {
    pub fn is_empty(&self) -> bool { self.start.is_none() && self.end.is_none() && self.count.is_none() }
}

impl LoopSpec {
    pub fn is_active(&self) -> bool { self.count != 0 }
    pub fn has_points(&self) -> bool { self.start > 0 || self.end > 0 || self.count > 0 }
    pub fn apply(self, change: LoopChange) -> LoopSpec {
        LoopSpec { start: change.start.unwrap_or(self.start), end: change.end.unwrap_or(self.end), count: change.count.unwrap_or(self.count) }
    }
    pub fn bounds(&self, total: usize) -> Option<(usize, usize)> {
        let end = if self.end <= 0 { total } else { (self.end as usize).min(total) };
        let start = self.start.max(0) as usize;
        (start < end).then_some((start, end))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopPlan {
    pub segments: Vec<(usize, usize)>,
    pub forever: Option<(usize, usize)>,
}

impl LoopPlan {
    pub fn new(total: usize, from: usize, spec: &LoopSpec) -> LoopPlan {
        let from = from.min(total);
        let plain = LoopPlan { segments: vec![(from, total)], forever: None };
        if !spec.is_active() { return plain; }
        let Some((start, end)) = spec.bounds(total) else { return plain };
        if from >= end { return plain; }
        let mut segments = vec![(from, end)];
        if spec.count < 0 { return LoopPlan { segments, forever: Some((start, end)) }; }
        for _ in 0..spec.count { segments.push((start, end)); }
        if end < total { segments.push((end, total)); }
        LoopPlan { segments, forever: None }
    }

    pub fn finite_len(&self) -> usize { self.segments.iter().map(|(a, b)| b - a).sum() }

    pub fn source_frame(&self, elapsed: usize) -> Option<usize> {
        let mut left = elapsed;
        for &(a, b) in &self.segments {
            if left < b - a { return Some(a + left); }
            left -= b - a;
        }
        let (start, end) = self.forever?;
        Some(start + left % (end - start))
    }

    pub fn unroll<T: Copy>(&self, frames: &[T]) -> (Vec<T>, Option<Vec<T>>) {
        let mut head = Vec::with_capacity(self.finite_len());
        for &(a, b) in &self.segments { head.extend_from_slice(&frames[a.min(frames.len())..b.min(frames.len())]); }
        let body = self.forever.map(|(a, b)| frames[a.min(frames.len())..b.min(frames.len())].to_vec());
        (head, body)
    }
}
