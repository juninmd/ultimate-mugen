//! Headless scan for fighters that lose control for too long (stuck states) during CPU-vs-CPU fights.
//! Usage: cargo run --release -p ume-engine --example stuck_check -- <char_dir>...
use std::collections::BTreeMap;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

const LIMIT: i32 = 420;

fn main() {
    for dir in std::env::args().skip(1) {
        let mut vfs = Vfs::new();
        if add_dir(&mut vfs, std::path::Path::new(&dir), "c").is_err() {
            continue;
        }
        for def in find_char_defs(&vfs) {
            let Ok(d) = load_mugen_char(&vfs, &def, 1, true) else {
                continue;
            };
            let d = Arc::new(d);
            let res = std::panic::catch_unwind(|| {
                let mut m = Match::new(common_def(), None, flat_stage());
                m.add_player(d.clone(), 0, 6, 1);
                m.add_player(d.clone(), 1, 6, 2);
                m.start_round();
                let (mut a, mut b) = (Ai::new(6, 7), Ai::new(6, 13));
                let mut run = [0i32; 2];
                let mut win = [0i32; 2];
                let verbose = std::env::var("TRACE").is_ok();
                let mut found: BTreeMap<(i32, i32, char, char), (i32, i32)> = BTreeMap::new();
                for _ in 0..60 * 120 {
                    let roots = m.root_indices();
                    let (ia, ib) = (a.poll(&m, roots[0]), b.poll(&m, roots[1]));
                    m.tick(&[ia, ib]);
                    for (k, &r) in roots.iter().enumerate() {
                        let f = &m.fighters[r];
                        let stuck = m.round_state == 2
                            && f.alive
                            && !f.ctrl
                            && f.hit_pause == 0
                            && f.hit_shake == 0
                            && m.pause.is_none()
                            && f.bind.is_none();
                        run[k] = if stuck { run[k] + 1 } else { 0 };
                        if verbose
                            && (f.state_no == 5050
                                || f.state_no == 5101
                                || f.state_no == 5110
                                || f.state_no == 5201)
                            && run[k] == 40
                        {
                            win[k] = 14;
                        }
                        if win[k] > 0 {
                            win[k] -= 1;
                            println!("      t={} P{k} st={} {}{}{} y={:.1} x={:.1} vx={:.2} vy={:.2} anim={} elem={} stime={} fall={} slide={} hover={} life={:.0}", m.game_time, f.state_no, f.state_type, f.move_type, f.physics, f.pos.y, f.pos.x, f.vel.x, f.vel.y, f.anim_no, f.elem, f.state_time, f.ghv.fall, f.slide_time, f.hit_over_time, f.life);
                        }
                        if run[k] > LIMIT {
                            let e = found
                                .entry((f.state_no, f.anim_no, f.state_type, f.move_type))
                                .or_insert((0, f.state_time));
                            e.0 = e.0.max(run[k]);
                            e.1 = f.state_time;
                        }
                    }
                    if m.match_over {
                        break;
                    }
                }
                found
            });
            match res {
                Ok(w) if w.is_empty() => println!("OK   {def}"),
                Ok(w) => {
                    println!("STUCK {def}");
                    for ((st, an, t, mt), (n, stime)) in w.iter().take(5) {
                        println!("    state {st} anim {an} type={t} move={mt} no-ctrl ticks={n} state_time={stime}");
                    }
                }
                Err(_) => println!("PANIC {def}"),
            }
        }
    }
}
