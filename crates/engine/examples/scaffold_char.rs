//! Scaffolding tool for generating a complete M.U.G.E.N character skeleton in Rust.
//! Usage: cargo run --release -p ume-engine --example scaffold_char -- <char_name> [target_dir]
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name = std::env::args().nth(1).unwrap_or_else(|| "MeuLutador".to_string());
    let target = std::env::args().nth(2).unwrap_or_else(|| format!("mugen/chars/{}", name));
    let base = Path::new(&target);

    fs::create_dir_all(base.join("files"))?;
    fs::create_dir_all(base.join("states"))?;
    fs::create_dir_all(base.join("palettes"))?;

    let def_content = format!(
        "[Info]\nname = \"{0}\"\ndisplayname = \"{0}\"\nversiondate = 01,01,2026\nmugenversion = 1.0\nauthor = \"Creator\"\npal.defaults = 1,2,3,4,5,6\nlocalcoord = 1280,720\n\n[Files]\nsprite   = files/sprites.sff\nanim     = files/animacoes.air\nsound    = files/sons.snd\ncmd      = files/comandos.cmd\ncns      = states/constants.cns\nstcommon = common1.cns\nst       = states/system.cns\nst1      = states/normals.cns\nst2      = states/specials.cns\nst3      = states/supers.cns\nst4      = states/helpers.cns\nst5      = states/state-2-3.cns\nst6      = config.txt\n\n[Palette Keymap]\nx = 1\ny = 2\nz = 3\na = 4\nb = 5\nc = 6\n",
        name
    );
    fs::write(base.join(format!("{}.def", name)), def_content)?;

    let constants = r#"[Data]
life = 1000
power = 3000
attack = 100
defence = 100
fall.defence_up = 50
liedown.time = 60
airjuggle = 15
sparkno = 2
guard.sparkno = 40
KO.echo = 0
volume = 0
IntPersistIndex = 50
FloatPersistIndex = 30

[Size]
xscale = 1
yscale = 1
ground.back = 50
ground.front = 50
air.back = 45
air.front = 45
height = 240
attack.dist = 640
proj.attack.dist = 360
proj.doscale = 0
head.pos = -10, -290
mid.pos = -15, -170
shadowoffset = 0
draw.offset = 0, 0

[Velocity]
walk.fwd  = 9.5
walk.back = -8.5
run.fwd  = 18.0, 0
run.back = -17.5, -14.0
jump.neu = 0, -33.0
jump.back = -10.0, -33.0
jump.fwd = 10.5, -33.0
runjump.back = -11.0, -32.0
runjump.fwd = 16.0, -32.0
airjump.neu = 0, -30.0
airjump.back = -9.0, -30.0
airjump.fwd = 9.5, -30.0

[Movement]
airjump.num = 1
airjump.height = 140
yaccel = 1.76
stand.friction = 0.85
crouch.friction = 0.82
stand.friction.threshold = 8
crouch.friction.threshold = 8
"#;
    fs::write(base.join("states/constants.cns"), constants)?;

    let system = r#"[Statedef 5900]
type = S

[State 5900, Reset Ints]
type = VarRangeSet
trigger1 = !roundsexisted
value = 0

[State 5900, Reset Floats]
type = VarRangeSet
trigger1 = !roundsexisted
fvalue = 0.0

[State 5900, Intro]
type = ChangeState
trigger1 = 1
value = 190
"#;
    fs::write(base.join("states/system.cns"), system)?;

    let cmd = r#"[Command]
name = "Hadouken"
command = ~D, DF, F, x
time = 15

[Command]
name = "x"
command = x
time = 1

[Command]
name = "y"
command = y
time = 1

[Statedef -1]
[State -1, Hadouken]
type = ChangeState
value = 1000
triggerall = command = "Hadouken" && statetype != A && ctrl
trigger1 = 1

[State -1, Jab]
type = ChangeState
value = 200
triggerall = command = "x" && statetype = S && ctrl
trigger1 = 1
"#;
    fs::write(base.join("files/comandos.cmd"), cmd)?;
    fs::write(base.join("config.txt"), "; Character personalization config\n")?;

    println!("Scaffolded character '{}' successfully at {:?}", name, base);
    Ok(())
}
