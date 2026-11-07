use ume_engine::chardef::ImageData;
use super::{decode_illustration, illustrated, StageEntry, StageKind};
use std::sync::{Arc, OnceLock};

pub const WONDER_IDS: [&str; 14] = [
    "great_wall",
    "petra",
    "christ_redeemer",
    "texas",
    "chichen_itza",
    "colosseum",
    "taj_mahal",
    "pyramids_giza",
    "hanging_gardens",
    "temple_artemis",
    "statue_zeus",
    "mausoleum_halicarnassus",
    "colossus_rhodes",
    "lighthouse_alexandria",
];

const THUMB_BYTES: [&[u8]; 14] = [
    include_bytes!("../../assets/stages/thumbs/great_wall.png"),
    include_bytes!("../../assets/stages/thumbs/petra.png"),
    include_bytes!("../../assets/stages/thumbs/christ_redeemer.png"),
    include_bytes!("../../assets/stages/thumbs/texas.png"),
    include_bytes!("../../assets/stages/thumbs/chichen_itza.png"),
    include_bytes!("../../assets/stages/thumbs/colosseum.png"),
    include_bytes!("../../assets/stages/thumbs/taj_mahal.png"),
    include_bytes!("../../assets/stages/thumbs/pyramids_giza.png"),
    include_bytes!("../../assets/stages/thumbs/hanging_gardens.png"),
    include_bytes!("../../assets/stages/thumbs/temple_artemis.png"),
    include_bytes!("../../assets/stages/thumbs/statue_zeus.png"),
    include_bytes!("../../assets/stages/thumbs/mausoleum_halicarnassus.png"),
    include_bytes!("../../assets/stages/thumbs/colossus_rhodes.png"),
    include_bytes!("../../assets/stages/thumbs/lighthouse_alexandria.png"),
];

#[cfg(not(target_arch = "wasm32"))]
const STAGE_BYTES: [&[u8]; 14] = [
    include_bytes!("../../assets/stages/great_wall.png"),
    include_bytes!("../../assets/stages/petra.png"),
    include_bytes!("../../assets/stages/christ_redeemer.png"),
    include_bytes!("../../assets/stages/texas.png"),
    include_bytes!("../../assets/stages/chichen_itza.png"),
    include_bytes!("../../assets/stages/colosseum.png"),
    include_bytes!("../../assets/stages/taj_mahal.png"),
    include_bytes!("../../assets/stages/pyramids_giza.png"),
    include_bytes!("../../assets/stages/hanging_gardens.png"),
    include_bytes!("../../assets/stages/temple_artemis.png"),
    include_bytes!("../../assets/stages/statue_zeus.png"),
    include_bytes!("../../assets/stages/mausoleum_halicarnassus.png"),
    include_bytes!("../../assets/stages/colossus_rhodes.png"),
    include_bytes!("../../assets/stages/lighthouse_alexandria.png"),
];

pub fn illustration_preview(id: &str) -> Option<Arc<ImageData>> {
    static IMAGES: [OnceLock<Arc<ImageData>>; 14] = [const { OnceLock::new() }; 14];
    let index = WONDER_IDS.iter().position(|name| *name == id)?;
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(THUMB_BYTES[index]).expect("stage thumbnail"))
            .clone(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn illustration(id: &str) -> Option<Arc<ImageData>> {
    static IMAGES: [OnceLock<Arc<ImageData>>; 14] = [const { OnceLock::new() }; 14];
    let index = WONDER_IDS.iter().position(|name| *name == id)?;
    Some(
        IMAGES[index]
            .get_or_init(|| decode_illustration(STAGE_BYTES[index]).expect("stage illustration"))
            .clone(),
    )
}

fn entry(id: &'static str, name: &'static str, music: u32, thumb: [[u8; 3]; 3]) -> StageEntry {
    StageEntry {
        id: id.into(),
        name: name.into(),
        thumb,
        build: Box::new(move || illustrated(id, name, music, StageKind::Custom)),
    }
}

pub fn builtin() -> Vec<StageEntry> {
    vec![
        entry(
            "great_wall",
            "Grande Muralha da China",
            0,
            [[145, 138, 144], [128, 122, 115], [95, 80, 66]],
        ),
        entry(
            "petra",
            "Petra — Al-Khazneh",
            3,
            [[139, 105, 90], [165, 87, 51], [185, 116, 68]],
        ),
        entry(
            "christ_redeemer",
            "Cristo Redentor — Corcovado",
            3,
            [[110, 96, 113], [118, 85, 77], [121, 94, 90]],
        ),
        entry(
            "texas",
            "Cidade do Interior — Texas, EUA",
            3,
            [[198, 90, 99], [164, 89, 63], [201, 112, 58]],
        ),
        entry(
            "chichen_itza",
            "Chichén Itzá — Templo de Kukulcán",
            4,
            [[80, 51, 94], [92, 61, 62], [135, 113, 91]],
        ),
        entry(
            "colosseum",
            "Coliseu de Roma",
            5,
            [[167, 126, 115], [127, 81, 65], [221, 165, 86]],
        ),
        entry(
            "taj_mahal",
            "Taj Mahal — Palácio de Mármore",
            0,
            [[223, 205, 207], [174, 151, 129], [219, 211, 203]],
        ),
        entry(
            "pyramids_giza",
            "Pirâmides de Gizé",
            3,
            [[216, 129, 44], [141, 91, 46], [160, 105, 51]],
        ),
        entry(
            "hanging_gardens",
            "Jardins Suspensos da Babilônia",
            4,
            [[168, 200, 192], [154, 160, 125], [123, 140, 132]],
        ),
        entry(
            "temple_artemis",
            "Templo de Ártemis em Éfeso",
            4,
            [[47, 74, 78], [55, 86, 86], [146, 171, 169]],
        ),
        entry(
            "statue_zeus",
            "Estátua de Zeus em Olímpia",
            5,
            [[81, 62, 43], [113, 79, 41], [112, 93, 79]],
        ),
        entry(
            "mausoleum_halicarnassus",
            "Mausoléu de Halicarnasso",
            0,
            [[228, 183, 127], [186, 164, 119], [211, 194, 172]],
        ),
        entry(
            "colossus_rhodes",
            "Colosso de Rodes",
            5,
            [[64, 106, 122], [77, 102, 117], [94, 110, 101]],
        ),
        entry(
            "lighthouse_alexandria",
            "Farol de Alexandria — Pharos",
            1,
            [[71, 67, 58], [29, 38, 54], [149, 153, 156]],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wonder_stages_decode_full_images_and_thumbnails() {
        assert_eq!(WONDER_IDS.len(), 14);
        for id in WONDER_IDS {
            let thumb = illustration_preview(id).expect("thumb");
            assert_eq!((thumb.w, thumb.h), (240, 135));
            #[cfg(not(target_arch = "wasm32"))]
            {
                let full = illustration(id).expect("full illustration");
                assert_eq!((full.w, full.h), (1672, 941));
            }
        }
    }

    #[test]
    fn wonder_entries_build_valid_stages() {
        let entries = builtin();
        assert_eq!(entries.len(), 14);
        for entry in entries {
            assert!(!entry.name.is_empty());
            let stage = (entry.build)();
            assert_eq!(stage.info.name, entry.name);
        }
    }
}
