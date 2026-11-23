use crate::gfx::Texture2D;
use std::collections::HashMap;
use ume_formats::screenpack::Storyboard;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum After {
    FirstFight,
    Fight {
        index: usize,
        is_boss: bool,
        boss_count: usize,
    },
    Results {
        winner: usize,
        cur: usize,
    },
    Ending,
}

impl After {
    pub(super) fn slot(self, fight_count: usize) -> usize {
        match self {
            Self::FirstFight => 0,
            Self::Fight { index, .. } => index,
            Self::Results { .. } | Self::Ending => fight_count,
        }
    }

    pub(super) fn continue_app(self, app: &mut crate::app::App) {
        match self {
            Self::FirstFight => app.begin_arcade_first_fight(),
            Self::Fight {
                index,
                is_boss,
                boss_count,
            } => app.continue_arcade_fight(index, is_boss, boss_count),
            Self::Results { winner, cur } => {
                app.screen = crate::app::Screen::Results { winner, cur }
            }
            Self::Ending => {
                app.screen = crate::app::Screen::Cutscene {
                    kind: 5,
                    page: 0,
                    t: 0.0,
                }
            }
        }
    }
}

pub(super) struct Sprite {
    pub(super) texture: Texture2D,
    pub(super) axis: [f32; 2],
}

pub struct Playback {
    pub(super) storyboard: Storyboard,
    pub(super) sprites: HashMap<(i32, i32), Sprite>,
    pub(super) fade_colors: Vec<(Option<[u8; 3]>, Option<[u8; 3]>)>,
}
