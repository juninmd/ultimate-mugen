use std::path::Path;
use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, Vfs};

const SP: &str =
    "/tmp/claude-0/-home-user-ultimate-mugen/fa14b17d-4289-5fed-86c2-aacfb04ddae6/scratchpad/sp";

fn kfm(dir: &str, def: &str) -> Option<Arc<ume_engine::chardef::CharDef>> {
    let p = Path::new(SP).join("chars").join(dir);
    if !p.exists() {
        return None;
    }
    let mut v = Vfs::new();
    add_dir(&mut v, &p, dir).unwrap();
    Some(Arc::new(load_mugen_char(&v, def, 1, false).unwrap()))
}

struct Run {
    m: Match,
    q: [std::collections::VecDeque<u32>; 2],
}
impl Run {
    fn new(a: Arc<ume_engine::chardef::CharDef>, b: Arc<ume_engine::chardef::CharDef>) -> Self {
        let mut m = Match::new(common_def(), None, flat_stage());
        m.add_player(a, 0, 0, 1);
        m.add_player(b, 1, 0, 2);
        m.start_round();
        Run {
            m,
            q: Default::default(),
        }
    }
    fn tick(&mut self, n: usize) {
        for _ in 0..n {
            let i = [
                self.q[0].pop_front().unwrap_or(0),
                self.q[1].pop_front().unwrap_or(0),
            ];
            self.m.tick(&i);
        }
    }
    fn push(&mut self, p: usize, s: &[u32]) {
        self.q[p].extend(s.iter().copied());
    }
}

#[test]
fn kfm_round() {
    let Some(d) = kfm("kfm", "kfm/kfm.def") else {
        return;
    };
    assert_eq!(d.name, "Kung Fu Man");
    assert!(d.anims.len() > 50 && d.states.len() > 30 && d.commands.len() > 20);
    let mut r = Run::new(d.clone(), d.clone());
    r.tick(120);
    assert_eq!(r.m.round_state, 2);
    assert_eq!(r.m.fighters[0].state_no, 0);
    r.push(0, &[R; 40]);
    let x0 = r.m.fighters[0].pos.x;
    r.tick(20);
    assert_eq!(r.m.fighters[0].state_no, 20);
    assert!(r.m.fighters[0].pos.x > x0 + 10.0);
    r.tick(25);
    r.push(0, &[X, 0, 0]);
    let life = r.m.fighters[1].life;
    let mut attacked = false;
    for _ in 0..40 {
        r.tick(1);
        if r.m.fighters[0].move_type == 'A' {
            attacked = true;
        }
    }
    assert!(attacked);
    assert!(r.m.fighters[1].life < life, "jab should hit");
    r.tick(60);
    r.push(0, &[U, U, U]);
    let mut miny = 0.0f32;
    for _ in 0..60 {
        r.tick(1);
        miny = miny.min(r.m.fighters[0].pos.y);
    }
    assert!(miny < -40.0);
    r.tick(30);
    assert_eq!(r.m.fighters[0].pos.y, 0.0);
    r.tick(60);
    r.push(0, &[D, D, D | R, D | R, R, R | X, 0]);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..60 {
        r.tick(1);
        seen.insert(r.m.fighters[0].state_no);
    }
    assert!(seen.contains(&1000), "palm: {seen:?}");
}

#[test]
fn kfm_throw_and_super() {
    let Some(d) = kfm("kfm", "kfm/kfm.def") else {
        return;
    };
    let mut r = Run::new(d.clone(), d.clone());
    r.tick(110);
    r.push(0, &[R; 60]);
    r.tick(60);
    r.push(0, &[R | Y, R]);
    let mut custom = false;
    for _ in 0..120 {
        r.tick(1);
        let p2 = &r.m.fighters[1];
        if Arc::ptr_eq(&p2.st_def, &r.m.fighters[0].def) && p2.state_no >= 800 {
            custom = true;
        }
    }
    assert!(custom, "throw custom state");
    assert!(r.m.fighters[1].life < 1000.0);

    let mut r = Run::new(d.clone(), d.clone());
    r.tick(110);
    r.m.fighters[0].power = 3000.0;
    r.push(0, &[R; 40]);
    r.tick(40);
    let q = [D, D, D | R, D | R, R, R];
    r.push(0, &q);
    r.push(0, &[0]);
    r.push(0, &q[..5]);
    r.push(0, &[R | X, 0]);
    let mut paused = false;
    for _ in 0..150 {
        r.tick(1);
        if r.m.pause.is_some() {
            paused = true;
        }
    }
    assert!(paused, "super pause");
    assert!(r.m.fighters[0].power < 3000.0);
}

#[test]
fn hd_scaling() {
    let (Some(a), Some(b)) = (
        kfm("kfm", "kfm/kfm.def"),
        kfm("kfm720", "kfm720/kfm720.def"),
    ) else {
        return;
    };
    assert_eq!(b.local_w, 1280.0);
    let mut r = Run::new(a, b);
    r.tick(110);
    let (ax, bx) = (r.m.fighters[0].pos.x, r.m.fighters[1].pos.x);
    r.push(0, &[R; 30]);
    r.push(1, &[L; 30]);
    r.tick(30);
    let (da, db) = (r.m.fighters[0].pos.x - ax, bx - r.m.fighters[1].pos.x);
    assert!((da - db).abs() < 15.0, "walk {da} {db}");
    r.push(1, &[U, U]);
    let mut miny = 0.0f32;
    for _ in 0..60 {
        r.tick(1);
        miny = miny.min(r.m.fighters[1].pos.y);
    }
    assert!(miny < -50.0 && miny > -120.0, "jump {miny}");
}

#[test]
fn ko_ends_round() {
    let Some(d) = kfm("kfm", "kfm/kfm.def") else {
        return;
    };
    let mut r = Run::new(d.clone(), d.clone());
    r.tick(110);
    r.m.fighters[1].life = 30.0;
    r.push(0, &[R; 60]);
    r.tick(50);
    for _ in 0..10 {
        r.push(0, &[D, D, D | R, D | R, R, R | X, 0]);
        r.tick(60);
        if r.m.round_state >= 3 {
            break;
        }
    }
    assert!(r.m.round_state >= 3, "KO");
    r.tick(600);
    assert_eq!(r.m.wins[0], 1);
    assert_eq!(r.m.round_no, 2);
}
