use crate::model::Kind;
use crate::safety::{file_stem, parent_dir};
use std::collections::HashSet;

pub(crate) fn is_fighter_sfx(path: &str, sprite_dirs: &HashSet<&str>) -> bool {
    let stem = file_stem(path).to_ascii_lowercase();
    let numbered = stem.strip_prefix("snd_").is_some_and(|r| !r.is_empty() && r.split('_').all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())));
    numbered || sprite_dirs.contains(parent_dir(path))
}

/// Installed music entries saved before sfx were excluded from the catalog.
pub fn is_stale_fighter_sfx_music(kind: Kind, item_id: &str) -> bool {
    kind == Kind::Music && is_fighter_sfx(item_id.strip_prefix("music:").unwrap_or(item_id), &HashSet::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_numbered_fighter_sfx_and_rejects_music_tracks() {
        let empty = HashSet::new();
        assert!(is_fighter_sfx("chars/ryu/snd_0.wav", &empty));
        assert!(is_fighter_sfx("chars/ryu/SND_10_2.ogg", &empty));
        assert!(!is_fighter_sfx("sound/boss_theme.mp3", &empty));
        assert!(!is_fighter_sfx("sound/snd_.wav", &empty));
        assert!(!is_fighter_sfx("sound/snd_1__2.wav", &empty));
        assert!(!is_fighter_sfx("sound/snd_intro.ogg", &empty));
    }

    #[test]
    fn treats_audio_inside_sprite_dirs_as_fighter_sfx() {
        let mut dirs = HashSet::new();
        dirs.insert("chars/kaito");
        assert!(is_fighter_sfx("chars/kaito/shout.wav", &dirs));
        assert!(!is_fighter_sfx("stages/dojo/theme.ogg", &dirs));
    }

    #[test]
    fn detects_stale_fighter_sfx_only_for_music_kind() {
        assert!(is_stale_fighter_sfx_music(Kind::Music, "music:chars/ryu/snd_5_0.wav"));
        assert!(!is_stale_fighter_sfx_music(Kind::Music, "music:stages/colosseum.ogg"));
        assert!(!is_stale_fighter_sfx_music(Kind::Character, "music:chars/ryu/snd_5_0.wav"));
    }
}

