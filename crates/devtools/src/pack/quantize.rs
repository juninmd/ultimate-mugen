use std::collections::{BTreeMap, HashMap};

type Key = (u8, [u8; 4]);

struct Leaf {
    depth: u8,
    prefix: [u8; 4],
    count: u64,
    sums: [u64; 4],
}

fn prefix_of(color: [u8; 4], depth: u8) -> [u8; 4] {
    let shift = 8 - depth as u32;
    [color[0].checked_shr(shift).unwrap_or(0), color[1].checked_shr(shift).unwrap_or(0), color[2].checked_shr(shift).unwrap_or(0), color[3].checked_shr(shift).unwrap_or(0)]
}

fn reduce_level(leaves: &mut Vec<Leaf>, depth: u8, max_colors: usize) {
    let mut groups: BTreeMap<[u8; 4], Vec<usize>> = BTreeMap::new();
    for (index, leaf) in leaves.iter().enumerate().filter(|(_, l)| l.depth == depth) {
        groups.entry(prefix_of(unscaled(leaf), depth - 1)).or_default().push(index);
    }
    let mut order: Vec<(u64, [u8; 4])> = groups.iter().filter(|(_, m)| m.len() > 1).map(|(k, m)| (m.iter().map(|&i| leaves[i].count).sum(), *k)).collect();
    order.sort();
    let mut total = leaves.len();
    let mut removed = vec![false; leaves.len()];
    let mut added = Vec::new();
    for (count, parent) in order {
        if total <= max_colors {
            break;
        }
        let members = &groups[&parent];
        let mut sums = [0u64; 4];
        for &i in members {
            removed[i] = true;
            (0..4).for_each(|c| sums[c] += leaves[i].sums[c]);
        }
        total -= members.len() - 1;
        added.push(Leaf { depth: depth - 1, prefix: parent, count, sums });
    }
    let mut kept: Vec<Leaf> = std::mem::take(leaves).into_iter().enumerate().filter(|(i, _)| !removed[*i]).map(|(_, l)| l).collect();
    kept.extend(added);
    if kept.len() > max_colors {
        for leaf in kept.iter_mut().filter(|l| l.depth == depth) {
            leaf.prefix = prefix_of(unscaled(leaf), depth - 1);
            leaf.depth = depth - 1;
        }
    }
    *leaves = kept;
}

fn unscaled(leaf: &Leaf) -> [u8; 4] {
    let shift = 8 - leaf.depth as u32;
    [leaf.prefix[0] << shift, leaf.prefix[1] << shift, leaf.prefix[2] << shift, leaf.prefix[3] << shift]
}

fn average(leaf: &Leaf) -> [u8; 4] {
    let n = leaf.count.max(1);
    [(leaf.sums[0] / n) as u8, (leaf.sums[1] / n) as u8, (leaf.sums[2] / n) as u8, (leaf.sums[3] / n) as u8]
}

pub fn quantize(rgba: &[u8], max_colors: usize) -> (Vec<[u8; 4]>, Vec<u8>) {
    let normalized = |p: &[u8]| if p[3] == 0 { [0, 0, 0, 0] } else { [p[0], p[1], p[2], p[3]] };
    let mut counts: BTreeMap<[u8; 4], u64> = BTreeMap::new();
    for p in rgba.chunks_exact(4) {
        *counts.entry(normalized(p)).or_default() += 1;
    }
    let mut leaves: Vec<Leaf> = counts.iter().map(|(c, n)| Leaf { depth: 8, prefix: *c, count: *n, sums: [c[0] as u64 * n, c[1] as u64 * n, c[2] as u64 * n, c[3] as u64 * n] }).collect();
    let mut depth = 8;
    while leaves.len() > max_colors && depth > 1 {
        reduce_level(&mut leaves, depth, max_colors);
        depth -= 1;
    }
    let palette: Vec<[u8; 4]> = leaves.iter().map(average).collect();
    let lookup: HashMap<Key, usize> = leaves.iter().enumerate().map(|(i, l)| ((l.depth, l.prefix), i)).collect();
    let owner = |color: [u8; 4]| (0..=8u8).rev().find_map(|d| lookup.get(&(d, prefix_of(color, d))).copied()).unwrap_or(0);
    let mut cache: HashMap<[u8; 4], u8> = HashMap::new();
    let indices = rgba.chunks_exact(4).map(|p| {
        let color = normalized(p);
        *cache.entry(color).or_insert_with(|| owner(color) as u8)
    }).collect();
    (palette, indices)
}
