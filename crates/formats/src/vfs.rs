//! Virtual file system over a folder, a zip archive or in-memory bytes.
//!
//! Entries are indexed once and their bytes are read on demand: opening a
//! folder with hundreds of fighters costs a directory walk, not gigabytes of
//! RAM. Zip members are inflated when first read, never at index time.

use crate::ini::unquote;
use crate::Reader;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Cached payloads up to this size are kept for reuse. Bigger files (sprite
/// and sound banks, tens of MB each) are read once per use instead of being
/// pinned in memory until the whole roster has been visited.
const CACHE_MAX_BYTES: usize = 4 << 20;

#[derive(Clone)]
enum ZipArchive {
    Bytes(Arc<Vec<u8>>),
    Disk(Arc<PathBuf>),
}

enum Source {
    /// Payload already in memory (imports, bundled packs, generated data).
    Bytes(Arc<Vec<u8>>),
    /// File on disk, read when requested.
    Disk(Arc<PathBuf>),
    /// Member of a zip archive, inflated when requested.
    Zip { archive: ZipArchive, method: u16, csize: u64, usize: u64, local: u64 },
}

struct File {
    source: Source,
    cache: Mutex<Option<Arc<Vec<u8>>>>,
}

impl File {
    fn new(source: Source) -> Self { File { source, cache: Mutex::new(None) } }

    fn bytes(&self) -> Option<Arc<Vec<u8>>> {
        if let Source::Bytes(data) = &self.source { return Some(data.clone()); }
        if let Some(hit) = self.cache.lock().unwrap_or_else(|e| e.into_inner()).as_ref() { return Some(hit.clone()); }
        let data = match &self.source {
            Source::Bytes(_) => unreachable!(),
            Source::Disk(path) => Arc::new(std::fs::read(&**path).ok()?),
            Source::Zip { archive, method, csize, usize, local } => Arc::new(read_zip_member(archive, *method, *csize, *usize, *local)?),
        };
        if data.len() <= CACHE_MAX_BYTES {
            *self.cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(data.clone());
        }
        Some(data)
    }

    /// Cheap content identity: length plus head/tail samples, never a full read
    /// of a multi-megabyte bank. Used to deduplicate imported packs.
    fn signature(&self) -> Option<(u64, u64)> {
        const SAMPLE: usize = 64 << 10;
        let h = |bytes: &[u8]| bytes.iter().fold(FNV_OFFSET, |h, &b| (h ^ b as u64).wrapping_mul(FNV_PRIME));
        match &self.source {
            Source::Bytes(data) => Some((data.len() as u64, h(data))),
            Source::Disk(path) => {
                let len = std::fs::metadata(&**path).ok()?.len();
                let mut file = std::fs::File::open(&**path).ok()?;
                let head = read_at(&mut file, 0, SAMPLE.min(len as usize))?;
                let tail = read_at(&mut file, len.saturating_sub(SAMPLE as u64), SAMPLE.min(len as usize))?;
                Some((len, h(&head).rotate_left(1) ^ h(&tail)))
            }
            Source::Zip { archive, csize, usize: size, .. } => {
                let sample = match archive {
                    ZipArchive::Bytes(b) => h(&b[..b.len().min(SAMPLE)]),
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

fn read_at(file: &mut std::fs::File, offset: u64, len: usize) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut out = vec![0u8; len];
    let mut got = 0;
    while got < len {
        let n = file.read(&mut out[got..]).ok()?;
        if n == 0 { break; }
        got += n;
    }
    out.truncate(got);
    Some(out)
}

/// A file payload borrowed from an in-memory entry or shared from a lazy read.
pub enum VfsFile<'a> {
    Borrowed(&'a [u8]),
    Shared(Arc<Vec<u8>>),
}

impl VfsFile<'_> {
    pub fn as_slice(&self) -> &[u8] {
        match self { VfsFile::Borrowed(b) => b, VfsFile::Shared(a) => a.as_slice() }
    }
    pub fn len(&self) -> usize { self.as_slice().len() }
    pub fn is_empty(&self) -> bool { self.as_slice().is_empty() }
    pub fn to_vec(&self) -> Vec<u8> { self.as_slice().to_vec() }
}

impl std::ops::Deref for VfsFile<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] { self.as_slice() }
}

impl AsRef<[u8]> for VfsFile<'_> {
    fn as_ref(&self) -> &[u8] { self.as_slice() }
}

impl Clone for VfsFile<'_> {
    fn clone(&self) -> Self { match self { VfsFile::Borrowed(b) => VfsFile::Borrowed(b), VfsFile::Shared(a) => VfsFile::Shared(a.clone()) } }
}

