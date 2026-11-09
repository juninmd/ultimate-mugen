#[derive(Clone)]
pub struct Mask {
    pub width: usize,
    pub height: usize,
    pub bits: Vec<bool>,
}

impl Mask {
    pub fn new(width: usize, height: usize) -> Self {
        Mask { width, height, bits: vec![false; width * height] }
    }

    pub fn at(&self, x: usize, y: usize) -> bool {
        self.bits[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.bits[y * self.width + x] = value;
    }

    pub fn dilate(&self, iterations: usize) -> Mask {
        let mut current = self.clone();
        for _ in 0..iterations {
            let mut next = Mask::new(self.width, self.height);
            for y in 0..self.height {
                for x in 0..self.width {
                    if current.at(x, y) {
                        for ny in y.saturating_sub(1)..=(y + 1).min(self.height - 1) {
                            for nx in x.saturating_sub(1)..=(x + 1).min(self.width - 1) {
                                next.set(nx, ny, true);
                            }
                        }
                    }
                }
            }
            current = next;
        }
        current
    }
}

#[derive(Clone, Debug, Default)]
pub struct Region {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub row: usize,
    pub axis_x: i32,
    pub axis_y: i32,
    pub fixed_axis: bool,
}

impl Region {
    pub fn width(&self) -> usize {
        self.right - self.left
    }

    pub fn height(&self) -> usize {
        self.bottom - self.top
    }
}

fn find(parent: &mut [usize], mut node: usize) -> usize {
    while parent[node] != node {
        parent[node] = parent[parent[node]];
        node = parent[node];
    }
    node
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra != rb {
        parent[ra.max(rb)] = ra.min(rb);
    }
}

pub fn label_components(mask: &Mask) -> Vec<usize> {
    let mut parent = vec![0usize];
    let mut labels = vec![0usize; mask.width * mask.height];
    let mut previous: Vec<(usize, usize, usize)> = Vec::new();
    for y in 0..mask.height {
        let mut runs = Vec::new();
        let mut x = 0;
        while x < mask.width {
            if !mask.at(x, y) {
                x += 1;
                continue;
            }
            let start = x;
            while x < mask.width && mask.at(x, y) {
                x += 1;
            }
            let mut label = 0;
            for &(p_start, p_end, p_label) in &previous {
                if p_end >= start && p_start <= x {
                    if label == 0 {
                        label = p_label;
                    } else {
                        union(&mut parent, label, p_label);
                    }
                }
            }
            if label == 0 {
                label = parent.len();
                parent.push(label);
            }
            for cx in start..x {
                labels[y * mask.width + cx] = label;
            }
            runs.push((start, x, label));
        }
        previous = runs;
    }
    for value in labels.iter_mut() {
        if *value != 0 {
            *value = find(&mut parent, *value);
        }
    }
    labels
}

pub fn order_rows(mut regions: Vec<Region>) -> Vec<Region> {
    regions.sort_by_key(|r| r.top + r.bottom);
    let mut rows: Vec<Vec<Region>> = Vec::new();
    for region in regions {
        let found = rows.iter_mut().find(|row| {
            let top = row.iter().map(|r| r.top).min().unwrap_or(0);
            let bottom = row.iter().map(|r| r.bottom).max().unwrap_or(0);
            let overlap = bottom.min(region.bottom) as i64 - top.max(region.top) as i64;
            overlap as f64 > 0.5 * ((bottom - top).min(region.height()) as f64)
        });
        match found {
            Some(row) => row.push(region),
            None => rows.push(vec![region]),
        }
    }
    rows.sort_by_key(|row| row.iter().map(|r| r.top).min().unwrap_or(0));
    let mut ordered = Vec::new();
    for (index, mut row) in rows.into_iter().enumerate() {
        row.sort_by_key(|r| r.left);
        for mut region in row {
            region.row = index;
            ordered.push(region);
        }
    }
    ordered
}

pub fn detect_auto(mask: &Mask, merge_distance: usize, min_area: usize) -> Vec<Region> {
    let grown = if merge_distance > 0 { mask.dilate(merge_distance) } else { mask.clone() };
    let labels = label_components(&grown);
    let mut found: std::collections::BTreeMap<usize, (Region, usize)> = std::collections::BTreeMap::new();
    for y in 0..mask.height {
        for x in 0..mask.width {
            let label = labels[y * mask.width + x];
            if label == 0 || !mask.at(x, y) {
                continue;
            }
            let entry = found.entry(label).or_insert((Region { left: x, top: y, right: x + 1, bottom: y + 1, ..Region::default() }, 0));
            entry.0.left = entry.0.left.min(x);
            entry.0.top = entry.0.top.min(y);
            entry.0.right = entry.0.right.max(x + 1);
            entry.0.bottom = entry.0.bottom.max(y + 1);
            entry.1 += 1;
        }
    }
    order_rows(found.into_values().filter(|(_, area)| *area >= min_area).map(|(r, _)| r).collect())
}

#[derive(Clone, Default)]
pub struct GridOptions {
    pub cell_width: Option<usize>,
    pub cell_height: Option<usize>,
    pub columns: Option<usize>,
    pub rows: Option<usize>,
}

pub fn detect_grid(mask: &Mask, grid: &GridOptions) -> Result<Vec<Region>, String> {
    let cell_width = match (grid.cell_width, grid.columns) {
        (Some(w), _) => w,
        (None, Some(c)) if c > 0 => mask.width / c,
        _ => return Err("set a cell size or a column count".into()),
    };
    let cell_height = match (grid.cell_height, grid.rows) {
        (Some(h), _) => h,
        (None, Some(r)) if r > 0 => mask.height / r,
        _ => return Err("set a cell size or a row count".into()),
    };
    let mut regions = Vec::new();
    for (row, y) in (0..).zip((0..).step_by(cell_height.max(1)).take_while(|y| y + cell_height <= mask.height)) {
        for x in (0..).step_by(cell_width.max(1)).take_while(|x| x + cell_width <= mask.width) {
            let mut bounds: Option<(usize, usize, usize, usize)> = None;
            for cy in y..y + cell_height {
                for cx in x..x + cell_width {
                    if mask.at(cx, cy) {
                        let b = bounds.get_or_insert((cx, cy, cx + 1, cy + 1));
                        *b = (b.0.min(cx), b.1.min(cy), b.2.max(cx + 1), b.3.max(cy + 1));
                    }
                }
            }
            if let Some((left, top, right, bottom)) = bounds {
                regions.push(Region {
                    left, top, right, bottom, row,
                    axis_x: (x + cell_width / 2) as i32 - left as i32,
                    axis_y: (y + cell_height) as i32 - top as i32,
                    fixed_axis: true,
                });
            }
        }
    }
    Ok(regions)
}
