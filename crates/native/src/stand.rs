use super::abilities::AbilityCode;
use super::companions::StandSpec;
use super::compile::extras::{hitdef_from_spec, hitdef_projectile};
use super::multihit::{glue, projectile_knockback};
use super::types::NativeMove;
use ume_engine::chardef::MoveInfo;
use ume_engine::commands::parse_command;
use std::collections::HashMap;

const STAND_IDLE: i32 = 7000;
const STAND_HIT: i32 = 7001;
const STAND_DESPAWN: i32 = 7002;
const STAND_ATTACK_BASE: i32 = 7010;

fn move_from_parent<'a>(moves: &'a [NativeMove], id: &str) -> Option<&'a NativeMove> {
    moves.iter().find(|m| m.id == id)
}

pub(super) fn compile(
    spec: Option<&StandSpec>,
    anim_no: &HashMap<String, i32>,
    move_states: &HashMap<String, i32>,
    parent_moves: &[NativeMove],
) -> AbilityCode {
    let mut code = AbilityCode::default();
    let Some(spec) = spec else { return code };
    let hid = spec.id.unwrap_or(STAND_IDLE);
    let off = spec.offset.unwrap_or([38.0, -10.0]);
    let idle_anim = anim_no.get(&spec.idle_anim).copied().unwrap_or(0);
    let bind = spec.bind.unwrap_or(true);
    let hittable = spec.hittable.unwrap_or(true);
    let spr = spec.spr_priority.unwrap_or(3);
    let cost = spec.cost.unwrap_or(0.0).max(0.0);
    let drain = spec.drain.unwrap_or(0.0).max(0.0);
    let lt = spec.life_transfer.unwrap_or(0.0).clamp(0.0, 2.0);
    let helper_spawn = format!(
        "type = Helper\nhelpertype = normal\nid = {hid}\nstateno = {STAND_IDLE}\nkeyctrl = 1\nownpal = 1\npos = {}, {}\n",
        off[0], off[1]
    );
    if let Some(input) = &spec.input {
        let mut cmd = parse_command("stand_toggle", input, 20, 4);
        cmd.pause_buffer = 4;
        code.commands.push(cmd);
        code.movelist.push(MoveInfo {
            name: "Summon Stand".into(),
            input: input.clone(),
            command: "stand_toggle".into(),
            cost,
            drain: drain > 0.0,
            forms: u32::MAX,
            state: -1,
        });
        let power_gate = if cost > 0.0 {
            format!("triggerall = power >= {cost}\n")
        } else {
            String::new()
        };
        code.minus1 += &format!(
            "[State -1, stand_recall]\ntype = MapSet\nmap = \"__ume_stand_recall\"\nvalue = 1\ntriggerall = !IsHelper && RoundState = 2 && NumHelper({hid}) > 0\ntrigger1 = command = \"stand_toggle\"\n\n"
        );
        if cost > 0.0 {
            code.minus1 += &format!(
                "[State -1, stand_cost]\ntype = PowerAdd\nvalue = -{cost}\ntriggerall = !IsHelper && RoundState = 2 && NumHelper({hid}) = 0 && ctrl\n{power_gate}trigger1 = command = \"stand_toggle\"\n\n"
            );
        }
        code.minus1 += &format!(
            "[State -1, stand_summon]\n{helper_spawn}triggerall = !IsHelper && RoundState = 2 && NumHelper({hid}) = 0 && ctrl\n{power_gate}trigger1 = command = \"stand_toggle\"\n\n"
        );
    } else if spec.auto_summon {
        code.minus2 += &format!(
            "[State -2, stand_auto]\n{helper_spawn}trigger1 = !IsHelper && RoundState = 2 && Alive && NumHelper({hid}) = 0 && map(__ume_stand_recall) = 0\n\n"
        );
    }
    code.minus2 += &format!(
        "[State -2, stand_clear_recall]\ntype = MapSet\nmap = \"__ume_stand_recall\"\nvalue = 0\ntrigger1 = !IsHelper && NumHelper({hid}) = 0 && map(__ume_stand_recall) != 0\n\n"
    );
    if drain > 0.0 {
        code.minus2 += &format!(
            "[State -2, stand_drain]\ntype = PowerAdd\nvalue = {}\ntrigger1 = !IsHelper && RoundState = 2 && NumHelper({hid}) > 0\n\n[State -2, stand_empty]\ntype = MapSet\nmap = \"__ume_stand_recall\"\nvalue = 1\ntrigger1 = !IsHelper && RoundState = 2 && NumHelper({hid}) > 0 && Power < 1\n\n",
            -drain / 60.0
        );
    }
    let defense = if hittable {
        format!("[State {STAND_IDLE}, stand_hitovr]\ntype = HitOverride\nignorehitpause = 1\ntrigger1 = 1\nattr = SCA, AA, AT, AP\nstateno = {STAND_HIT}\ntime = 1\n\n")
    } else {
        format!("[State {STAND_IDLE}, stand_inv]\ntype = NotHitBy\nignorehitpause = 1\ntrigger1 = 1\nvalue = SCA\ntime = 1\n\n")
    };
    let follow = if bind {
        format!("[State {STAND_IDLE}, bind]\ntype = BindToParent\ntrigger1 = ParentExist\ntime = 1\npos = {}, {}\n\n", off[0], off[1])
    } else {
        format!(
            "[State {STAND_IDLE}, follow_x]\ntype = VelSet\ntrigger1 = ParentExist\nx = (Parent, Pos X + ({}) * Parent, Facing - Pos X) * Facing * 0.2\ny = (Parent, Pos Y + ({}) - Pos Y) * 0.2\n\n",
            off[0], off[1]
        )
    };
    let mut attack_transitions = String::new();
    let mut attack_states = String::new();
    for (idx, mv) in spec.moves.iter().enumerate() {
        let sid = STAND_ATTACK_BASE + idx as i32;
        let anim = anim_no.get(&mv.anim).copied().unwrap_or(idle_anim);
        let mcost = mv.cost.unwrap_or(0.0).max(0.0);
        let mut triggers = vec![];
        if let Some(inp) = &mv.input {
            let cname = format!("stand_m_{}", mv.id);
            let mut cmd = parse_command(&cname, inp, 20, 4);
            cmd.pause_buffer = 6;
            code.commands.push(cmd);
            code.movelist.push(MoveInfo {
                name: format!("Stand: {}", mv.name),
                input: inp.clone(),
                command: cname.clone(),
                cost: mcost,
                drain: false,
                forms: u32::MAX,
                state: -1,
            });
            triggers.push(format!("command = \"{cname}\""));
        }
        if let Some(pm) = &mv.trigger_move {
            let fallback_sid =
                move_from_parent(parent_moves, pm).and_then(|m| move_states.get(&m.id).copied());
            if let Some(psid) = move_states.get(pm).copied().or(fallback_sid) {
                triggers.push(format!(
                    "(ParentExist && Parent, StateNo = {psid} && Parent, Time = 1)"
                ));
            }
        }
        if !triggers.is_empty() {
            let cost_gate = if mcost > 0.0 {
                format!("triggerall = ParentExist && Parent, Power >= {mcost}\n")
            } else {
                String::new()
            };
            attack_transitions += &format!(
                "[State {STAND_IDLE}, atk_{idx}]\ntype = ChangeState\nvalue = {sid}\n{cost_gate}"
            );
            for (ti, tr) in triggers.iter().enumerate() {
                attack_transitions += &format!("trigger{} = {tr}\n", ti + 1);
            }
            attack_transitions += "\n";
        }
        attack_states += &format!(
            "[Statedef {sid}]\ntype = S\nmovetype = A\nphysics = N\nanim = {anim}\nctrl = 0\nvelset = 0, 0\nsprpriority = {spr}\n\n[State {sid}, sb]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n"
        );
        if mcost > 0.0 {
            attack_states += &format!(
                "[State {sid}, cost]\ntype = PowerAdd\ntrigger1 = Time = 0\nvalue = -{mcost}\n\n"
            );
        }
        if !mv.detach {
            attack_states += &format!("[State {sid}, bind]\ntype = BindToParent\ntrigger1 = ParentExist\ntime = 1\npos = {}, {}\n\n", off[0], off[1]);
        }
        attack_states +=
            &defense.replace(&format!("State {STAND_IDLE},"), &format!("State {sid},"));
        for v in &mv.moves {
            attack_states += &format!(
                "[State {sid}, vel]\ntype = VelSet\ntrigger1 = AnimElem = {}\n{}{}\n",
                v.at,
                v.vel[0].map(|x| format!("x = {x}\n")).unwrap_or_default(),
                v.vel[1].map(|y| format!("y = {y}\n")).unwrap_or_default()
            );
        }
        for h in glue(&mv.hits).iter() {
            attack_states += &format!(
                "[State {sid}, hit]\ntype = HitDef\ntrigger1 = AnimElem = {}\n{}\n",
                h.at,
                hitdef_from_spec(h, 1.0, "S, SA", false)
            );
        }
        for (pi, p) in mv
            .projectile
            .as_ref()
            .map(|p| p.to_vec())
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let pa = anim_no.get(&p.anim).copied().unwrap_or(0);
            let pid = sid * 10 + pi as i32;
            attack_states += &format!(
                "[State {sid}, proj_{pi}]\ntype = Projectile\ntrigger1 = AnimElem = {}\nprojid = {pid}\nprojanim = {pa}\noffset = {}, {}\nvelocity = {}, {}\nprojhits = {}\nprojmisstime = 6\nprojpriority = {}\nprojremovetime = {}\n{}\n",
                p.at, p.offset[0], p.offset[1], p.speed[0], p.speed[1],
                p.hits.unwrap_or(1), p.priority.unwrap_or(1), p.remove_time.unwrap_or(-1),
                hitdef_projectile(p.damage, p.level.as_deref(), projectile_knockback(p), p.launch.unwrap_or(false), "S, SP", p.element.as_deref(), p.stun)
            );
        }
        attack_states += &format!(
            "[State {sid}, end]\ntype = ChangeState\ntrigger1 = AnimTime = 0 || Time >= 90\nvalue = {STAND_IDLE}\n\n[State {sid}, bye]\ntype = ChangeState\ntrigger1 = !ParentExist || Parent, Alive = 0 || RoundState != 2 || Parent, map(__ume_stand_recall) != 0\nvalue = {STAND_DESPAWN}\n\n"
        );
    }
    let transfer = if lt > 0.0 {
        format!(
            "[State {STAND_HIT}, transfer]\ntype = ModifyPlayer\nignorehitpause = 1\npersistent = 0\ntrigger1 = Time = 0 && ParentExist\nid = Parent, ID\nlife = ifelse((Parent, Life) - ceil(GetHitVar(hitdamage) * {lt}) > 0, (Parent, Life) - ceil(GetHitVar(hitdamage) * {lt}), 0)\n\n"
        )
    } else {
        String::new()
    };
    let despawn_anim = spec
        .despawn_anim
        .as_ref()
        .and_then(|n| anim_no.get(n).copied())
        .unwrap_or(idle_anim);
    let despawn_cond = if spec.despawn_anim.is_some() {
        "AnimTime = 0 || Time >= 20"
    } else {
        "Time >= 1"
    };
    code.states += &format!(
        "[Statedef {STAND_IDLE}]\ntype = S\nmovetype = I\nphysics = N\nanim = {idle_anim}\nctrl = 0\nvelset = 0, 0\nsprpriority = {spr}\n\n[State {STAND_IDLE}, sb]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n[State {STAND_IDLE}, face]\ntype = Turn\ntrigger1 = ParentExist && Facing != Parent, Facing\n\n{follow}{defense}{attack_transitions}[State {STAND_IDLE}, bye]\ntype = ChangeState\ntrigger1 = !ParentExist || Parent, Alive = 0 || RoundState != 2 || Parent, map(__ume_stand_recall) != 0\nvalue = {STAND_DESPAWN}\n\n[Statedef {STAND_HIT}]\ntype = U\nmovetype = H\nphysics = N\nctrl = 0\nvelset = 0, 0\n\n[State {STAND_HIT}, sb]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n[State {STAND_HIT}, nhb]\ntype = NotHitBy\nignorehitpause = 1\ntrigger1 = 1\nvalue = SCA\ntime = 1\n\n{transfer}[State {STAND_HIT}, recover]\ntype = ChangeState\ntrigger1 = Time >= 10\nvalue = {STAND_IDLE}\n\n[Statedef {STAND_DESPAWN}]\ntype = S\nmovetype = I\nphysics = N\nanim = {despawn_anim}\nctrl = 0\nvelset = 0, 0\n\n[State {STAND_DESPAWN}, inv]\ntype = NotHitBy\ntrigger1 = 1\nvalue = SCA\ntime = 1\n\n[State {STAND_DESPAWN}, kill]\ntype = DestroySelf\ntrigger1 = {despawn_cond}\n\n{attack_states}"
    );
    code
}