impl std::fmt::Debug for VfsFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.as_slice().fmt(f) }
}

impl PartialEq<[u8]> for VfsFile<'_> {
    fn eq(&self, other: &[u8]) -> bool { self.as_slice() == other }
}

impl<const N: usize> PartialEq<&[u8; N]> for VfsFile<'_> {
    fn eq(&self, other: &&[u8; N]) -> bool { self.as_slice() == *other }
}

impl PartialEq<&[u8]> for VfsFile<'_> {
    fn eq(&self, other: &&[u8]) -> bool { self.as_slice() == *other }
}

/// Keys: normalized (lowercase, `/`-separated, relative) paths.
pub struct Vfs { files: BTreeMap<String, File> }

impl Default for Vfs {
    fn default() -> Self { Vfs::new() }
}

impl Vfs {
    pub fn new() -> Self { Vfs { files: BTreeMap::new() } }
    pub fn len(&self) -> usize { self.files.len() }
    pub fn is_empty(&self) -> bool { self.files.is_empty() }
    pub fn contains_key(&self, key: &str) -> bool { self.files.contains_key(key) }
    pub fn insert(&mut self, key: String, data: Vec<u8>) { self.files.insert(key, File::new(Source::Bytes(Arc::new(data)))); }
    pub fn insert_shared(&mut self, key: String, data: Arc<Vec<u8>>) { self.files.insert(key, File::new(Source::Bytes(data))); }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn insert_disk(&mut self, key: String, path: PathBuf) { self.files.insert(key, File::new(Source::Disk(Arc::new(path)))); }
    pub fn get(&self, key: &str) -> Option<VfsFile<'_>> {
        let file = self.files.get(key)?;
        match &file.source {
            Source::Bytes(data) => Some(VfsFile::Borrowed(data)),
            _ => file.bytes().map(VfsFile::Shared),
        }
    }
    /// Content identity of an indexed file: length plus head/tail samples.
    pub fn signature(&self, key: &str) -> Option<(u64, u64)> { self.files.get(key)?.signature() }
    pub fn keys(&self) -> impl Iterator<Item = &String> { self.files.keys() }
    /// Keys from `start` onwards, in order: lets the folder fallback skip
    /// unrelated characters without scanning the whole map.
    pub fn keys_from<'a>(&'a self, start: &str) -> impl Iterator<Item = &'a String> { self.files.range(start.to_string()..).map(|(k, _)| k) }
    pub fn values(&self) -> impl Iterator<Item = VfsFile<'_>> { self.files.iter().filter_map(|(_, f)| f.bytes().map(VfsFile::Shared)) }
    pub fn iter(&self) -> impl Iterator<Item = (&String, VfsFile<'_>)> { self.files.iter().filter_map(|(k, f)| f.bytes().map(|b| (k, VfsFile::Shared(b)))) }
    pub fn append(&mut self, other: &mut Vfs) { self.files.append(&mut other.files); }
}

impl FromIterator<(String, Vec<u8>)> for Vfs {
    fn from_iter<T: IntoIterator<Item = (String, Vec<u8>)>>(iter: T) -> Self {
        let mut vfs = Vfs::new();
        for (k, d) in iter { vfs.insert(k, d); }
        vfs
    }
}

impl<const N: usize> From<[(String, Vec<u8>); N]> for Vfs {
    fn from(items: [(String, Vec<u8>); N]) -> Self { items.into_iter().collect() }
}

