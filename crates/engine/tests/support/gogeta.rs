#![allow(dead_code)]
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::commands::{CmdSymbol, CommandDef};
use ume_engine::common::{common_def, flat_stage};
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, Vfs};

pub fn dir() -> Option<PathBuf> {
    std::env::var("UME_CHAR_DIR")
        .or_else(|_| std::env::var("UME_GOGETA_DIR"))
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

pub fn def_name() -> Option<String> {
    if let Ok(n) = std::env::var("UME_CHAR_DEF") {
        return Some(n);
    }
    let mut names: Vec<String> = std::fs::read_dir(dir()?)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.to_lowercase().ends_with(".def"))
        .collect();
    names.sort();
    names.into_iter().next()
}

pub fn cmd_path() -> Option<PathBuf> {
    let d = dir()?;
    let def = std::fs::read(d.join(def_name()?)).ok()?;
    let text: String = def.iter().map(|&b| b as char).collect();
    for l in text.lines() {
        let l = l.trim();
        if l.to_lowercase().starts_with("cmd") {
            if let Some((_, v)) = l.split_once('=') {
                let v = v.split(';').next()?.trim();
                return Some(d.join(v));
            }
        }
    }
    None
}

pub fn load(pal: i32) -> Option<Arc<CharDef>> {
    let p = dir()?;
    let mut v = Vfs::new();
    add_dir(&mut v, &p, "g").unwrap();
    let def = def_name()?;
    Some(Arc::new(
        load_mugen_char(&v, &format!("g/{def}"), pal, true).unwrap(),
    ))
}

fn step_frames(s: &CmdSymbol, out: &mut Vec<u32>) {
    let bits = s.dir | s.btns;
    let bits =
        (bits & !12) | if s.dir & 4 != 0 { 4 } else { 0 } | if s.dir & 8 != 0 { 8 } else { 0 };
    let hold_len = if s.charge > 0 {
        s.charge as usize + 3
    } else if s.hold {
        6
    } else {
        2
    };
    if s.neutral {
        out.extend([0, 0]);
        return;
    }
    for _ in 0..hold_len {
        out.push(bits);
    }
    if s.release {
        out.push(0);
    } else if !s.hold {
        out.push(0);
    }
}

pub fn synth(c: &CommandDef) -> Vec<u32> {
    let mut v = vec![0u32; 2];
    for s in &c.steps {
        step_frames(s, &mut v);
    }
    v.extend([0u32; 3]);
    v
}

pub struct Duel {
    pub m: Match,
    pub q: [std::collections::VecDeque<u32>; 2],
}
impl Duel {
    pub fn new(d: Arc<CharDef>, gap: f32) -> Self {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(d.clone(), 0, 0, 1);
        m.add_player(d, 1, 0, 2);
        m.start_round();
        let mut r = Duel {
            m,
            q: Default::default(),
        };
        for _ in 0..800 {
            r.tick(1);
            if r.m.round_state == 2 {
                break;
            }
        }
        r.m.fighters[0].pos.x = -gap / 2.0;
        r.m.fighters[1].pos.x = gap / 2.0;
        r.m.fighters[0].power = 3000.0;
        r.m.fighters[1].life = 100000.0;
        r.m.fighters[1].life_max = 100000.0;
        r
    }
    pub fn tick(&mut self, n: usize) {
        for _ in 0..n {
            let i = [
                self.q[0].pop_front().unwrap_or(0),
                self.q[1].pop_front().unwrap_or(0),
            ];
            self.m.tick(&i);
        }
    }
    pub fn push(&mut self, p: usize, s: &[u32]) {
        self.q[p].extend(s.iter().copied());
    }
    pub fn jump(&mut self) {
        self.push(0, &[1, 1, 1, 0]);
        for _ in 0..60 {
            self.tick(1);
            if self.m.fighters[0].state_no == 50 && self.m.fighters[0].pos.y < -30.0 {
                break;
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Outcome {
    pub command_fired: bool,
    pub p1_states: BTreeSet<i32>,
    pub p2_states: BTreeSet<i32>,
    pub p1_anims: BTreeSet<i32>,
    pub dmg: f32,
    pub helpers: usize,
    pub p1_recovered: bool,
    pub p2_recovered: bool,
    pub p1_final: (i32, bool),
    pub p2_final: (i32, bool),
}

pub fn run_command(d: &Arc<CharDef>, c: &CommandDef, gap: f32, air: bool, ticks: usize) -> Outcome {
    let mut r = Duel::new(d.clone(), gap);
    if air {
        r.jump();
    }
    let seq = synth(c);
    r.push(0, &seq);
    let mut o = Outcome::default();
    let life0 = r.m.fighters[1].life;
    for _ in 0..ticks {
        r.tick(1);
        if r.m.fighters[0].cmd.active.contains(c.name.as_str()) {
            o.command_fired = true;
        }
        o.p1_states.insert(r.m.fighters[0].state_no);
        o.p1_anims.insert(r.m.fighters[0].anim_no);
        o.p2_states.insert(r.m.fighters[1].state_no);
        o.helpers = o
            .helpers
            .max(r.m.fighters.iter().filter(|f| f.is_helper).count());
    }
    o.dmg = life0 - r.m.fighters[1].life;
    r.m.fighters[1].pos.x = 300.0;
    r.tick(900);
    let f = |i: usize| (r.m.fighters[i].state_no, r.m.fighters[i].ctrl);
    o.p1_final = f(0);
    o.p2_final = f(1);
    o.p1_recovered =
        (r.m.fighters[0].ctrl || r.m.fighters[0].move_type == 'I') && r.m.fighters[0].alive;
    o.p2_recovered =
        (r.m.fighters[1].ctrl || r.m.fighters[1].move_type == 'I') && r.m.fighters[1].alive;
    o
}
