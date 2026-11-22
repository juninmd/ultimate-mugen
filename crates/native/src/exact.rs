mod anims;
mod pack;
mod spec;

pub use anims::{export_anims, import_anims};
pub use pack::{decode_pack, encode_pack, PackMeta, META};
pub use spec::{ExactAnim, ExactFrame, MugenFile, MugenSpec};

use super::pack::read_sprite;
use super::pixel::sprite_roots;
use super::types::NativeChar;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use ume_engine::chardef::{CharDef, Source};
use ume_engine::loader::{assemble_mugen_char, MugenSources, MugenText};

fn text(f: &MugenFile) -> MugenText {
    MugenText::new(f.name.clone(), f.text.clone())
}

fn wanted_sprites(spec: &MugenSpec, only: Option<&[i32]>) -> Option<HashSet<(i32, i32)>> {
    let only = only?;
    let mut keys: HashSet<(i32, i32)> = spec.anims.iter().filter(|(id, _)| only.contains(id)).flat_map(|(_, a)| a.frames.iter().map(|f| (f.g, f.i))).collect();
    keys.extend([(9000, 0), (9000, 1)]);
    Some(keys)
}

pub type Reader<'a> = &'a (dyn Fn(&str) -> Option<Arc<Vec<u8>>> + Sync);

pub fn sources(nc: &NativeChar, spec: &MugenSpec, read: Reader, with_images: bool, only: Option<&[i32]>) -> MugenSources {
    let meta = read(META).and_then(|bytes| serde_json::from_slice::<PackMeta>(&bytes).ok());
    let wanted = if with_images { wanted_sprites(spec, only) } else { Some(HashSet::new()) };
    let decoded = meta.map(|meta| decode_pack(&meta, read, wanted.as_ref()));
    let (sff, sounds, pal_files) = match decoded {
        Some(p) => (Some(p.sff), p.sounds, p.acts),
        None => (None, HashMap::new(), vec![None; 12]),
    };
    MugenSources {
        id: format!("umf:{}", nc.id),
        name: nc.name.clone(),
        display_name: nc.name.clone(),
        author: nc.author.clone(),
        localcoord: spec.localcoord.clone(),
        mugen_version: spec.mugen_version.clone(),
        stcommon: spec.stcommon.as_ref().map(text),
        cns: text(&spec.cns),
        st: spec.st.iter().map(text).collect(),
        cmd: text(&spec.cmd),
        sff,
        anims: import_anims(&spec.anims),
        sounds,
        pal_files,
    }
}

fn rename_moves(d: &mut CharDef, spec: &MugenSpec) {
    for mv in &mut d.movelist {
        if let Some(name) = spec.names.iter().find(|(k, _)| k.eq_ignore_ascii_case(&mv.command)).map(|(_, v)| v) {
            mv.name = name.clone();
        }
    }
}

pub fn compile_exact(nc: &NativeChar, spec: &MugenSpec, pal_no: i32, with_images: bool, only: Option<&[i32]>) -> CharDef {
    let roots = sprite_roots(nc.sprite_dir());
    let root = roots.iter().find(|root| read_sprite(root, META).is_some()).cloned().unwrap_or_default();
    compile_exact_with(nc, spec, &|name| read_sprite(&root, name), pal_no, with_images, only)
}

pub fn compile_exact_with(nc: &NativeChar, spec: &MugenSpec, read: Reader, pal_no: i32, with_images: bool, only: Option<&[i32]>) -> CharDef {
    let mut d = assemble_mugen_char(sources(nc, spec, read, with_images, only), pal_no, with_images);
    let anim_no: HashMap<String, i32> = spec.anims.keys().map(|id| (id.to_string(), *id)).collect();
    super::scripts::compile_scripts(nc.scripts.as_ref(), &anim_no, &HashMap::new(), &mut d.states);
    super::scripts::script_commands(nc.scripts.as_ref(), &mut d.commands, &mut d.movelist);
    rename_moves(&mut d, spec);
    d.source = Source::Mugen;
    d.series = nc.series.clone();
    d.description = nc.description.clone().unwrap_or_default();
    d.intro = nc.intro.clone().unwrap_or_default();
    d.taunt = nc.taunt.clone().unwrap_or_default();
    d.win_quote = nc.win_quote.clone().unwrap_or_default();
    d.boss = nc.boss;
    d.element = nc.element;
    let cv = u32::from_str_radix(nc.color.trim_start_matches('#'), 16).unwrap_or(0x5a648c);
    d.color = [(cv >> 16) as u8, (cv >> 8) as u8, cv as u8];
    d.rivals = ume_engine::rivals::Rivals::new(&nc.id, &nc.rivals);
    if let Some(p) = &nc.profile {
        d.element = d.element.or_else(|| p.element.as_deref().and_then(ume_engine::element::Element::parse));
        d.weakness = p.weakness.as_deref().and_then(ume_engine::element::Element::parse);
    }
    d
}
