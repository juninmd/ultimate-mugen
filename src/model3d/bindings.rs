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
    pub anim_names: HashMap<i32, String>,
    pub height: f32,
}

fn anim_names(character: &str) -> HashMap<i32, String> {
    let Some((_, json)) = crate::registry::BUILTIN
        .iter()
        .find(|(id, _)| *id == character)
    else {
        return HashMap::new();
    };
    let Ok(native) = ume_native::parse_char(json) else {
        return HashMap::new();
    };
    ume_native::anim_numbers(&native)
        .into_iter()
        .map(|(name, number)| (number, name))
        .collect()
}

fn load(character: &str, awakened: bool) -> Option<Binding> {
    let spec = super::specs::spec(character, awakened)?;
    let model = Model::from_glb(spec.glb).ok()?;
    let camera = Camera::fit_height(&model, spec.height);
    Some(Binding {
        model,
        states: StateMap::from_json(spec.states).ok()?,
        camera,
        posed: Posed::default(),
        triangles: Vec::new(),
        anim_names: anim_names(character),
        height: spec.height,
    })
}

thread_local! {
    static CACHE: RefCell<HashMap<(String, bool), Option<Binding>>> = RefCell::new(HashMap::new());
}

pub(super) fn with<R>(
    character: &str,
    awakened: bool,
    run: impl FnOnce(&mut Binding) -> R,
) -> Option<R> {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let slot = cache
            .entry((character.to_string(), awakened))
            .or_insert_with(|| load(character, awakened));
        slot.as_mut().map(run)
    })
}
