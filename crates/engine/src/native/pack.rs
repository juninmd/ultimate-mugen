use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

const MAGIC: &[u8; 4] = b"UMEP";
const VERSION: u8 = 2;
const MAX_PACKS: usize = 64;

pub struct SpritePack {
    pub character: String,
    files: HashMap<String, Arc<Vec<u8>>>,
    revision: u64,
    partial: bool,
}

impl SpritePack {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 16 << 20 { return Err("sprite pack: larger than 16 MiB".into()); }
        if bytes.len() < 11 || &bytes[0..4] != MAGIC { return Err("sprite pack: bad header".into()); }
        if bytes[4] != VERSION { return Err(format!("sprite pack: unsupported version {}", bytes[4])); }
        let id_len = u16::from_le_bytes(bytes[5..7].try_into().unwrap()) as usize;
        let id_end = 7usize.checked_add(id_len).ok_or("sprite pack: truncated id")?;
        let character = std::str::from_utf8(bytes.get(7..id_end).ok_or("sprite pack: truncated id")?).map_err(|_| "sprite pack: id is not utf-8")?.to_string();
        if character.is_empty() { return Err("sprite pack: empty id".into()); }
        let count_at = id_end;
        let count = u32::from_le_bytes(bytes.get(count_at..count_at + 4).ok_or("sprite pack: truncated count")?.try_into().unwrap()) as usize;
        let mut at = count_at + 4;
        if count > (bytes.len() - at) / 6 { return Err(format!("sprite pack: {count} entries do not fit in {} bytes", bytes.len())); }
        let mut files = HashMap::with_capacity(count.min(64));
        for _ in 0..count {
            let name_len = u16::from_le_bytes(bytes.get(at..at + 2).ok_or("sprite pack: truncated name")?.try_into().unwrap()) as usize;
            at += 2;
            let name_end = at.checked_add(name_len).ok_or("sprite pack: truncated name")?;
            let name = std::str::from_utf8(bytes.get(at..name_end).ok_or("sprite pack: truncated name")?).map_err(|_| "sprite pack: name is not utf-8")?.to_string();
            at = name_end;
            let data_len = u32::from_le_bytes(bytes.get(at..at + 4).ok_or("sprite pack: truncated payload")?.try_into().unwrap()) as usize;
            at += 4;
            let data_end = at.checked_add(data_len).ok_or("sprite pack: truncated payload")?;
            let data = bytes.get(at..data_end).ok_or("sprite pack: truncated payload")?;
            at = data_end;
            if files.insert(name.clone(), Arc::new(data.to_vec())).is_some() { return Err(format!("sprite pack: duplicate entry {name}")); }
        }
        Ok(SpritePack { character, files, revision: 0, partial: false })
    }
}

pub(super) fn read_sprite(root: &Path, name: &str) -> Option<Arc<Vec<u8>>> {
    let packed = root.file_name().and_then(|character| character.to_str()).and_then(|character| read(character, name));
    let bytes = packed.or_else(|| std::fs::read(root.join(name)).ok().map(Arc::new))?;
    if let Some(seen) = TRACE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).as_mut() {
        seen.insert(name.to_string());
    }
    Some(bytes)
}

static TRACE: Mutex<Option<BTreeSet<String>>> = Mutex::new(None);

pub(super) fn trace_reads(work: impl FnOnce()) -> Vec<String> {
    *TRACE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(BTreeSet::new());
    work();
    TRACE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take().unwrap_or_default().into_iter().collect()
}

thread_local! {
    static PACKS: RefCell<Vec<(String, Arc<SpritePack>)>> = RefCell::new(Vec::new());
    static REVISION: Cell<u64> = const { Cell::new(0) };
}

pub fn insert(character: &str, bytes: &[u8]) -> Result<(), String> {
    store(character, bytes, false)
}

pub fn insert_menu(character: &str, bytes: &[u8]) -> Result<(), String> {
    if contains(character) { return Ok(()); }
    store(character, bytes, true)
}

pub fn complete(character: &str) -> bool {
    PACKS.with(|packs| packs.borrow().iter().any(|(name, pack)| name == character && !pack.partial))
}

