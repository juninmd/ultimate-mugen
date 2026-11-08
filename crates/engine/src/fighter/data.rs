use crate::palette::AfterImageParams;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 { pub x: f32, pub y: f32 }

#[derive(Clone, Debug, Default)]
pub struct HitDefData {
    pub attr_state: String, pub attr_kind: Vec<String>, pub hitflag: String, pub guardflag: String,
    pub animtype: i32, pub air_animtype: i32, pub fall_animtype: i32, pub priority: i32,
    pub damage: f32, pub guard_damage: f32, pub pause_p1: i32, pub pause_p2: i32, pub guard_pause_p1: i32, pub guard_pause_p2: i32,
    pub spark_no: i32, pub spark_own: bool, pub guard_spark_no: i32, pub guard_spark_own: bool, pub spark_x: f32, pub spark_y: f32,
    pub hit_sound: Option<(i32, i32, bool)>, pub guard_sound: Option<(i32, i32, bool)>,
    pub ground_type: i32, pub air_type: i32, pub ground_slide_time: i32, pub guard_slide_time: i32,
    pub ground_hit_time: i32, pub guard_hit_time: i32, pub air_hit_time: i32, pub guard_ctrl_time: i32, pub guard_dist: f32,
    pub yaccel: f32, pub ground_vel: [f32; 2], pub guard_vel: f32, pub air_vel: [f32; 2], pub air_guard_vel: [f32; 2],
    pub p1sprpriority: i32, pub p2sprpriority: i32, pub p1facing: i32, pub p2facing: i32,
    pub p1stateno: i32, pub p2stateno: i32, pub p2getp1state: bool, pub forcestand: bool,
    pub fall: bool, pub air_fall: bool, pub fall_xvel: Option<f32>, pub fall_yvel: f32, pub fall_recover: bool, pub fall_recover_time: i32,
    pub fall_damage: f32, pub fall_kill: bool, pub kill: bool, pub guard_kill: bool,
    pub get_power: f32, pub give_power: f32, pub guard_get_power: f32, pub guard_give_power: f32,
    pub id: i32, pub chain_id: Option<i32>, pub envshake_time: i32, pub num_hits: i32,
    /** juggle points this move costs on an airborne, falling target (its Statedef `juggle`) and the attacker's `data.airjuggle` budget */
    pub juggle: i32, pub airjuggle: i32,
    pub no_chain_id: [Option<i32>; 2], pub hit_once: bool, pub snap: Option<V2>, pub min_dist: [Option<f32>; 2], pub max_dist: [Option<f32>; 2], pub affect_team: char,
    pub down_vel: [f32; 2], pub down_hit_time: i32, pub down_bounce: bool, pub fall_env_shake: Shake, pub env_shake: Shake,
    pub ground_corner_push: f32, pub air_corner_push: f32, pub guard_corner_push: f32, pub air_guard_corner_push: f32,
    pub target_palfx: Option<PalFx>,
    pub dizzy_points: f32, pub guard_points: f32, pub red_life: [f32; 2], pub score: [f32; 2], pub down_recover: bool,
    pub miss_on_override: bool, pub unhittable_time: i32, pub spark_scale: f32, pub xaccel: f32, pub attack_depth: f32, pub p2clsncheck: bool, pub p2clsnrequire: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shake { pub time: i32, pub freq: f32, pub ampl: f32, pub phase: f32, pub dir: u8 }

#[derive(Clone, Debug)]
pub struct HitOverride { pub attr: String, pub state_no: i32, pub time: i32, pub force_air: bool, pub keep: bool, pub hits: i32, pub threshold: f32, pub chip: f32 }

#[derive(Clone, Debug)]
pub struct GetHitVars {
    pub kind: i32, pub animtype: i32, pub airtype: i32, pub groundtype: i32, pub damage: f32, pub hitcount: i32, pub guardcount: i32,
    pub hittime: i32, pub slidetime: i32, pub ctrltime: i32, pub xvel: f32, pub yvel: f32, pub yaccel: f32, pub xaccel: f32, pub chainid: i32,
    pub guarded: bool, pub fall: bool, pub fall_damage: f32, pub fall_xvel: f32, pub fall_yvel: f32, pub fall_recover: bool,
    pub fall_recover_time: i32, pub fall_kill: bool, pub kill: bool, pub attacker_id: u32, pub attacker_no: usize,
    pub hit_id: i32, pub fall_env_shake: Shake, pub down_bounce: bool,
    pub priority: i32, pub facing: i32, pub power: f32, pub attr: i32, pub guardflag: i32,
    pub hit_damage: f32, pub guard_damage: f32, pub hit_power: f32, pub guard_power: f32,
    pub dizzy_points: f32, pub guard_points: f32, pub red_life: f32, pub score: f32,
    pub team_side: i32, pub proj_id: i32, pub p1stateno: i32, pub p2stateno: i32, pub down_recover: bool,
}
impl GetHitVars {
    pub fn record_hitdef(&mut self, hd: &HitDefData, guarded: bool, attacker: (u32, usize, usize), attacker_facing: f32, proj_id: Option<i32>) {
        let side = guarded as usize;
        self.attacker_id = attacker.0; self.team_side = attacker.1 as i32 + 1; self.attacker_no = attacker.2;
        self.priority = hd.priority; self.facing = if attacker_facing < 0.0 { -1 } else { 1 };
        self.power = if guarded { hd.guard_give_power } else { hd.give_power };
        self.attr = crate::hitdef::attr_bits(&hd.attr_state, &hd.attr_kind); self.guardflag = crate::hitdef::guardflag_bits(&hd.guardflag);
        self.hit_damage = hd.damage; self.guard_damage = hd.guard_damage; self.hit_power = hd.get_power; self.guard_power = hd.guard_get_power;
        self.dizzy_points = hd.dizzy_points; self.guard_points = hd.guard_points; self.red_life = hd.red_life[side]; self.score = hd.score[side];
        self.proj_id = proj_id.unwrap_or(-1);
        self.p1stateno = hd.p1stateno; self.p2stateno = hd.p2stateno; self.down_recover = hd.down_recover;
    }
}
impl Default for GetHitVars {
    fn default() -> Self {
        GetHitVars { kind: 0, animtype: 0, airtype: 0, groundtype: 0, damage: 0.0, hitcount: 0, guardcount: 0, hittime: 0, slidetime: 0, ctrltime: 0,
            xvel: 0.0, yvel: 0.0, yaccel: 0.44, xaccel: 0.0, chainid: -1, guarded: false, fall: false, fall_damage: 0.0, fall_xvel: 0.0, fall_yvel: -4.5,
            fall_recover: true, fall_recover_time: 4, fall_kill: true, kill: true, attacker_id: 0, attacker_no: 0,
            hit_id: -1, fall_env_shake: Shake::default(), down_bounce: false,
            priority: 0, facing: 0, power: 0.0, attr: 0, guardflag: 0, hit_damage: 0.0, guard_damage: 0.0, hit_power: 0.0, guard_power: 0.0,
            dizzy_points: 0.0, guard_points: 0.0, red_life: 0.0, score: 0.0, team_side: 0, proj_id: -1, p1stateno: -1, p2stateno: -1, down_recover: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BindKind { Target, Parent, Root }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bind { pub to: u32, pub time: i32, pub x: f32, pub y: f32, pub kind: BindKind }

pub use crate::palette::PalFx;

#[derive(Clone, Debug)]
pub struct TrailPoint { pub x: f32, pub y: f32, pub group: i32, pub index: i32, pub fx: f32, pub fy: f32, pub flip_h: bool, pub facing: f32 }
#[derive(Clone, Debug)]
pub struct AfterImage { pub time: i32, pub length: usize, pub trail: Vec<TrailPoint>, pub params: AfterImageParams, pub ticks: i32 }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveHitVars { pub target_id: i32, pub target_no: i32, pub spark_x: f32, pub spark_y: f32, pub corner_push: f32, pub hit_time: i64, pub overridden: bool }