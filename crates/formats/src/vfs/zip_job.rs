use super::archive::ZipArchive;
use super::zip::{read_zip_member, zip_entries, ZipEntry};
use super::{join, Vfs};
use std::sync::Arc;

/// Incremental in-memory zip unpacking: each `step` inflates a bounded amount of data, so a
/// single-threaded host (the browser) can paint a real progress bar between slices.
pub struct ZipJob {
    source: ZipArchive,
    entries: Vec<ZipEntry>,
    next: usize,
    vfs: Vfs,
    done_bytes: u64,
    total_bytes: u64,
}

impl ZipJob {
    pub fn new(bytes: Vec<u8>) -> Result<ZipJob, String> {
        let archive = Arc::new(bytes);
        let entries = zip_entries(
            |at, len| {
                let start = usize::try_from(at).ok()?;
                Some(archive.get(start..start.checked_add(len)?)?.to_vec())
            },
            archive.len() as u64,
        )?;
        let total_bytes = entries.iter().map(|e| e.usize).sum();
        Ok(ZipJob { source: ZipArchive::Bytes(archive), entries, next: 0, vfs: Vfs::new(), done_bytes: 0, total_bytes })
    }

    /// Unpack entries until about `budget` uncompressed bytes were produced. Returns true when
    /// every entry has been read.
    pub fn step(&mut self, budget: u64) -> Result<bool, String> {
        let mut spent = 0;
        while let Some(e) = self.entries.get(self.next) {
            if spent >= budget {
                return Ok(false);
            }
            let data = read_zip_member(&self.source, e.method, e.csize, e.usize, e.local)
                .ok_or_else(|| format!("zip: {}: could not read entry", e.name))?;
            spent += e.usize.max(data.len() as u64);
            self.done_bytes += e.usize;
            self.vfs.insert(join("", &e.name), data);
            self.next += 1;
        }
        Ok(true)
    }

    pub fn entries_done(&self) -> usize {
        self.next
    }

    pub fn entries_total(&self) -> usize {
        self.entries.len()
    }

    pub fn current(&self) -> &str {
        self.entries.get(self.next.saturating_sub(1)).map_or("", |e| e.name.as_str())
    }

    pub fn fraction(&self) -> f32 {
        if self.total_bytes == 0 {
            return if self.next >= self.entries.len() { 1.0 } else { 0.0 };
        }
        (self.done_bytes as f64 / self.total_bytes as f64) as f32
    }

    pub fn into_vfs(self) -> Vfs {
        self.vfs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepwise_unpack_matches_eager_unpack() {
        let zip = include_bytes!("../../../../assets/mugen/elecbyte.zip");
        let mut eager = Vfs::new();
        super::super::add_zip(&mut eager, zip, "").unwrap();
        let mut job = ZipJob::new(zip.to_vec()).unwrap();
        let mut steps = 0;
        while !job.step(64 << 10).unwrap() {
            steps += 1;
            assert!(job.fraction() <= 1.0);
        }
        assert!(steps > 1);
        assert_eq!(job.entries_done(), job.entries_total());
        let sliced = job.into_vfs();
        assert_eq!(sliced.iter().count(), eager.iter().count());
    }

    #[test]
    fn garbage_is_an_error_not_a_hang() {
        assert!(ZipJob::new(vec![1, 2, 3, 4, 5]).is_err());
    }
}
