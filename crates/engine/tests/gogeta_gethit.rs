mod support;
use std::collections::BTreeSet;
use support::gogeta::*;
use support::*;

fn attacker(hitdef: &str) -> std::sync::Arc<ume_engine::chardef::CharDef> {
    let cns = format!(
        "[Statedef 0]\ntype=S\nphysics=S\nctrl=1\n\n[Statedef 200]\ntype=S\nmovetype=A\nphysics=S\nanim=200\n\n[State 200,hd]\ntype=HitDef\ntrigger1=AnimElem=2\n{hitdef}\n\n[State 200,end]\ntype=ChangeState\ntrigger1=AnimTime=0\nvalue=0\nctrl=1\n"
    );
    mugen_char("atk", &cns)
}

const VARIANTS: &[(&str, &str)] = &[
    ("stand-light", "attr=S,NA\nanimtype=Light\nhitflag=MAF\nguardflag=MA\ndamage=20,0\nground.type=High\nground.hittime=12\nair.hittime=15\nair.velocity=-2,-3\nground.velocity=-3\npausetime=8,8\nsparkno=0"),
    ("stand-low", "attr=S,NA\nanimtype=Light\nhitflag=MAF\nground.type=Low\ndamage=25\nground.hittime=14\nground.velocity=-3"),
    ("stand-trip", "attr=S,NA\nanimtype=Back\nhitflag=MAF\nground.type=Trip\ndamage=30\nground.velocity=-4,-2"),
    ("stand-heavy-fall", "attr=S,NA\nanimtype=Up\nhitflag=MAF\nground.type=High\ndamage=60\nfall=1\nground.velocity=-5,-6\nair.velocity=-4,-6\nfall.recover=1\nfall.recovertime=15\nground.hittime=20"),
    ("knockaway-norecover", "attr=S,HA\nanimtype=Hard\nhitflag=MAF\ndamage=80\nfall=1\nfall.recover=0\nground.velocity=-6,-8\nair.velocity=-6,-8\nforcenofall=0"),
    ("air-juggle", "attr=S,NA\nanimtype=Light\nhitflag=MAF\ndamage=20\nair.type=High\nair.velocity=-2,-4\nair.hittime=20\nair.fall=1\nfall=1\nground.velocity=-2,-4"),
    ("sweep-down", "attr=S,NA\nanimtype=Back\nhitflag=MAFD\ndamage=15\nground.type=Low\nground.velocity=-3\nfall=1\nground.velocity=-4,-5"),
    ("tiny-hittime", "attr=S,NA\nanimtype=Light\nhitflag=MAF\ndamage=5\nground.hittime=1\nair.hittime=1\nground.velocity=0\npausetime=0,0"),
    ("long-pause", "attr=S,NA\nanimtype=Light\nhitflag=MAF\ndamage=5\nground.hittime=30\npausetime=40,40\nground.velocity=-2"),
    ("super-like", "attr=S,HA\nanimtype=Heavy\nhitflag=MAFD\ndamage=300\nground.type=High\nfall=1\nground.velocity=-8,-12\nair.velocity=-8,-12\nfall.recover=1\nfall.recovertime=40\nhitonce=1\npausetime=0,20\nground.hittime=40"),
    ("guard", "attr=S,NA\nanimtype=Light\nhitflag=MAF\nguardflag=MA\ndamage=20,5\nguard.hittime=12\nguard.velocity=-4\nguard.pausetime=8,8\nground.hittime=12"),
];

#[derive(Clone, Copy, Debug)]
enum Posture {
    Stand,
    Crouch,
    Air,
    Guarding,
    Down,
    Recover,
    GuardCrouch,
}

fn trial(
    d: &std::sync::Arc<ume_engine::chardef::CharDef>,
    hit: &str,
    post: Posture,
    life: f32,
) -> (bool, Vec<i32>, BTreeSet<i32>, (i32, bool, bool, f32)) {
    let atk = attacker(hit);
    let mut m = duel(atk, d.clone());
    face_off(&mut m, 34.0);
    m.fighters[1].life = life;
    m.fighters[1].life_max = life.max(1000.0);
    let mut hold = 0u32;
    match post {
        Posture::Crouch => {
            hold = 2;
        }
        Posture::Air => {
            m.fighters[1].pos.y = -25.0;
            m.fighters[1].vel.y = -1.0;
            let dd = d.clone();
            m.change_state(1, 50, dd);
        }
        Posture::Guarding => {
            hold = 8;
        }
        Posture::GuardCrouch => {
            hold = 8 | 2;
        }
        Posture::Recover => {}
        Posture::Down => {
            let dd = d.clone();
            m.change_state(1, 5110, dd);
            m.fighters[1].ctrl = false;
        }
        Posture::Stand => {}
    }
    if matches!(post, Posture::Crouch | Posture::GuardCrouch) {
        for _ in 0..30 {
            m.tick(&[0, hold]);
        }
    }
    enter(&mut m, 0, 200);
    let mut seen = BTreeSet::new();
    let mut hit_seen = false;
    let mut order: Vec<i32> = vec![];
    for t in 0..900 {
        let h = if t < 300 { hold } else { 0 };
        let h = if matches!(post, Posture::Recover) && (4..260).contains(&t) && (t / 3) % 2 == 0 {
            128 | 256
        } else {
            h
        };
        m.tick(&[0, h]);
        let f = &m.fighters[1];
        if f.move_type == 'H' {
            hit_seen = true;
        }
        if order.last() != Some(&f.state_no) {
            order.push(f.state_no);
        }
        seen.insert(f.state_no);
    }
    let f = &m.fighters[1];
    (hit_seen, order, seen, (f.state_no, f.ctrl, f.alive, f.life))
}

#[test]
fn gethit_matrix() {
    let Some(d) = load(1) else { return };
    let mut stuck = 0;
    let mut reached = BTreeSet::new();
    for (name, hit) in VARIANTS {
        for post in [
            Posture::Stand,
            Posture::Crouch,
            Posture::Air,
            Posture::Guarding,
            Posture::GuardCrouch,
            Posture::Down,
            Posture::Recover,
        ] {
            for life in [100000.0, 40.0] {
                let (hit_seen, order, seen, (st, ctrl, alive, hp)) = trial(&d, hit, post, life);
                reached.extend(seen.iter().copied());
                let ok = if st == 5900 {
                    true
                } else if alive {
                    ctrl && st == 0 || (ctrl && matches!(post, Posture::Air) && false)
                } else {
                    st == 5150 || st == 5110 || !ctrl
                };
                println!("{} {name:<20} {post:?} life={life:.0} hit={hit_seen} end=({st},{ctrl},{alive}) hp={hp:.0} order={order:?}", if ok { "OK   " } else { "STUCK" });
                if !ok {
                    stuck += 1;
                }
            }
        }
    }
    println!("REACHED {:?}", reached);
    println!("stuck cases: {stuck}");
    assert_eq!(stuck, 0);
}
