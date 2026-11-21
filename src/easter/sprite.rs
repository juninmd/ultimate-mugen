use crate::gfx::*;
use std::cell::OnceCell;
use ume_formats::png::decode_png;

pub(crate) const SPRITE_COUNT: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EggSprite {
    None,
    ToastyRef,
    Tumbleweed,
    ArcadeCoin,
    CrownPerfect,
    Stopwatch,
    ShieldFortress,
    OrigamiCrane,
    UfoNeon,
    KitsuneMask,
    Capybara,
    Toucan,
    ScarabLaurel,
    PirateParrot,
    RamenBowl,
    CrossedBlades,
    EclipseMoon,
}

pub(crate) struct SpriteSpec {
    pub png: &'static [u8],
    pub cell: (f32, f32),
    pub frames: usize,
    pub fps: f32,
}

const TOASTY_PNG: &[u8] = include_bytes!("../../assets/easter/toasty_ref.png");
const TUMBLEWEED_PNG: &[u8] = include_bytes!("../../assets/easter/tumbleweed.png");
const COIN_PNG: &[u8] = include_bytes!("../../assets/easter/arcade_coin.png");
const CROWN_PNG: &[u8] = include_bytes!("../../assets/easter/crown_perfect.png");
const STOPWATCH_PNG: &[u8] = include_bytes!("../../assets/easter/stopwatch.png");
const SHIELD_PNG: &[u8] = include_bytes!("../../assets/easter/shield_fortress.png");
const CRANE_PNG: &[u8] = include_bytes!("../../assets/easter/origami_crane.png");
const UFO_PNG: &[u8] = include_bytes!("../../assets/easter/ufo_neon.png");
const KITSUNE_PNG: &[u8] = include_bytes!("../../assets/easter/kitsune_mask.png");
const CAPYBARA_PNG: &[u8] = include_bytes!("../../assets/easter/capybara.png");
const TOUCAN_PNG: &[u8] = include_bytes!("../../assets/easter/toucan.png");
const SCARAB_PNG: &[u8] = include_bytes!("../../assets/easter/scarab_laurel.png");
const PARROT_PNG: &[u8] = include_bytes!("../../assets/easter/pirate_parrot.png");
const RAMEN_PNG: &[u8] = include_bytes!("../../assets/easter/ramen_bowl.png");
const BLADES_PNG: &[u8] = include_bytes!("../../assets/easter/crossed_blades.png");
const ECLIPSE_PNG: &[u8] = include_bytes!("../../assets/easter/eclipse_moon.png");

impl EggSprite {
    pub(crate) fn index(self) -> Option<usize> {
        match self {
            EggSprite::None => None,
            EggSprite::ToastyRef => Some(0),
            EggSprite::Tumbleweed => Some(1),
            EggSprite::ArcadeCoin => Some(2),
            EggSprite::CrownPerfect => Some(3),
            EggSprite::Stopwatch => Some(4),
            EggSprite::ShieldFortress => Some(5),
            EggSprite::OrigamiCrane => Some(6),
            EggSprite::UfoNeon => Some(7),
            EggSprite::KitsuneMask => Some(8),
            EggSprite::Capybara => Some(9),
            EggSprite::Toucan => Some(10),
            EggSprite::ScarabLaurel => Some(11),
            EggSprite::PirateParrot => Some(12),
            EggSprite::RamenBowl => Some(13),
            EggSprite::CrossedBlades => Some(14),
            EggSprite::EclipseMoon => Some(15),
        }
    }

