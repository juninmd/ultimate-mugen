use super::*;

/// Heavy half of an import (hash + decompression), safe to run on a worker thread.
#[allow(dead_code)]
pub fn read_zip(bytes: &[u8]) -> Result<(String, Vfs), String> {
    read_zip_with_progress(bytes, None)
}

pub fn read_zip_with_progress(
    bytes: &[u8],
    progress: Option<&ume_formats::vfs::ZipProgress>,
) -> Result<(String, Vfs), String> {
    let pack = ume_formats::vfs::content_hash(bytes);
    let mut vfs = Vfs::new();
    let clock = ume_shell::Clock::now();
    ume_formats::vfs::add_zip_with_progress(&mut vfs, bytes, "", progress)?;
    if ume_formats::perf::enabled() {
        let mut by_ext: HashMap<String, (usize, usize)> = HashMap::new();
        for (k, v) in vfs.iter() {
            let ext = k
                .rsplit_once('.')
                .map(|(_, e)| e.to_string())
                .unwrap_or_else(|| "(none)".into());
            let e = by_ext.entry(ext).or_default();
            e.0 += 1;
            e.1 += v.len();
        }
        let mut rows: Vec<_> = by_ext.into_iter().collect();
        rows.sort_by_key(|(_, (_, b))| std::cmp::Reverse(*b));
        let total: usize = rows.iter().map(|(_, (_, b))| *b).sum();
        let listing: Vec<String> = rows
            .iter()
            .take(12)
            .map(|(e, (n, b))| format!("{e}:{n}/{:.1}MB", *b as f64 / 1048576.0))
            .collect();
        ume_formats::perf::emit(&format!(
            "[perf] zip {:.1}MB -> vfs {:.1}MB in {:.0} ms ({})",
            bytes.len() as f64 / 1048576.0,
            total as f64 / 1048576.0,
            clock.elapsed().as_secs_f64() * 1000.0,
            listing.join(" ")
        ));
    }
    Ok((pack, vfs))
}
