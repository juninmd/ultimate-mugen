use super::zip::read_zip_member;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Cached payloads up to this size are kept for reuse. Bigger files (sprite
/// and sound banks, tens of MB each) are read once per use instead of being
/// pinned in memory until the whole roster has been visited.
pub(super) const CACHE_MAX_BYTES: usize = 4 << 20;

#[derive(Clone)]
pub(crate) enum ZipArchive {
    Bytes(Arc<Vec<u8>>),
    #[cfg(not(target_arch = "wasm32"))]
    Disk(Arc<PathBuf>),
}

pub(crate) enum Source {
    /// Payload already in memory (imports, bundled packs, generated data).
    Bytes(Arc<Vec<u8>>),
    /// File on disk, read when requested.
    #[cfg(not(target_arch = "wasm32"))]
    Disk(Arc<PathBuf>),
    /// Member of a zip archive, inflated when requested.
    Zip {
        archive: ZipArchive,
        method: u16,
        csize: u64,
        usize: u64,
        local: u64,
    },
}

pub(crate) struct File {
    pub(crate) source: Source,
    cache: Mutex<Option<Arc<Vec<u8>>>>,
}

impl File {
    pub(super) fn new(source: Source) -> Self {
        File {
            source,
            cache: Mutex::new(None),
        }
    }

    pub(super) fn bytes(&self) -> Option<Arc<Vec<u8>>> {
        if let Source::Bytes(data) = &self.source {
            return Some(data.clone());
        }
        if let Some(hit) = self
            .cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return Some(hit.clone());
        }
        let data = match &self.source {
            Source::Bytes(_) => unreachable!(),
            #[cfg(not(target_arch = "wasm32"))]
            Source::Disk(path) => Arc::new(std::fs::read(&**path).ok()?),
            Source::Zip {
                archive,
                method,
                csize,
                usize,
                local,
            } => Arc::new(read_zip_member(archive, *method, *csize, *usize, *local)?),
        };
        if data.len() <= CACHE_MAX_BYTES {
            *self.cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(data.clone());
        }
        Some(data)
    }

    /// Cheap content identity: length plus head/tail samples, never a full read
    /// of a multi-megabyte bank. Used to deduplicate imported packs.
    pub(super) fn signature(&self) -> Option<(u64, u64)> {
        const SAMPLE: usize = 64 << 10;
        let h = |bytes: &[u8]| {
            bytes.iter().fold(super::find::FNV_OFFSET, |h, &b| {
                (h ^ b as u64).wrapping_mul(super::find::FNV_PRIME)
            })
        };
        match &self.source {
            Source::Bytes(data) => Some((data.len() as u64, h(data))),
            #[cfg(not(target_arch = "wasm32"))]
            Source::Disk(path) => {
                let len = std::fs::metadata(&**path).ok()?.len();
                let mut file = std::fs::File::open(&**path).ok()?;
                let head = read_at(&mut file, 0, SAMPLE.min(len as usize))?;
                let tail = read_at(
                    &mut file,
                    len.saturating_sub(SAMPLE as u64),
                    SAMPLE.min(len as usize),
                )?;
                Some((len, h(&head).rotate_left(1) ^ h(&tail)))
            }
            Source::Zip {
                archive,
                csize,
                usize: size,
                ..
            } => {
                let sample = match archive {
                    ZipArchive::Bytes(b) => h(&b[..b.len().min(SAMPLE)]),
                    #[cfg(not(target_arch = "wasm32"))]
                    ZipArchive::Disk(p) => {
                        let mut file = std::fs::File::open(&**p).ok()?;
                        let len = file.metadata().ok()?.len();
                        h(&read_at(&mut file, 0, SAMPLE.min(len as usize))?)
                    }
                };
                Some((*size, sample ^ csize.rotate_left(17) ^ size.rotate_left(33)))
            }
        }
    }
}

pub(super) fn read_at(file: &mut std::fs::File, offset: u64, len: usize) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut out = vec![0u8; len];
    let mut got = 0;
    while got < len {
        let n = file.read(&mut out[got..]).ok()?;
        if n == 0 {
            break;
        }
        got += n;
    }
    out.truncate(got);
    Some(out)
}