impl IntoIterator for Vfs {
    type Item = (String, Vec<u8>);
    type IntoIter = std::vec::IntoIter<(String, Vec<u8>)>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter().filter_map(|(k, f)| f.bytes().map(|b| (k, b.as_ref().clone()))).collect::<Vec<_>>().into_iter()
    }
}

/// `\` -> `/`, strip a leading `./`, collapse repeated `/`, lowercase.
pub fn norm_path(p: &str) -> String {
    if p.is_ascii() {
        let p = p.strip_prefix("./").or_else(|| p.strip_prefix(".\\")).unwrap_or(p);
        let mut out = String::with_capacity(p.len());
        let mut last_slash = false;
        for b in p.bytes() {
            let b = if b == b'\\' { b'/' } else { b.to_ascii_lowercase() };
            if b == b'/' && last_slash { continue; }
            last_slash = b == b'/';
            out.push(b as char);
        }
        return out;
    }
    let s = p.replace('\\', "/");
    let s = s.strip_prefix("./").unwrap_or(&s);
    let mut out = String::with_capacity(s.len());
    let mut last_slash = false;
    for c in s.chars() {
        if c == '/' {
            if last_slash {
                continue;
            }
            last_slash = true;
        } else {
            last_slash = false;
        }
        out.push(c);
    }
    out.to_lowercase()
}

fn join(prefix: &str, name: &str) -> String {
    let prefix = prefix.trim_end_matches(['/', '\\']);
    if prefix.is_empty() {
        norm_path(name)
    } else {
        norm_path(&format!("{prefix}/{name}"))
    }
}

/// Latin-1 decode (strips a UTF-8 BOM).
pub fn text(b: impl AsRef<[u8]>) -> String {
    let b = b.as_ref();
    let b = b.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(b);
    if b.is_ascii() {
        // ASCII is valid UTF-8
        return String::from_utf8(b.to_vec()).unwrap_or_default();
    }
    b.iter().map(|&c| c as char).collect()
}

// ---------------------------------------------------------------------------
// zip
// ---------------------------------------------------------------------------

const EOCD_SIG: u32 = 0x0605_4b50;
const EOCD64_LOC_SIG: u32 = 0x0706_4b50;
const EOCD64_SIG: u32 = 0x0606_4b50;
const CDIR_SIG: u32 = 0x0201_4b50;
const LOCAL_SIG: u32 = 0x0403_4b50;
const MAX_ENTRY_BYTES: usize = 256 << 20;

struct ZipEntry {
    name: String,
    method: u16,
    csize: u64,
    usize: u64,
    local: u64,
}

/// Import progress shared with the UI while an in-memory archive is unpacked.
#[derive(Clone, Default)]
pub struct ZipProgress {
    pub done: Arc<std::sync::atomic::AtomicUsize>,
    pub total: Arc<std::sync::atomic::AtomicUsize>,
    pub current_name: Arc<std::sync::Mutex<String>>,
}

/// Read every file of a zip archive into `vfs` under `prefix` (may be empty).
/// Directory entries are skipped. Supports stored and deflated entries (and
/// zip64). Returns the number of files added.
pub fn add_zip(vfs: &mut Vfs, bytes: &[u8], prefix: &str) -> Result<usize, String> {
    add_zip_with_progress(vfs, bytes, prefix, None)
}

/// Same as [`add_zip`], reporting progress entry by entry. Archives that arrive
/// as bytes (web imports, bundled packs) are unpacked eagerly so the caller can
/// show a progress bar and the bytes can be released afterwards.
pub fn add_zip_with_progress(vfs: &mut Vfs, bytes: &[u8], prefix: &str, progress: Option<&ZipProgress>) -> Result<usize, String> {
    use std::sync::atomic::Ordering::Relaxed;
    let archive = Arc::new(bytes.to_vec());
    let entries = zip_entries(|at, len| {
        let start = usize::try_from(at).ok()?;
        Some(archive.get(start..start.checked_add(len)?)?.to_vec())
    }, archive.len() as u64)?;
    if let Some(p) = progress { p.total.store(entries.len(), Relaxed); }
    let source = ZipArchive::Bytes(archive);
    let mut added = 0;
    for e in &entries {
        let data = read_zip_member(&source, e.method, e.csize, e.usize, e.local)
            .ok_or_else(|| format!("zip: {}: could not read entry", e.name))?;
        if let Some(p) = progress {
            *p.current_name.lock().unwrap_or_else(|e| e.into_inner()) = e.name.clone();
            p.done.fetch_add(1, Relaxed);
        }
        vfs.insert(join(prefix, &e.name), data);
        added += 1;
    }
    Ok(added)
}

