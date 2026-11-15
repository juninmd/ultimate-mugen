//! `[Sky]` section of a MUGEN stage .def: an animated sky drawn behind the fighters.
//! Four looks are available (`type = aurora | godrays | stars | clouds`); the renderer
//! generates their textures, so the section needs no image files.

use crate::ini::{find_section, nums, Section};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyKind {
    Aurora,
    GodRays,
    Stars,
    Clouds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StageSky {
    pub kind: SkyKind,
    /// RGB in 0..1; `color = r, g, b` (0..255) in the .def.
    pub color: [f32; 3],
    /// 0..1, opacity of the effect at its brightest point.
    pub intensity: f32,
    /// Speed multiplier for the drift and the pulse (0..8).
    pub speed: f32,
}

pub fn parse_sky(secs: &[Section]) -> Option<StageSky> {
    let s = find_section(secs, "sky")?;
    let kind = match s.get("type")?.trim().to_ascii_lowercase().as_str() {
        "aurora" => SkyKind::Aurora,
        "godrays" => SkyKind::GodRays,
        "stars" => SkyKind::Stars,
        "clouds" => SkyKind::Clouds,
        _ => return None,
    };
    let c = nums(s.get("color"), &[255.0, 255.0, 255.0]);
    let channel = |i: usize| (c.get(i).copied().unwrap_or(255.0) / 255.0).clamp(0.0, 1.0);
    Some(StageSky {
        kind,
        color: [channel(0), channel(1), channel(2)],
        intensity: nums(s.get("intensity"), &[0.6])[0].clamp(0.0, 1.0),
        speed: nums(s.get("speed"), &[1.0])[0].clamp(0.0, 8.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    #[test]
    fn reads_every_kind_and_ignores_unknown() {
        for (ty, want) in [("aurora", SkyKind::Aurora), ("GodRays", SkyKind::GodRays), ("stars", SkyKind::Stars), ("clouds", SkyKind::Clouds)] {
            let secs = parse_ini(&format!("[Sky]\ntype = {ty}\n"));
            assert_eq!(parse_sky(&secs).map(|s| s.kind), Some(want));
        }
        assert!(parse_sky(&parse_ini("[Sky]\ntype = lava\n")).is_none());
        assert!(parse_sky(&parse_ini("[Info]\nname = x\n")).is_none());
    }

    #[test]
    fn colour_and_ranges_are_clamped() {
        let s = parse_sky(&parse_ini("[Sky]\ntype = aurora\ncolor = 255, 0\nintensity = 4\nspeed = -2\n")).unwrap();
        assert_eq!(s.color, [1.0, 0.0, 1.0]);
        assert_eq!(s.intensity, 1.0);
        assert_eq!(s.speed, 0.0);
    }
}