    pub(crate) fn spec(self) -> Option<SpriteSpec> {
        match self {
            EggSprite::None => None,
            EggSprite::ToastyRef => Some(SpriteSpec { png: TOASTY_PNG, cell: (48.0, 48.0), frames: 2, fps: 7.0 }),
            EggSprite::Tumbleweed => Some(SpriteSpec { png: TUMBLEWEED_PNG, cell: (32.0, 32.0), frames: 4, fps: 10.0 }),
            EggSprite::ArcadeCoin => Some(SpriteSpec { png: COIN_PNG, cell: (32.0, 32.0), frames: 4, fps: 10.0 }),
            EggSprite::CrownPerfect => Some(SpriteSpec { png: CROWN_PNG, cell: (32.0, 32.0), frames: 2, fps: 5.0 }),
            EggSprite::Stopwatch => Some(SpriteSpec { png: STOPWATCH_PNG, cell: (32.0, 32.0), frames: 2, fps: 8.0 }),
            EggSprite::ShieldFortress => Some(SpriteSpec { png: SHIELD_PNG, cell: (32.0, 32.0), frames: 2, fps: 6.0 }),
            EggSprite::OrigamiCrane => Some(SpriteSpec { png: CRANE_PNG, cell: (32.0, 32.0), frames: 2, fps: 6.0 }),
            EggSprite::UfoNeon => Some(SpriteSpec { png: UFO_PNG, cell: (32.0, 32.0), frames: 4, fps: 9.0 }),
            EggSprite::KitsuneMask => Some(SpriteSpec { png: KITSUNE_PNG, cell: (32.0, 32.0), frames: 2, fps: 4.0 }),
            EggSprite::Capybara => Some(SpriteSpec { png: CAPYBARA_PNG, cell: (32.0, 32.0), frames: 4, fps: 7.0 }),
            EggSprite::Toucan => Some(SpriteSpec { png: TOUCAN_PNG, cell: (32.0, 32.0), frames: 2, fps: 7.0 }),
            EggSprite::ScarabLaurel => Some(SpriteSpec { png: SCARAB_PNG, cell: (32.0, 32.0), frames: 2, fps: 5.0 }),
            EggSprite::PirateParrot => Some(SpriteSpec { png: PARROT_PNG, cell: (32.0, 32.0), frames: 2, fps: 8.0 }),
            EggSprite::RamenBowl => Some(SpriteSpec { png: RAMEN_PNG, cell: (32.0, 32.0), frames: 2, fps: 4.0 }),
            EggSprite::CrossedBlades => Some(SpriteSpec { png: BLADES_PNG, cell: (32.0, 32.0), frames: 2, fps: 6.0 }),
            EggSprite::EclipseMoon => Some(SpriteSpec { png: ECLIPSE_PNG, cell: (32.0, 32.0), frames: 2, fps: 4.0 }),
        }
    }
}

thread_local! {
    static TEXTURES: [OnceCell<Option<Texture2D>>; SPRITE_COUNT] = [const { OnceCell::new() }; SPRITE_COUNT];
}

fn load_texture(png: &[u8]) -> Option<Texture2D> {
    let decoded = decode_png(png).ok()?;
    let rgba = match &decoded.rgba {
        Some(bytes) => bytes.clone(),
        None => ume_formats::png::png_indexed_to_rgba(&decoded),
    };
    let tex = Texture2D::from_rgba8(decoded.width as u16, decoded.height as u16, &rgba);
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

pub(crate) fn draw_sprite(kind: EggSprite, x: f32, y: f32, size: f32, t: f32, alpha: f32, flip: bool, rot: f32) {
    let (Some(idx), Some(spec)) = (kind.index(), kind.spec()) else {
        return;
    };
    let frame = ((t * spec.fps) as usize) % spec.frames.max(1);
    TEXTURES.with(|cache| {
        let tex = cache[idx].get_or_init(|| load_texture(spec.png));
        let Some(tex) = tex.as_ref() else {
            return;
        };
        draw_texture_ex(
            tex,
            x - size * 0.5,
            y - size * 0.5,
            Color::new(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0)),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                source: Some(Rect::new(frame as f32 * spec.cell.0, 0.0, spec.cell.0, spec.cell.1)),
                rotation: rot,
                flip_x: flip,
                ..Default::default()
            },
        );
    });
}