/// Same as [`add_zip`] for an archive that is already owned (no copy).
pub fn add_zip_owned(vfs: &mut Vfs, bytes: Vec<u8>, prefix: &str) -> Result<usize, String> {
    let archive = Arc::new(bytes);
    let entries = zip_entries(|at, len| {
        let start = usize::try_from(at).ok()?;
        Some(archive.get(start..start.checked_add(len)?)?.to_vec())
    }, archive.len() as u64)?;
    Ok(index_entries(vfs, ZipArchive::Bytes(archive), &entries, prefix))
}

fn index_entries(vfs: &mut Vfs, archive: ZipArchive, entries: &[ZipEntry], prefix: &str) -> usize {
    let mut added = 0;
    for e in entries {
        let source = Source::Zip { archive: archive.clone(), method: e.method, csize: e.csize, usize: e.usize, local: e.local };
        vfs.files.insert(join(prefix, &e.name), File::new(source));
        added += 1;
    }
    added
}

/// Index a zip file on disk by reading only its end record and central
/// directory. Member bytes stay on disk until read.
#[cfg(not(target_arch = "wasm32"))]
pub fn add_zip_file(vfs: &mut Vfs, path: &Path, prefix: &str) -> std::io::Result<usize> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len < 22 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "zip: file too small")); }
    let tail_len = (22 + 0xffffu64).min(len);
    let mut tail = vec![0u8; tail_len as usize];
    file.seek(SeekFrom::Start(len - tail_len))?;
    file.read_exact(&mut tail)?;
    let base = len - tail_len;
    let entries = zip_entries(|at, n| {
        // the end record lives in the buffered tail; the central directory can
        // sit anywhere before it, so everything else is a positioned read
        if at >= base {
            let off = usize::try_from(at - base).ok()?;
            return tail.get(off..off.checked_add(n)?).map(|s| s.to_vec());
        }
        read_at(&mut file, at, n)
    }, len).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let archive = ZipArchive::Disk(Arc::new(path.to_path_buf()));
    Ok(index_entries(vfs, archive, &entries, prefix))
}

struct Probe<F>(F);

impl<F: FnMut(u64, usize) -> Option<Vec<u8>>> Probe<F> {
    fn u16(&mut self, at: u64) -> Option<u16> { (self.0)(at, 2).and_then(|b| Some(u16::from_le_bytes(b.get(..2)?.try_into().ok()?))) }
    fn u32(&mut self, at: u64) -> Option<u32> { (self.0)(at, 4).and_then(|b| Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?))) }
    fn u64(&mut self, at: u64) -> Option<u64> { (self.0)(at, 8).and_then(|b| Some(u64::from_le_bytes(b.get(..8)?.try_into().ok()?))) }
}

