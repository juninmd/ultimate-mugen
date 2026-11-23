pub(super) struct Spec {
    pub glb: &'static [u8],
    pub states: &'static str,
    pub height: f32,
    pub portrait: &'static [u8],
}

const ICHIGO3D: Spec = Spec {
    glb: include_bytes!("../../assets/model3d/ichigo3d.glb"),
    states: include_str!("../../assets/model3d/ichigo3d.states.json"),
    height: 106.0,
    portrait: include_bytes!("../../assets/model3d/ichigo3d_portrait.png"),
};

const ICHIGO3D_HOLLOW: Spec = Spec {
    glb: include_bytes!("../../assets/model3d/ichigo3d_hollow.glb"),
    states: include_str!("../../assets/model3d/ichigo3d.states.json"),
    height: 118.0,
    portrait: include_bytes!("../../assets/model3d/ichigo3d_portrait.png"),
};

pub(super) fn spec(character: &str, awakened: bool) -> Option<Spec> {
    match (character, awakened) {
        ("ichigo3d", false) => Some(ICHIGO3D),
        ("ichigo3d", true) => Some(ICHIGO3D_HOLLOW),
        _ => None,
    }
}