fn store(character: &str, bytes: &[u8], partial: bool) -> Result<(), String> {
    let mut pack = SpritePack::parse(bytes)?;
    pack.partial = partial;
    if pack.character != character { return Err(format!("sprite pack: {} sent for {character}", pack.character)); }
    pack.revision = REVISION.with(|revision| {
        let next = revision.get().wrapping_add(1).max(1);
        revision.set(next);
        next
    });
    let pack = Arc::new(pack);
    PACKS.with(|packs| {
        let mut packs = packs.borrow_mut();
        packs.retain(|(name, _)| name != character);
        if packs.len() >= MAX_PACKS { packs.remove(0); }
        packs.push((character.to_string(), pack));
    });
    Ok(())
}

pub fn contains(character: &str) -> bool {
    PACKS.with(|packs| packs.borrow().iter().any(|(name, _)| name == character))
}

pub fn revision(character: &str) -> u64 {
    PACKS.with(|packs| packs.borrow().iter().find(|(name, _)| name == character).map_or(0, |(_, pack)| pack.revision))
}

pub fn read(character: &str, name: &str) -> Option<Arc<Vec<u8>>> {
    PACKS.with(|packs| packs.borrow().iter().find(|(id, _)| id == character).and_then(|(_, pack)| pack.files.get(name).cloned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(character: &str, entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.push(VERSION);
        out.extend_from_slice(&(character.len() as u16).to_le_bytes());
        out.extend_from_slice(character.as_bytes());
        out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (name, data) in entries {
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(data);
        }
        out
    }

    #[test]
    fn reads_every_file_back() {
        let bytes = pack("kaito", &[("01_idle.png", b"idle"), ("meta.json", b"{}")]);
        let parsed = SpritePack::parse(&bytes).unwrap();
        assert_eq!(parsed.character, "kaito");
        assert_eq!(parsed.files.get("01_idle.png").unwrap().as_slice(), b"idle");
        assert_eq!(parsed.files.get("meta.json").unwrap().as_slice(), b"{}");
    }

    #[test]
    fn rejects_foreign_or_truncated_payloads() {
        assert!(SpritePack::parse(b"nope").is_err());
        let mut broken = pack("kaito", &[("01_idle.png", b"idle")]);
        broken.truncate(broken.len() - 2);
        assert!(SpritePack::parse(&broken).is_err());
    }

    #[test]
    fn rejects_an_entry_count_that_cannot_fit() {
        let mut header = MAGIC.to_vec();
        header.push(VERSION);
        header.extend_from_slice(&5u16.to_le_bytes());
        header.extend_from_slice(b"kaito");
        header.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(SpritePack::parse(&header).is_err());
    }

    #[test]
    fn menu_pack_never_replaces_a_full_pack_and_is_upgraded_by_one() {
        let menu = pack("menu_tier", &[("portrait.png", b"face")]);
        let full = pack("menu_tier", &[("portrait.png", b"face"), ("01_idle.png", b"idle")]);
        insert_menu("menu_tier", &menu).unwrap();
        assert!(contains("menu_tier") && !complete("menu_tier"));
        assert!(read("menu_tier", "01_idle.png").is_none());
        let before = revision("menu_tier");
        insert("menu_tier", &full).unwrap();
        assert!(complete("menu_tier") && revision("menu_tier") != before);
        insert_menu("menu_tier", &menu).unwrap();
        assert!(complete("menu_tier") && read("menu_tier", "01_idle.png").is_some());
    }

    #[test]
    fn rejects_a_pack_for_another_fighter() {
        let bytes = pack("akio", &[("01_idle.png", b"idle")]);
        assert!(insert("kaito", &bytes).is_err());
    }

    #[test]
    fn a_pack_feeds_the_sprite_loader() {
        let mut pixmap = tiny_skia::Pixmap::new(8, 16).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(10, 200, 30, 255));
        let png = pixmap.encode_png().unwrap();
        insert("virtual-fighter", &pack("virtual-fighter", &[("idle0.png", &png)])).unwrap();
        let mut fighter: super::super::types::NativeChar = serde_json::from_str(include_str!("../../../../assets/chars/kaito.json")).unwrap();
        fighter.id = "virtual-fighter".into();
        let frame: super::super::types::NativeFrame = serde_json::from_value(serde_json::json!({"pose": "idle0", "ticks": 1})).unwrap();
        let render = crate::native::pixel::try_load(&fighter, &frame, None).expect("sprite read from the pack");
        assert_eq!(render.image.unwrap().w, 8);
    }
}
