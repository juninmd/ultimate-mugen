use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use ume_engine::chardef::{CharDef, Controller, StateDef};

#[derive(Default)]
pub struct Section {
    pub compared: usize,
    pub differ: Vec<String>,
}

pub type Structural = BTreeMap<&'static str, Section>;

fn controller_key(c: &Controller) -> String {
    format!("{}|{}|{:?}|{:?}|{:?}|{}|{}", c.kind, c.label, c.trigger_all, c.triggers, c.params, c.persistent, c.ignore_hitpause)
}

fn state_key(s: &StateDef) -> String {
    let ctrls: Vec<String> = s.controllers.iter().map(controller_key).collect();
    format!("{:?}|{}", s.params, ctrls.join("\n"))
}

fn compare<K: Ord + Hash + Debug + Clone, V>(a: &HashMap<K, V>, b: &HashMap<K, V>, key: impl Fn(&V) -> String) -> Section {
    let keys: BTreeSet<&K> = a.keys().chain(b.keys()).collect();
    let mut out = Section::default();
    for k in keys {
        out.compared += 1;
        match (a.get(k), b.get(k)) {
            (Some(x), Some(y)) if key(x) == key(y) => {}
            (Some(_), Some(_)) => out.differ.push(format!("{k:?}: different")),
            (Some(_), None) => out.differ.push(format!("{k:?}: missing in UMF")),
            (None, _) => out.differ.push(format!("{k:?}: only in UMF")),
        }
    }
    out
}

fn image_key(s: &ume_engine::chardef::Sprite) -> String {
    let px = s.image.as_ref().map(|i| (i.w, i.h, ume_formats::vfs::content_hash(&i.rgba)));
    format!("{}|{}|{}|{}|{}|{px:?}", s.width, s.height, s.axis_x, s.axis_y, s.scale)
}

pub fn structural(orig: &CharDef, umf: &CharDef) -> Structural {
    let mut out = Structural::new();
    out.insert("constants", compare(&orig.constants, &umf.constants, |v| v.to_bits().to_string()));
    out.insert("states", compare(&orig.states, &umf.states, state_key));
    let cmds = |d: &CharDef| d.commands.iter().enumerate().map(|(i, c)| ((i, c.name.clone()), format!("{}|{:?}|{}|{}|{}", c.raw, c.steps, c.time, c.buffer_time, c.pause_buffer))).collect::<HashMap<_, _>>();
    out.insert("commands", compare(&cmds(orig), &cmds(umf), String::clone));
    out.insert("anims", compare(&orig.anims, &umf.anims, |a| format!("{:?}|{}|{}", a.frames, a.loop_start, a.total_time)));
    out.insert("sprites", compare(&orig.sprites, &umf.sprites, image_key));
    out.insert("indexed", compare(&orig.indexed, &umf.indexed, |s| format!("{}|{}|{:?}|{}|{:?}", s.w, s.h, s.pal, ume_formats::vfs::content_hash(&s.pixels), s.palette.as_ref().map(|p| ume_formats::vfs::content_hash(p)))));
    out.insert("palettes", compare(&orig.palettes, &umf.palettes, |p| ume_formats::vfs::content_hash(p)));
    out.insert("sounds", compare(&orig.sounds, &umf.sounds, |s| ume_formats::vfs::content_hash(s)));
    let size = |d: &CharDef| format!("{}x{} v{} {:?}", d.local_w, d.local_h, d.mugen_version, [d.size_ground_back, d.size_ground_front, d.size_air_back, d.size_air_front, d.size_height, d.size_attack_dist]);
    let mut frame = Section { compared: 1, differ: vec![] };
    if size(orig) != size(umf) {
        frame.differ.push(format!("{} vs {}", size(orig), size(umf)));
    }
    out.insert("frame", frame);
    out
}
