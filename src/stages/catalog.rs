use super::*;

pub struct StageEntry {
    pub id: String,
    pub name: String,
    pub thumb: [[u8; 3]; 3],
    pub build: Box<dyn Fn() -> Stage + Send + Sync>,
}

pub fn builtin() -> Vec<StageEntry> {
    let mut stages = vec![
        StageEntry {
            id: "dojo".into(),
            name: "Telhado do Dojo — Pôr do Sol".into(),
            thumb: [[42, 30, 92], [212, 80, 122], [255, 154, 74]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => dojo(),
                _ => illustrated("dojo", "Telhado do Dojo — Pôr do Sol", 0, StageKind::Dojo),
            }),
        },
        StageEntry {
            id: "neon".into(),
            name: "Neo-Metrópole 2099".into(),
            thumb: [[5, 3, 15], [29, 11, 58], [255, 47, 208]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => neon(),
                _ => illustrated("neon", "Neo-Metrópole 2099", 1, StageKind::Neon),
            }),
        },
        StageEntry {
            id: "bamboo".into(),
            name: "Bambuzal ao Luar".into(),
            thumb: [[4, 16, 28], [18, 58, 74], [44, 106, 106]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => bamboo(),
                _ => illustrated("bamboo", "Bambuzal ao Luar", 2, StageKind::Bamboo),
            }),
        },
        StageEntry {
            id: "plains".into(),
            name: "Planalto Rochoso".into(),
            thumb: [[44, 111, 214], [140, 198, 242], [216, 176, 128]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => plains(),
                _ => illustrated("plains", "Planalto Rochoso", 3, StageKind::Plains),
            }),
        },
        StageEntry {
            id: "spirit".into(),
            name: "Portão dos Espíritos".into(),
            thumb: [[10, 20, 32], [29, 74, 82], [154, 208, 200]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => spirit(),
                _ => illustrated("spirit", "Portão dos Espíritos", 4, StageKind::Spirit),
            }),
        },
        StageEntry {
            id: "volcano".into(),
            name: "Cratera Vulcânica".into(),
            thumb: [[18, 4, 4], [74, 14, 8], [200, 56, 26]],
            build: Box::new(|| match std::env::var("UME_STAGE_ART").as_deref() {
                Ok("procedural") => volcano(),
                _ => illustrated("volcano", "Cratera Vulcânica", 5, StageKind::Volcano),
            }),
        },
        StageEntry {
            id: "brazil".into(),
            name: "Praça do Interior — Brasil".into(),
            thumb: [[97, 59, 132], [238, 163, 69], [166, 106, 70]],
            build: Box::new(|| illustrated("brazil", "Praça do Interior — Brasil", 6, StageKind::Arena)),
        },
    ];
    stages.extend(wonders::builtin());
    stages
}
