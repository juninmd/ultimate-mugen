use crate::safety::{safe_rel_path, MAX_ITEM_BYTES, MAX_ITEM_FILES};

const MAGIC: &[u8; 6] = b"UMKT1\n";

pub type Files = Vec<(String, Vec<u8>)>;

pub fn encode(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let total: usize = files.iter().map(|(p, d)| p.len() + d.len() + 12).sum();
    let mut out = Vec::with_capacity(MAGIC.len() + 4 + total);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (path, data) in files {
        out.extend_from_slice(&(path.len() as u32).to_le_bytes());
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if n > self.0.len() {
            return Err("bundle is truncated".into());
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| "bundle is truncated")?))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_| "bundle is truncated")?))
    }
}

pub fn decode(bytes: &[u8]) -> Result<Files, String> {
    let mut c = Cursor(bytes);
    if c.take(MAGIC.len())? != MAGIC {
        return Err("not a market bundle".into());
    }
    let count = c.u32()? as usize;
    if count > MAX_ITEM_FILES * 2 {
        return Err("bundle has too many files".into());
    }
    let mut out = Vec::with_capacity(count.min(1024));
    let mut total = 0u64;
    for _ in 0..count {
        let plen = c.u32()? as usize;
        let path = std::str::from_utf8(c.take(plen)?).map_err(|_| "bundle path is not utf-8")?;
        let path = safe_rel_path(path).ok_or("bundle contains an unsafe path")?;
        let len = c.u64()?;
        total = total.saturating_add(len);
        if total > MAX_ITEM_BYTES * 2 || len > c.0.len() as u64 {
            return Err("bundle is truncated or too large".into());
        }
        out.push((path, c.take(len as usize)?.to_vec()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Files {
        vec![("chars/a/a.def".into(), b"[Info]".to_vec()), ("music/x.ogg".into(), vec![0, 1, 2, 255]), ("empty.txt".into(), vec![])]
    }

    #[test]
    fn roundtrips_files() {
        assert_eq!(decode(&encode(&sample())).unwrap(), sample());
        assert_eq!(decode(&encode(&[])).unwrap(), Vec::new());
    }

    #[test]
    fn rejects_corrupt_or_hostile_bundles() {
        let good = encode(&sample());
        assert!(decode(b"").is_err());
        assert!(decode(b"nope").is_err());
        for cut in [7, 12, good.len() - 1] {
            assert!(decode(&good[..cut]).is_err(), "cut {cut}");
        }
        let traversal = encode(&[("../evil".into(), vec![1])]);
        assert!(decode(&traversal).is_err());
        let mut huge = encode(&[("a".into(), vec![1])]);
        let at = huge.len() - 1 - 8;
        huge[at..at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(decode(&huge).is_err());
        let mut many = MAGIC.to_vec();
        many.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&many).is_err());
    }
}
