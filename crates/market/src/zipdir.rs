use crate::model::TreeEntry;
use crate::safety::safe_rel_path;

pub const TAIL_BYTES: u64 = 64 << 10;
const MAX_DIRECTORY_BYTES: u64 = 4 << 20;
const EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const ENTRY_SIG: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];

#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    Done(Vec<TreeEntry>),
    Fetch { start: u64, end: u64, count: usize },
}

pub fn tail_range(size: u64) -> Option<(u64, u64)> {
    (size >= 22).then(|| (size - size.min(TAIL_BYTES), size - 1))
}

fn u16_at(b: &[u8], at: usize) -> usize {
    u16::from_le_bytes([b[at], b[at + 1]]) as usize
}

fn u32_at(b: &[u8], at: usize) -> u64 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]) as u64
}

pub fn from_tail(tail: &[u8], tail_start: u64) -> Result<Step, String> {
    let at = (0..tail.len().saturating_sub(21)).rev().find(|&i| tail[i..i + 4] == EOCD_SIG).ok_or("zip end record not found")?;
    let rec = &tail[at..];
    if rec.len() < 22 {
        return Err("truncated zip end record".into());
    }
    let (count, size, offset) = (u16_at(rec, 10), u32_at(rec, 12), u32_at(rec, 16));
    if count == 0xFFFF || size == 0xFFFF_FFFF || offset == 0xFFFF_FFFF {
        return Err("zip64 archives are not inspected".into());
    }
    if size > MAX_DIRECTORY_BYTES {
        return Err("zip directory too large".into());
    }
    if offset >= tail_start && offset + size <= tail_start + tail.len() as u64 {
        let from = (offset - tail_start) as usize;
        return from_directory(&tail[from..from + size as usize], count).map(Step::Done);
    }
    Ok(Step::Fetch { start: offset, end: offset + size.max(1) - 1, count })
}

pub fn from_directory(bytes: &[u8], count: usize) -> Result<Vec<TreeEntry>, String> {
    let mut out = Vec::with_capacity(count.min(4096));
    let mut at = 0usize;
    for _ in 0..count {
        if at + 46 > bytes.len() || bytes[at..at + 4] != ENTRY_SIG {
            return Err("corrupt zip directory".into());
        }
        let (size, name_len, extra, comment) = (u32_at(bytes, at + 24), u16_at(bytes, at + 28), u16_at(bytes, at + 30), u16_at(bytes, at + 32));
        let name_end = at + 46 + name_len;
        if name_end > bytes.len() {
            return Err("corrupt zip directory".into());
        }
        let name = String::from_utf8_lossy(&bytes[at + 46..name_end]).replace('\\', "/");
        if !name.ends_with('/') {
            if let Some(path) = safe_rel_path(&name) {
                out.push(TreeEntry { path, size, sha: String::new() });
            }
        }
        at = name_end + extra + comment;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zip(names: &[(&str, u32)], pad: usize, gap: usize) -> Vec<u8> {
        let mut out = vec![7u8; pad];
        let dir_start = out.len();
        for (name, size) in names {
            out.extend_from_slice(&ENTRY_SIG);
            let mut head = [0u8; 42];
            head[20..24].copy_from_slice(&size.to_le_bytes());
            head[24..26].copy_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&head);
            out.extend_from_slice(name.as_bytes());
        }
        let dir_len = out.len() - dir_start;
        out.extend(vec![9u8; gap]);
        out.extend_from_slice(&EOCD_SIG);
        let mut rec = [0u8; 18];
        rec[6..8].copy_from_slice(&(names.len() as u16).to_le_bytes());
        rec[8..12].copy_from_slice(&(dir_len as u32).to_le_bytes());
        rec[12..16].copy_from_slice(&(dir_start as u32).to_le_bytes());
        out.extend_from_slice(&rec);
        out
    }

    #[test]
    fn directory_inside_the_tail_is_read_without_another_request() {
        let z = zip(&[("kfm/", 0), ("kfm/kfm.def", 10), ("kfm/kfm.sff", 99), ("../evil", 1)], 100, 0);
        let (start, end) = tail_range(z.len() as u64).unwrap();
        let step = from_tail(&z[start as usize..=end as usize], start).unwrap();
        let Step::Done(entries) = step else { panic!("{step:?}") };
        assert_eq!(entries.iter().map(|e| (e.path.as_str(), e.size)).collect::<Vec<_>>(), vec![("kfm/kfm.def", 10), ("kfm/kfm.sff", 99)]);
    }

    #[test]
    fn directory_before_the_tail_asks_for_its_own_range() {
        let z = zip(&[("a/a.def", 1), ("b/b.def", 2)], 10, 200_000);
        let size = z.len() as u64;
        let (start, end) = tail_range(size).unwrap();
        let Step::Fetch { start: a, end: b, count } = from_tail(&z[start as usize..=end as usize], start).unwrap() else { panic!("expected a fetch") };
        assert_eq!(count, 2);
        assert_eq!(from_directory(&z[a as usize..=b as usize], count).unwrap().len(), 2);
    }

    #[test]
    fn garbage_is_rejected_not_trusted() {
        assert!(from_tail(&[0u8; 100], 0).is_err());
        assert!(from_directory(&[1u8; 60], 1).is_err());
        assert_eq!(tail_range(10), None);
        assert_eq!(tail_range(100), Some((0, 99)));
    }
}
