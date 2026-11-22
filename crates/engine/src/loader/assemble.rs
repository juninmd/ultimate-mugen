use super::format::local_size;
use super::sources::MugenSources;
use crate::chardef::{default_constants, CharDef, Source};
use crate::cns::{parse_cmd, parse_constants, parse_states, scale_default_constants};
use crate::commands::parse_command;
use std::collections::{HashMap, HashSet};

pub fn assemble_mugen_char(src: MugenSources, pal_no: i32, with_images: bool) -> CharDef {
    let (local_w, local_h) = local_size(src.localcoord.as_deref());
    let mut constants = default_constants();
    let mut provided = HashSet::new();
    let mut states = HashMap::new();
    let mut spans = crate::diagnostics::SourceSpans::default();
    if let Some(common) = &src.stcommon {
        spans.track(&common.name, || crate::zss::parse_state_file(&common.name, &common.text, &mut states, false));
    }
    parse_constants(&src.cns.text, &mut constants, &mut provided);
    scale_default_constants(&mut constants, &provided, local_w as f64 / 320.0);
    crate::perf::time("cns:parse", || spans.track(&src.cns.name, || parse_states(&src.cns.text, &mut states, true)));
    for st in &src.st {
        spans.track(&st.name, || crate::zss::parse_state_file(&st.name, &st.text, &mut states, true));
    }
    let (mut commands, cmd_states) = spans.track(&src.cmd.name, || parse_cmd(&src.cmd.text));
    for (id, sd) in cmd_states {
        if id < 0 && states.contains_key(&id) {
            if let Some(s) = states.get_mut(&id) {
                s.controllers.extend(sd.controllers)
            }
        } else {
            states.insert(id, sd);
        }
    }
    for (n, c) in [("holdfwd", "/$F"), ("holdback", "/$B"), ("holdup", "/$U"), ("holddown", "/$D")] {
        if !commands.iter().any(|x| x.name == n) {
            commands.push(parse_command(n, c, 1, 1));
        }
    }
    let tables = src.sff.as_ref().map(|sff| super::sprites::sprite_tables(sff, &src.pal_files, pal_no, with_images));
    let movelist = super::movelist::movelist(&commands, &states);
    let ai_commands = super::movelist::ai_commands(&commands);
    let mut d = CharDef::empty(&src.id);
    d.name = src.name;
    d.display_name = src.display_name;
    d.author = src.author;
    d.source = Source::Mugen;
    d.local_w = local_w;
    d.local_h = local_h;
    d.mugen_version = crate::chardef::parse_mugen_version(src.mugen_version.as_deref());
    d.constants = constants;
    d.refresh_sizes();
    if let Some(t) = tables {
        d.portrait = t.sprites.get(&(9000, 0)).cloned();
        d.sprites = t.sprites;
        d.palettes = t.palettes;
        d.indexed = t.indexed;
    }
    d.anims = src.anims;
    d.commands = commands;
    d.states = states;
    d.sounds = src.sounds;
    d.ai_commands = ai_commands;
    d.movelist = movelist;
    d.series = "M.U.G.E.N".into();
    d.source_spans = spans;
    d
}
