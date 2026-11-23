use super::epilogue::Epilogue;
use super::epilogue_store;
use ume_native::LocalText;

pub const TEXT_MAX_CHARS: usize = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Title,
    Boss,
    Reply,
    Page(usize),
}

pub fn fields(epilogue: &Epilogue) -> Vec<Field> {
    [Field::Title, Field::Boss, Field::Reply]
        .into_iter()
        .chain((0..epilogue.pages.len()).map(Field::Page))
        .collect()
}

fn slot(epilogue: &mut Epilogue, field: Field) -> Option<&mut LocalText> {
    match field {
        Field::Title => Some(&mut epilogue.title),
        Field::Boss => Some(&mut epilogue.boss),
        Field::Reply => Some(&mut epilogue.reply),
        Field::Page(n) => epilogue.pages.get_mut(n),
    }
}

pub fn read(epilogue: &Epilogue, field: Field, lang: &str) -> String {
    let mut copy = epilogue.clone();
    slot(&mut copy, field).map(|text| text.pick(lang).to_string()).unwrap_or_default()
}

pub fn write(epilogue: &mut Epilogue, field: Field, lang: &str, text: &str) {
    let clean: String = text.trim().chars().take(TEXT_MAX_CHARS).collect();
    if let Some(target) = slot(epilogue, field) {
        target.0.insert(lang.to_string(), clean);
    }
}

pub fn commit(id: &str, field: Field, lang: &str, text: &str) -> Result<(), String> {
    let mut epilogue = epilogue_store::table().get(id).cloned().ok_or_else(|| format!("unknown epilogue {id}"))?;
    write(&mut epilogue, field, lang, text);
    epilogue_store::save_override(id, epilogue)
}

#[cfg(test)]
#[path = "epilogue_edit_tests.rs"]
mod tests;