/// Parse the end-of-central-directory record and every central directory
/// entry, using `read(at, len)` for random access into the archive.
fn zip_entries(read: impl FnMut(u64, usize) -> Option<Vec<u8>>, size: u64) -> Result<Vec<ZipEntry>, String> {
    let mut probe = Probe(read);

    // locate the end of central directory record (may be followed by a comment)
    let min = size.saturating_sub(22 + 0xffff);
    let mut eocd = None;
    let mut p = size.checked_sub(22).ok_or("zip: file too small")?;
    loop {
        if probe.u32(p) == Some(EOCD_SIG) && p + 22 + probe.u16(p + 20).unwrap_or(0) as u64 <= size {
            eocd = Some(p);
            break;
        }
        if p == min { break; }
        p -= 1;
    }
    let eocd = eocd.ok_or("zip: end of central directory not found")?;
    let mut entries = probe.u16(eocd + 10).unwrap_or(0) as u64;
    let mut cd_ofs = probe.u32(eocd + 16).unwrap_or(0) as u64;
    let mut cd_size = probe.u32(eocd + 12).unwrap_or(0) as u64;
    if (entries == 0xffff || cd_ofs == 0xffff_ffff) && eocd >= 20 && probe.u32(eocd - 20) == Some(EOCD64_LOC_SIG) {
        let z64 = probe.u64(eocd - 20 + 8).unwrap_or(0);
        if probe.u32(z64) == Some(EOCD64_SIG) {
            entries = probe.u64(z64 + 32).unwrap_or(entries);
            cd_size = probe.u64(z64 + 40).unwrap_or(cd_size);
            cd_ofs = probe.u64(z64 + 48).unwrap_or(cd_ofs);
        }
    }
    let dir = (probe.0)(cd_ofs, usize::try_from(cd_size).map_err(|_| "zip: central directory too large")?).ok_or("zip: truncated central directory")?;
    let mut out = Vec::with_capacity(entries.min(1 << 20) as usize);
    let mut p = 0usize;
    for _ in 0..entries {
        let r = Reader(&dir);
        if r.u32(p) != CDIR_SIG {
            return Err("zip: corrupt central directory".into());
        }
        let flags = r.u16(p + 8);
        let method = r.u16(p + 10);
        let mut csize = r.u32(p + 20) as u64;
        let mut usize_ = r.u32(p + 24) as u64;
        let name_len = r.u16(p + 28) as usize;
        let extra_len = r.u16(p + 30) as usize;
        let comment_len = r.u16(p + 32) as usize;
        let mut local = r.u32(p + 42) as u64;
        let name_start = p + 46;
        let name_bytes = dir.get(name_start..name_start + name_len).ok_or("zip: truncated central directory")?;
        // zip64 extended information
        let extra = dir.get(name_start + name_len..name_start + name_len + extra_len).unwrap_or(&[]);
        let mut e = 0;
        while e + 4 <= extra.len() {
            let (id, sz) = (u16::from_le_bytes([extra[e], extra[e + 1]]), u16::from_le_bytes([extra[e + 2], extra[e + 3]]) as usize);
            if id == 1 {
                let er = Reader(&extra[e + 4..(e + 4 + sz).min(extra.len())]);
                let mut q = 0;
                if usize_ == 0xffff_ffff {
                    usize_ = er.u64(q);
                    q += 8;
                }
                if csize == 0xffff_ffff {
                    csize = er.u64(q);
                    q += 8;
                }
                if local == 0xffff_ffff {
                    local = er.u64(q);
                }
            }
            e += 4 + sz;
        }
        p = name_start + name_len + extra_len + comment_len;

        let name = if flags & 0x800 != 0 { String::from_utf8_lossy(name_bytes).into_owned() } else { text(name_bytes) };
        if name.ends_with('/') || name.ends_with('\\') {
            continue;
        }
        if flags & 1 != 0 {
            return Err(format!("zip: {name}: encrypted entries are not supported"));
        }
        if usize_ > MAX_ENTRY_BYTES as u64 {
            return Err(format!("zip: {name}: entry too large"));
        }
        if method == 0 && csize > MAX_ENTRY_BYTES as u64 {
            return Err(format!("zip: {name}: entry too large"));
        }
        if method != 0 && method != 8 {
            return Err(format!("zip: {name}: unsupported compression method {method}"));
        }
        out.push(ZipEntry { name, method, csize, usize: usize_, local });
    }
    Ok(out)
}

