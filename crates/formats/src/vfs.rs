//! Virtual file system over a folder, a zip archive or in-memory bytes.
//!
//! Entries are indexed once and their bytes are read on demand: opening a
//! folder with hundreds of fighters costs a directory walk, not gigabytes of
//! RAM. Zip members are inflated when first read, never at index time.

use crate::Reader;
use std::collections::BTreeMap;
use std::sync::Arc;

mod archive;
mod find;
mod zip;

use archive::{File, Source};
pub use find::{char_key, content_hash, find_char_defs, find_stage_defs, resolve, resolve_key, stage_key};
pub use zip::{add_zip, add_zip_with_progress, ZipProgress};

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
                    if let Ok(k) = zip::add_zip_file(vfs, &path, rel) {
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

/// A file payload borrowed from an in-memory entry or shared from a lazy read.
pub enum VfsFile<'a> {
    Borrowed(&'a [u8]),
    Shared(Arc<Vec<u8>>),
}

impl VfsFile<'_> {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            VfsFile::Borrowed(b) => b,
            VfsFile::Shared(a) => a.as_slice(),
        }
    }
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}

impl std::ops::Deref for VfsFile<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsRef<[u8]> for VfsFile<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Clone for VfsFile<'_> {
    fn clone(&self) -> Self {
        match self {
            VfsFile::Borrowed(b) => VfsFile::Borrowed(b),
            VfsFile::Shared(a) => VfsFile::Shared(a.clone()),
        }
    }
}

impl std::fmt::Debug for VfsFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl PartialEq<[u8]> for VfsFile<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for VfsFile<'_> {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.as_slice() == *other
    }
}

impl PartialEq<&[u8]> for VfsFile<'_> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

/// Keys: normalized (lowercase, `/`-separated, relative) paths.
pub struct Vfs {
    files: BTreeMap<String, File>,
}

impl Default for Vfs {
    fn default() -> Self {
        Vfs::new()
    }
}

impl Vfs {
    pub fn new() -> Self {
        Vfs { files: BTreeMap::new() }
    }
    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.files.contains_key(key)
    }
    pub fn insert(&mut self, key: String, data: Vec<u8>) {
        self.files.insert(key, File::new(Source::Bytes(Arc::new(data))));
    }
    pub fn insert_shared(&mut self, key: String, data: Arc<Vec<u8>>) {
        self.files.insert(key, File::new(Source::Bytes(data)));
    }
    pub(crate) fn put(&mut self, key: String, file: File) {
        self.files.insert(key, file);
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn insert_disk(&mut self, key: String, path: std::path::PathBuf) {
        self.files.insert(key, File::new(Source::Disk(Arc::new(path))));
    }
    pub fn get(&self, key: &str) -> Option<VfsFile<'_>> {
        let file = self.files.get(key)?;
        match &file.source {
            Source::Bytes(data) => Some(VfsFile::Borrowed(data.as_slice())),
            _ => file.bytes().map(VfsFile::Shared),
        }
    }
    /// Content identity of an indexed file: length plus head/tail samples.
    pub fn signature(&self, key: &str) -> Option<(u64, u64)> {
        self.files.get(key)?.signature()
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.files.keys()
    }
    /// Keys from `start` onwards, in order: lets the folder fallback skip
    /// unrelated characters without scanning the whole map.
    pub fn keys_from<'a>(&'a self, start: &str) -> impl Iterator<Item = &'a String> {
        self.files.range(start.to_string()..).map(|(k, _)| k)
    }
    pub fn values(&self) -> impl Iterator<Item = VfsFile<'_>> {
        self.files.iter().filter_map(|(_, f)| f.bytes().map(VfsFile::Shared))
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, VfsFile<'_>)> {
        self.files.iter().filter_map(|(k, f)| f.bytes().map(|b| (k, VfsFile::Shared(b))))
    }
    pub fn append(&mut self, other: &mut Vfs) {
        self.files.append(&mut other.files);
    }
}

impl FromIterator<(String, Vec<u8>)> for Vfs {
    fn from_iter<T: IntoIterator<Item = (String, Vec<u8>)>>(iter: T) -> Self {
        let mut vfs = Vfs::new();
        for (k, d) in iter {
            vfs.insert(k, d);
        }
        vfs
    }
}

impl<const N: usize> From<[(String, Vec<u8>); N]> for Vfs {
    fn from(items: [(String, Vec<u8>); N]) -> Self {
        items.into_iter().collect()
    }
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
            if b == b'/' && last_slash {
                continue;
            }
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

pub(crate) fn join(prefix: &str, name: &str) -> String {
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
        return String::from_utf8(b.to_vec()).unwrap_or_default();
    }
    b.iter().map(|&c| c as char).collect()
}