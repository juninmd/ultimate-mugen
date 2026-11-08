//! Headless scan for fighters that hover above the floor (pos.y < 0 with no vertical motion) during CPU-vs-CPU fights.
//! Usage: cargo run --release -p ume-engine --example hover_check -- <char_dir>...
//! HIT_ONLY=1 keeps only hovers while the fighter is in a hit state (move_type H).
use std::collections::BTreeMap;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_dir, find_char_defs, Vfs};

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
                let mut worst: BTreeMap<(i32, char, char, char), (i32, f32)> = BTreeMap::new();
                let mut hist: Vec<Vec<String>> = vec![vec![], vec![]];
                let mut last = [-1i32; 2];
                let mut printed = [false; 2];
                let verbose = std::env::var("TRACE").is_ok();
                let mut win = [0i32; 2];
                for _ in 0..60 * 90 {
                    let roots = m.root_indices();
                    let (ia, ib) = (a.poll(&m, roots[0]), b.poll(&m, roots[1]));
                    m.tick(&[ia, ib]);
                    for (k, &r) in roots.iter().enumerate() {
                        let f = &m.fighters[r];
                        if let Some((lo, hi)) = std::env::var("TICKS").ok().and_then(|v| {
                            v.split_once('-').and_then(|(a, b)| {
                                Some((a.parse::<i64>().ok()?, b.parse::<i64>().ok()?))
                            })
                        }) {
                            if k == 0 && m.game_time >= lo && m.game_time <= hi {
                                let o = &m.fighters[roots[1]];
                                println!("TICK t={} st={} type={}{} shake={} hp={} vy={:.2} y={:.1} st_time={} life={} | opp st={} anim={} hd={} mh={} ghv_yv={:.2} ghv_ya={:.2}", m.game_time, f.state_no, f.state_type, f.move_type, f.hit_shake, f.hit_pause, f.vel.y, f.pos.y, f.state_time, f.life, o.state_no, o.anim_no, o.hitdef_active, o.move_hit, f.ghv.yvel, f.ghv.yaccel);
                            }
                        }
                        if f.state_no != last[k] {
                            last[k] = f.state_no;
                            hist[k].push(format!("t={} st={} type={}{}{} y={:.0} vy={:.2} ghv.yv={:.2} yacc={:.2} shake={} anim={}", m.game_time, f.state_no, f.state_type, f.move_type, f.physics, f.pos.y, f.vel.y, f.ghv.yvel, f.ghv.yaccel, f.hit_shake, f.anim_no));
                            if hist[k].len() > 8 {
                                hist[k].remove(0);
                            }
                        }
                        let hover = f.pos.y < -2.0
                            && f.vel.y.abs() < 0.01
                            && f.bind.is_none()
                            && !f.pos_freeze
                            && f.hit_pause == 0
                            && f.hit_shake == 0;
                        run[k] = if hover { run[k] + 1 } else { 0 };
                        if win[k] > 0 {
                            win[k] -= 1;
                            let o = &m.fighters[roots[1 - k]];
                            println!("      [atk st={} anim={} pause={} hd={} mh={} t={}] t={} st={} y={:.1} vy={:.2} shake={} pause={} over={} anim={} elem={} sl={} life={}", o.state_no, o.anim_no, o.hit_pause, o.hitdef_active, o.move_hit, o.state_time, m.game_time, f.state_no, f.pos.y, f.vel.y, f.hit_shake, f.hit_pause, f.hit_over_time, f.anim_no, f.elem, f.slide_time, f.life);
                        }
                        if run[k] == 1 && verbose && win[k] == 0 && !printed[k] {
                            win[k] = 40;
                        }
                        if run[k] == 21 && verbose && !printed[k] {
                            printed[k] = true;
                            println!("  -- P{k} hover begins, recent transitions:");
                            for h in &hist[k] {
                                println!("     {h}");
                            }
                        }
                        if run[k] > 20
                            && (std::env::var_os("HIT_ONLY").is_none() || f.move_type == 'H')
                        {
                            let e = worst
                                .entry((f.state_no, f.state_type, f.move_type, f.physics))
                                .or_insert((0, 0.0));
                            e.0 = e.0.max(run[k]);
                            e.1 = e.1.min(f.pos.y);
                        }
                    }
                    if m.match_over {
                        break;
                    }
                }
                worst
            });
            match res {
                Ok(w) if w.is_empty() => println!("OK   {def}"),
                Ok(w) => {
                    println!("HOVER {def}");
                    for ((st, t, mt, ph), (n, y)) in w.iter().take(6) {
                        println!("    state {st} type={t} move={mt} phys={ph} ticks={n} y={y:.0}");
                    }
                }
                Err(_) => println!("PANIC {def}"),
            }
        }
    }
}