fn read_zip_member(archive: &ZipArchive, method: u16, csize: u64, usize_: u64, local: u64) -> Option<Vec<u8>> {
    let read = |at: u64, len: usize| -> Option<Vec<u8>> {
        match archive {
            ZipArchive::Bytes(b) => {
                let start = usize::try_from(at).ok()?;
                Some(b.get(start..start.checked_add(len)?)?.to_vec())
            }
            ZipArchive::Disk(path) => {
                let mut file = std::fs::File::open(&**path).ok()?;
                read_at(&mut file, at, len)
            }
        }
    };
    if csize > MAX_ENTRY_BYTES as u64 { return None; }
    let header = read(local, 30)?;
    let r = Reader(&header);
    if r.u32(0) != LOCAL_SIG { return None; }
    let data_start = local + 30 + r.u16(26) as u64 + r.u16(28) as u64;
    let data = read(data_start, usize::try_from(csize).ok()?)?;
    match method {
        0 => Some(data),
        8 => inflate_raw(&data, usize::try_from(usize_).unwrap_or(0)).ok(),
        _ => None,
    }
}

fn inflate_raw(data: &[u8], size_hint: usize) -> Result<Vec<u8>, String> {
    use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
    use miniz_oxide::inflate::TINFLStatus;
    // fast path: exact-size output buffer from the declared size
    if size_hint > 0 && size_hint <= MAX_ENTRY_BYTES && size_hint <= data.len().saturating_mul(64).max(1 << 20) {
        let mut out = vec![0u8; size_hint];
        let mut st = DecompressorOxide::new();
        let (status, _, written) =
            decompress(&mut st, data, &mut out, 0, inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF);
        if status == TINFLStatus::Done && written == size_hint {
            return Ok(out);
        }
    }
    miniz_oxide::inflate::decompress_to_vec_with_limit(data, MAX_ENTRY_BYTES).map_err(|e| format!("inflate: {:?}", e.status))
}

/// Recursively index every file under `dir` (keys relative to `dir`, under
/// `prefix`). `.zip` files are indexed in place (their entries are keyed
/// relative to the zip's folder); unreadable zips are indexed as plain files.
/// File contents are not read.
#[cfg(not(target_arch = "wasm32"))]
pub fn add_dir(vfs: &mut Vfs, dir: &std::path::Path, prefix: &str) -> std::io::Result<usize> {
    fn walk(vfs: &mut Vfs, dir: &std::path::Path, rel: &str, n: &mut usize) -> std::io::Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let fname = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            let child = if rel.is_empty() { fname.clone() } else { format!("{rel}/{fname}") };
            let Ok(meta) = std::fs::metadata(&path) else { continue };
            if meta.is_dir() {
                walk(vfs, &path, &child, n)?;
            } else if meta.is_file() {
                if fname.to_ascii_lowercase().ends_with(".zip") {
                    if let Ok(k) = add_zip_file(vfs, &path, rel) {
                        *n += k;
                        continue;
                    }
                }
                vfs.insert_disk(norm_path(&child), path);
                *n += 1;
            }
        }
        Ok(())
    }
    let mut n = 0;
    let rel = norm_path(prefix.trim_end_matches(['/', '\\']));
    walk(vfs, dir, &rel, &mut n)?;
    Ok(n)
}

// ---------------------------------------------------------------------------
// resolution / discovery
// ---------------------------------------------------------------------------

/// Resolve a file name referenced from a .def in `base_dir` to its VFS key.
/// Tries `base_dir/name`, `name`, `data/name`, then the basename anywhere under
/// `base_dir`, then the basename anywhere.
pub fn resolve_key(vfs: &Vfs, base_dir: &str, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let n = norm_path(&unquote(name));
    if n.is_empty() {
        return None;
    }
    let base_dir = norm_path(base_dir.trim_end_matches(['/', '\\']));
    let tries = [if base_dir.is_empty() { n.clone() } else { format!("{base_dir}/{n}") }, n.clone(), format!("data/{n}")];
    for t in &tries {
        if vfs.contains_key(t) { return Some(t.clone()); }
    }
    // fallback: basename anywhere in the folder, then anywhere
    let base = n.rsplit('/').next().unwrap_or(&n);
    let suffix = format!("/{base}");
    if !base_dir.is_empty() {
        let folder = format!("{base_dir}/");
        if let Some(k) = vfs.keys_from(&folder).take_while(|k| k.starts_with(&folder)).find(|k| k.ends_with(&suffix)) { return Some(k.clone()); }
    }
    vfs.keys().find(|k| k.ends_with(&suffix) || k.as_str() == base).cloned()
}

