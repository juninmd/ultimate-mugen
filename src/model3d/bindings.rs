use crate::gfx::Texture2D;
use std::cell::RefCell;
use std::collections::HashMap;
use ume_model3d::model::Posed;
use ume_model3d::{Camera, Model, StateMap, Triangle};

pub(super) struct Binding {
    pub model: Model,
    pub states: StateMap,
    pub camera: Camera,
    pub posed: Posed,
    pub triangles: Vec<Triangle>,
    pub textures: Vec<Option<Texture2D>>,
}

struct Spec {
    glb: &'static [u8],
    states: &'static str,
    height: f32,
    yaw: f32,
}

const FOX3D: Spec = Spec {
    glb: include_bytes!("../../assets/model3d/fox3d.glb"),
    states: include_str!("../../assets/model3d/fox3d.states.json"),
    height: 70.0,
    yaw: 0.0,
};

const ICHIGO3D: Spec = Spec {
    glb: include_bytes!("../../assets/model3d/ichigo3d.glb"),
    states: include_str!("../../assets/model3d/ichigo3d.states.json"),
    height: 104.0,
    yaw: 0.0,
};

fn load(id: &str) -> Option<Binding> {
    let spec = match id {
        "fox3d" => FOX3D,
        "ichigo3d" => ICHIGO3D,
        _ => return None,
    };
    let model = Model::from_glb(spec.glb).ok()?;
    let mut camera = Camera::fit_height(&model, spec.height);
    camera.yaw_offset = spec.yaw;
    let textures = model
        .images
        .iter()
        .map(|i| {
            i.as_ref()
                .map(|i| Texture2D::from_rgba8(i.width as u16, i.height as u16, &i.rgba))
        })
        .collect();
    Some(Binding {
        model,
        states: StateMap::from_json(spec.states).ok()?,
        camera,
        posed: Posed::default(),
        triangles: Vec::new(),
        textures,
    })
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<Binding>>> = RefCell::new(HashMap::new());
}

pub(super) fn with<R>(character: &str, run: impl FnOnce(&mut Binding) -> R) -> Option<R> {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let slot = cache
            .entry(character.to_string())
            .or_insert_with(|| load(character));
        slot.as_mut().map(run)
    })
}
