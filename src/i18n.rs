use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { Pt, En }

pub const LANGS: [(Lang, &str); 2] = [(Lang::Pt, "Português (Brasil)"), (Lang::En, "English")];

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set(l: Lang) { CURRENT.store(l as u8, Ordering::Relaxed); }
pub fn get() -> Lang { if CURRENT.load(Ordering::Relaxed) == 1 { Lang::En } else { Lang::Pt } }
pub fn tr<'a>(pt: &'a str, en: &'a str) -> &'a str { if get() == Lang::En { en } else { pt } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_active_language() {
        set(Lang::En);
        assert_eq!(tr("sim", "yes"), "yes");
        set(Lang::Pt);
        assert_eq!(tr("sim", "yes"), "sim");
    }
}