/// Same as [`resolve_key`], returning the bytes.
pub fn resolve<'a>(vfs: &'a Vfs, base_dir: &str, name: &str) -> Option<VfsFile<'a>> {
    let key = resolve_key(vfs, base_dir, name)?;
    vfs.get(&key)
}

/// `\b<word>\s*=` in lowercase text.
fn has_key(t: &str, word: &str) -> bool {
    let b = t.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut from = 0;
    while let Some(p) = t[from..].find(word) {
        let s = from + p;
        if s == 0 || !is_word(b[s - 1]) {
            let mut i = s + word.len();
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if b.get(i) == Some(&b'=') {
                return true;
            }
        }
        from = s + 1;
    }
    false
}

/// Character .def files (has `[Files]` with `cns =` and `sprite =`).
pub fn find_char_defs(vfs: &Vfs) -> Vec<String> {
    let candidates: Vec<&String> = vfs.keys().filter(|k| k.ends_with(".def")).collect();
    let hits = crate::par::map(&candidates, |key| {
        let Some(bytes) = vfs.get(key) else { return false };
        if bytes.len() > 1 << 20 { return false; }
        let t = text(&bytes).to_ascii_lowercase();
        t.contains("[files]") && has_key(&t, "cns") && has_key(&t, "sprite")
    });
    candidates.into_iter().zip(hits).filter(|(_, hit)| *hit).map(|(k, _)| k.clone()).collect()
}

/// Stage .def files (has `[BGdef` and `[StageInfo]` or `[Camera]`).
pub fn find_stage_defs(vfs: &Vfs) -> Vec<String> {
    let candidates: Vec<&String> = vfs.keys().filter(|k| k.ends_with(".def")).collect();
    let hits = crate::par::map(&candidates, |key| {
        let Some(bytes) = vfs.get(key) else { return false };
        if bytes.len() > 1 << 20 { return false; }
        let t = text(&bytes).to_ascii_lowercase();
        t.contains("[bgdef") && (t.contains("[stageinfo]") || t.contains("[camera]"))
    });
    candidates.into_iter().zip(hits).filter(|(_, hit)| *hit).map(|(k, _)| k.clone()).collect()
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub(crate) fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Stable 64-bit hash (hex) of arbitrary bytes, for deduplicating persisted or repeated imports.
pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:016x}", fnv(FNV_OFFSET, bytes))
}

fn canonical_def_text(def_bytes: &[u8]) -> String {
    text(def_bytes)
        .lines()
        .map(|l| l.split(';').next().unwrap_or("").trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn def_key(vfs: &Vfs, def: &str, section: &str, sprite_key: &str) -> String {
    let def = norm_path(def);
    let Some(def_bytes) = vfs.get(&def) else {
        return format!("path:{def}");
    };
    let secs = crate::ini::parse_ini(&text(&def_bytes));
    let sprite = crate::ini::find_section(&secs, section)
        .and_then(|s| s.get(sprite_key))
        .map(|v| unquote(v))
        .unwrap_or_default();
    let base = def.rfind('/').map(|i| &def[..i]).unwrap_or("");
    let mut h = fnv(FNV_OFFSET, canonical_def_text(&def_bytes).as_bytes());
    match resolve_key(vfs, base, &sprite).and_then(|k| vfs.signature(&k)) {
        Some((len, sample)) => {
            h = fnv(h, &len.to_le_bytes());
            h = fnv(h, &sample.to_le_bytes());
        }
        None => h = fnv(h, norm_path(&sprite).as_bytes()),
    }
    format!("{h:016x}")
}

/// Identity of a character independent of where it was found: hash of its normalized `.def` and sprite file.
/// The same fighter imported twice, from another path, or with different letter case yields the same key.
pub fn char_key(vfs: &Vfs, def: &str) -> String {
    def_key(vfs, def, "files", "sprite")
}

/// Identity of a stage, same idea as [`char_key`] (uses `[BGdef] spr`).
pub fn stage_key(vfs: &Vfs, def: &str) -> String {
    def_key(vfs, def, "bgdef", "spr")
}
