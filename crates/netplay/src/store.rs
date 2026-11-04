use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use crate::replay::Replay;
use crate::replay::Header;

pub const EXTENSION: &str = "umr";
const MAX_NAME_CHARS: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sort {
    Newest,
    Oldest,
    Name,
}

impl Sort {
    pub fn next(self) -> Sort {
        match self {
            Sort::Newest => Sort::Oldest,
            Sort::Oldest => Sort::Name,
            Sort::Name => Sort::Newest,
        }
    }

}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub modified: u64,
    pub ticks: usize,
    pub header: Result<Header, String>,
}

pub fn sanitize_name(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else if c == ' ' { '-' } else { '\0' })
        .filter(|c| *c != '\0')
        .take(MAX_NAME_CHARS)
        .collect();
    cleaned.trim_matches('-').to_string()
}

pub fn default_name(header: &Header) -> String {
    let side = |team: &Vec<String>| team.first().cloned().unwrap_or_else(|| "x".into());
    let base = sanitize_name(&format!("{}-vs-{}", side(&header.teams[0]), side(&header.teams[1])));
    let (y, mo, d, h, mi, s) = civil(header.created);
    format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}-{base}")
}

pub fn unique_path(dir: &Path, name: &str, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let name = match sanitize_name(name) {
        n if n.is_empty() => "replay".to_string(),
        n => n,
    };
    let first = dir.join(format!("{name}.{EXTENSION}"));
    if !exists(&first) {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{name}-{n}.{EXTENSION}")))
        .find(|p| !exists(p))
        .unwrap_or(first)
}

pub fn civil(unix: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (unix / 86_400) as i64;
    let secs = unix % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day, (secs / 3_600) as u32, (secs % 3_600 / 60) as u32, (secs % 60) as u32)
}

pub fn format_time(unix: u64) -> String {
    let (y, mo, d, h, mi, _) = civil(unix);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}")
}

pub fn format_duration(ticks: usize) -> String {
    let secs = ticks / 60;
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub fn sort_entries(entries: &mut [Entry], by: Sort) {
    match by {
        Sort::Newest => entries.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| a.name.cmp(&b.name))),
        Sort::Oldest => entries.sort_by(|a, b| a.modified.cmp(&b.modified).then_with(|| a.name.cmp(&b.name))),
        Sort::Name => entries.sort_by_key(|e| e.name.to_lowercase()),
    }
}

pub fn now() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod disk {
    use super::*;

    pub fn list_in(dir: &Path, by: Sort) -> Vec<Entry> {
        let mut out: Vec<Entry> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == EXTENSION))
            .map(|e| {
                let path = e.path();
                let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                let modified = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
                match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| Replay::from_bytes(&b)) {
                    Ok(r) => Entry { path, name, modified, ticks: r.len(), header: r.header() },
                    Err(e) => Entry { path, name, modified, ticks: 0, header: Err(e) },
                }
            })
            .collect();
        sort_entries(&mut out, by);
        out
    }

    pub fn save_in(dir: &Path, name: &str, replay: &Replay) -> Result<PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let path = unique_path(dir, name, |p| p.exists());
        std::fs::write(&path, replay.to_bytes()).map_err(|e| e.to_string())?;
        Ok(path)
    }

    pub fn load(path: &Path) -> Result<Replay, String> {
        Replay::from_bytes(&std::fs::read(path).map_err(|e| e.to_string())?)
    }

    pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf, String> {
        let clean = sanitize_name(new_name);
        if clean.is_empty() {
            return Err("invalid name".into());
        }
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let target = dir.join(format!("{clean}.{EXTENSION}"));
        if target == path {
            return Ok(target);
        }
        if target.exists() {
            return Err("a replay with that name already exists".into());
        }
        std::fs::rename(path, &target).map_err(|e| e.to_string())?;
        Ok(target)
    }

    pub fn delete(path: &Path) -> Result<(), String> {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use disk::{delete, list_in, load, rename, save_in};

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, modified: u64) -> Entry {
        Entry { path: PathBuf::from(name), name: name.into(), modified, ticks: 0, header: Err(String::new()) }
    }

    #[test]
    fn names_are_reduced_to_safe_file_names() {
        assert_eq!(sanitize_name("  my fight! ../x "), "my-fight-x");
        assert_eq!(sanitize_name("a/b\\c:d"), "abcd");
        assert_eq!(sanitize_name("---"), "");
        assert_eq!(sanitize_name(&"x".repeat(100)).len(), MAX_NAME_CHARS);
    }

    #[test]
    fn colliding_names_get_a_numeric_suffix() {
        let taken = [PathBuf::from("d/a.umr"), PathBuf::from("d/a-2.umr")];
        let p = unique_path(Path::new("d"), "a", |p| taken.iter().any(|t| t == p));
        assert_eq!(p, PathBuf::from("d/a-3.umr"));
        assert_eq!(unique_path(Path::new("d"), "???", |_| false), PathBuf::from("d/replay.umr"));
    }

    #[test]
    fn unix_time_is_formatted_as_utc_date() {
        assert_eq!(format_time(0), "1970-01-01 00:00");
        assert_eq!(format_time(1_700_000_000), "2023-11-14 22:13");
        assert_eq!(format_time(951_782_400 + 86_399), "2000-02-29 23:59");
        assert_eq!(format_duration(3_900), "1:05");
    }

    #[test]
    fn sorting_by_date_and_by_name() {
        let mut v = vec![entry("b", 10), entry("A", 30), entry("c", 20)];
        sort_entries(&mut v, Sort::Newest);
        assert_eq!(v.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["A", "c", "b"]);
        sort_entries(&mut v, Sort::Oldest);
        assert_eq!(v.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["b", "c", "A"]);
        sort_entries(&mut v, Sort::Name);
        assert_eq!(v.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["A", "b", "c"]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn save_list_rename_delete_load_on_disk() {
        let dir = std::env::temp_dir().join(format!("ume-replays-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let header = Header { build: "t".into(), mode: "versus".into(), stage: "s".into(), teams: [vec!["a".into()], vec!["b".into()]], created: 5, ..Default::default() };
        let mut rec = Replay::with_header(2, &header);
        for n in 0..300u32 {
            rec.push(&[n / 50, 0]);
        }
        let first = save_in(&dir, "fight one", &rec).unwrap();
        let second = save_in(&dir, "fight one", &rec).unwrap();
        assert_ne!(first, second);
        let listed = list_in(&dir, Sort::Name);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].ticks, 300);
        assert_eq!(listed[0].header.as_ref().unwrap().mode, "versus");
        let renamed = rename(&first, "final round").unwrap();
        assert_eq!(renamed.file_name().unwrap(), "final-round.umr");
        assert!(rename(&renamed, "fight-one-2").is_err());
        assert_eq!(load(&renamed).unwrap(), rec);
        std::fs::write(dir.join("broken.umr"), b"junk").unwrap();
        assert!(list_in(&dir, Sort::Name).iter().any(|e| e.name == "broken" && e.header.is_err()));
        delete(&renamed).unwrap();
        assert!(!renamed.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
