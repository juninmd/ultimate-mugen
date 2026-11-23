use super::*;

pub(super) const ID: &str = "airport3d";
const NAME: &str = "Aeroporto 3D — Pôr do Sol";

fn flat_backdrop() -> Arc<ImageData> {
    ImageData::new(16, 9, [[40u8, 46, 98, 255]; 144].concat())
}

pub(super) fn entry() -> StageEntry {
    StageEntry {
        id: ID.into(),
        name: NAME.into(),
        thumb: [[51, 61, 133], [153, 105, 130], [128, 133, 143]],
        build: Arc::new(stage),
    }
}

pub(super) fn stage() -> Stage {
    Stage {
        info: info(NAME),
        art: StageArt::Illustrated {
            image: preview(),
            frames: Vec::new(),
            ambience: ambience::profile_for(ID),
            span: 0.0,
        },
        music: 21,
        kind: StageKind::Airport,
        bgm: None,
    }
}

pub(super) fn preview() -> Arc<ImageData> {
    crate::model3d::airport_still().unwrap_or_else(flat_backdrop)
}
