use std::sync::Arc;

use crate::support::duel;
use ume_engine::ai::Ai;
use ume_engine::chardef::CharDef;
use ume_engine::input::{k, InputSource};
use ume_engine::matchs::{Match, ARENA_P1_SPAWN_X, ARENA_P2_SPAWN_X, ENGINE_STATE_ARENA_EXIT};
use ume_native::{compile_native, NativeChar};

const DEFAULT_CPU_LEVEL: i32 = 4;
const DEFAULT_CPU_SEED: u32 = 99;
const QA_CPU_LEVEL: i32 = 1;

fn native(id: &str) -> Arc<CharDef> {
    let path = format!(
        "{}/../../assets/chars/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    Arc::new(compile_native(&nc, 1, false))
}

fn arena(level: i32, seed: u32) -> (Match, Ai, [usize; 2]) {
    let mut battle = duel(native("kaito"), native("shion"));
    battle.enable_power_arena();
    battle.enable_swarm(1);
    let roots = battle.root_indices();
    (battle, Ai::new(level, seed), [roots[0], roots[1]])
}

fn on_floor(battle: &Match, index: usize) -> bool {
    let f = &battle.fighters[index];
    f.state_no == 5110 && f.ground_level > -1.0 && f.pos.y >= -1.0
}

#[test]
fn holding_left_and_up_reaches_the_first_platform_without_a_floor_knockdown() {
    let (mut battle, mut cpu, roots) = arena(DEFAULT_CPU_LEVEL, DEFAULT_CPU_SEED);
    assert!((battle.fighters[roots[0]].pos.x - ARENA_P1_SPAWN_X).abs() < 1.0);
    assert!((battle.fighters[roots[1]].pos.x - ARENA_P2_SPAWN_X).abs() < 1.0);
    let mut landed = false;
    for _ in 0..360 {
        let foe = cpu.poll(&battle, roots[1]);
        battle.tick(&[k::L | k::U, foe]);
        let p = &battle.fighters[roots[0]];
        assert!(
            !on_floor(&battle, roots[0]),
            "downed on the floor at x={:.1} y={:.1} state={}",
            p.pos.x,
            p.pos.y,
            p.state_no
        );
        if p.ground_level <= -50.0 && (p.pos.y - p.ground_level).abs() <= 0.5 {
            landed = true;
            break;
        }
    }
    let p = &battle.fighters[roots[0]];
    assert!(
        landed,
        "still on the floor at x={:.1} y={:.1} ground={:.1} state={}",
        p.pos.x, p.pos.y, p.ground_level, p.state_no
    );
    assert_ne!(p.state_no, 5110);

    let life = p.life;
    let power = battle.fighters[roots[1]].power;
    let mut fought = false;
    for _ in 0..240 {
        let foe = cpu.poll(&battle, roots[1]);
        battle.tick(&[0, foe]);
        let cpu_f = &battle.fighters[roots[1]];
        if cpu_f.move_type == 'A'
            || cpu_f.state_no == 3920
            || cpu_f.power > power + 1.0
            || battle.fighters[roots[0]].life < life
        {
            fought = true;
            break;
        }
    }
    assert!(
        fought,
        "cpu stayed passive after the landing, state {} power {}",
        battle.fighters[roots[1]].state_no, battle.fighters[roots[1]].power
    );
}

#[test]
fn scripted_qa_climb_rings_out_against_the_loading_cpu() {
    let (mut battle, mut cpu, roots) = arena(QA_CPU_LEVEL, DEFAULT_CPU_SEED);
    let mut phase = 0u8;
    let mut up = 0i32;
    for _ in 0..20 * 60 {
        let foe = cpu.poll(&battle, roots[1]);
        let bits = match phase {
            0 | 5 | 6 => k::L,
            1 | 3 => k::U,
            _ => 0,
        };
        battle.tick(&[bits, foe]);
        let p = &battle.fighters[roots[0]];
        assert!(
            !on_floor(&battle, roots[0]),
            "phase {phase} downed on the floor at x={:.1} state={}",
            p.pos.x,
            p.state_no
        );
        match phase {
            0 if p.pos.x < -218.0 => phase = 1,
            1 => {
                up += 1;
                if up >= 8 {
                    phase = 2;
                    up = 0;
                }
            }
            2 if p.state_no == 0
                && p.ground_level <= -50.0
                && (p.pos.y - p.ground_level).abs() <= 0.5 =>
            {
                phase = 3
            }
            3 => {
                up += 1;
                if up >= 8 {
                    phase = 4;
                }
            }
            4 if p.state_no == 0
                && p.ground_level <= -102.0
                && (p.pos.y - p.ground_level).abs() <= 0.5 =>
            {
                phase = 5
            }
            5 if p.pos.x < -510.0 && battle.round_state == 2 => phase = 6,
            6 if p.state_no == ENGINE_STATE_ARENA_EXIT || battle.winner_side == 1 => phase = 7,
            _ => {}
        }
        if phase == 7 && battle.winner_side == 1 && !battle.fighters[roots[0]].alive {
            break;
        }
    }
    let p = &battle.fighters[roots[0]];
    assert_eq!(
        phase, 7,
        "stopped at phase {phase} x={:.1} y={:.1} ground={:.1} state={} winner={}",
        p.pos.x, p.pos.y, p.ground_level, p.state_no, battle.winner_side
    );
    assert_eq!(battle.winner_side, 1);
}
