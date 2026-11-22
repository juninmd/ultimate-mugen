use super::{Epilogue, Table};
use ume_native::LocalText;

const TALE_PREFIX: &str = "*:tale";
const NAME_SLOT: &str = "{name}";
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn stable_hash(text: &str) -> u64 {
    text.bytes()
        .fold(FNV_OFFSET, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}

fn fill(text: &LocalText, name: &str) -> LocalText {
    LocalText(
        text.0
            .iter()
            .map(|(lang, line)| (lang.clone(), line.replace(NAME_SLOT, name)))
            .collect(),
    )
}

pub fn tale_for(table: &Table, fighter_id: &str, name: &str) -> Option<Epilogue> {
    if name.is_empty() {
        return None;
    }
    let tales: Vec<&Epilogue> = table
        .iter()
        .filter(|(key, _)| key.starts_with(TALE_PREFIX))
        .map(|(_, tale)| tale)
        .collect();
    let tale = tales.get(stable_hash(fighter_id) as usize % tales.len().max(1))?;
    Some(Epilogue {
        scene: tale.scene.clone(),
        title: fill(&tale.title, name),
        boss: fill(&tale.boss, name),
        reply: fill(&tale.reply, name),
        pages: tale.pages.iter().map(|page| fill(page, name)).collect(),
    })
}
