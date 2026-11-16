use serde::Deserialize;
use std::collections::BTreeMap;
use ume_engine::stage_objects::{Breakable, PickupSpec};

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Kind {
    pub sprite: String,
    pub frames: usize,
    pub size: [f32; 2],
    pub half_width: f32,
    pub height: f32,
    pub depth: f32,
    pub life: f32,
    pub score: f32,
    pub solid: bool,
    pub debris: [f32; 3],
    pub sound: String,
}

impl Default for Kind {
    fn default() -> Self {
        Kind {
            sprite: String::new(),
            frames: 1,
            size: [40.0, 40.0],
            half_width: 16.0,
            height: 36.0,
            depth: 8.0,
            life: 100.0,
            score: 0.0,
            solid: false,
            debris: [0.8, 0.8, 0.8],
            sound: "wood".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Food {
    pub sprite: String,
    pub heal: f32,
    pub power: f32,
    pub score: f32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Placement {
    pub fight: usize,
    pub kind: String,
    pub x: f32,
    pub lane: f32,
    pub drop: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Bonus {
    pub seconds: i32,
    pub objects: Vec<Placement>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct StageObjects {
    pub objects: Vec<Placement>,
    pub bonus: Option<Bonus>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Catalog {
    pub kinds: BTreeMap<String, Kind>,
    pub pickups: BTreeMap<String, Food>,
    pub stages: BTreeMap<String, StageObjects>,
}

impl Catalog {
    pub fn parse(text: &str) -> Option<Catalog> {
        toml::from_str(text).ok()
    }

    pub fn kind(&self, index: u16) -> Option<&Kind> {
        self.kinds.values().nth(index as usize)
    }

    pub fn food(&self, index: u16) -> Option<&Food> {
        self.pickups.values().nth(index as usize)
    }

    pub fn bonus(&self, arena: &str) -> Option<&Bonus> {
        self.stages
            .get(arena)?
            .bonus
            .as_ref()
            .filter(|b| b.seconds > 0 && !b.objects.is_empty())
    }

    pub(super) fn pickup_spec(&self, name: &str) -> Option<PickupSpec> {
        let index = self.pickups.keys().position(|k| k == name)?;
        let food = &self.pickups[name];
        Some(PickupSpec {
            kind: index as u16,
            heal: food.heal.clamp(0.0, 1.0),
            power: food.power.clamp(0.0, 1.0),
            score: food.score,
        })
    }

    pub(super) fn breakable(
        &self,
        place: &Placement,
        center: f32,
        band: (f32, f32),
    ) -> Option<Breakable> {
        let index = self.kinds.keys().position(|k| *k == place.kind)?;
        let kind = &self.kinds[&place.kind];
        let lane = place.lane.clamp(band.0, band.1);
        let mut b = Breakable::new(
            index as u16,
            center + place.x,
            lane,
            kind.half_width,
            kind.height,
            kind.life.max(1.0),
        );
        b.depth = kind.depth;
        b.score = kind.score;
        b.solid = kind.solid;
        b.drop = place
            .drop
            .as_deref()
            .and_then(|name| self.pickup_spec(name));
        Some(b)
    }
}
