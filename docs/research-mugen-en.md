# MUGEN / Ikemen GO research report (for "ultimate-mugen")

Date: 2026-11-02. Sources: Elecbyte M.U.G.E.N 1.0/1.1 docs (elecbyte.com/mugendocs was not reachable
from the sandbox, so the format details below come from those docs as I know them, cross-checked against the
open-source readers listed), the Ikemen GO GitHub repo, wiki and releases, the SFF reader libraries
(openkakutou/sff for Go, kumohakasemk9/MUGENSFFReaderLibJava), roundonejs_mugen_renderer, MUGEN Database (fandom),
and github.com/morluto. Treat byte layouts as "verify against a reference decoder".

---------------------------------------------------------------------------------------------------
## 1. How M.U.G.E.N works (Elecbyte, 1999 to 2013; 1.0 = 2009, 1.1 beta = 2013, then abandoned)

### 1.1 Character package (`chars/<name>/<name>.def`)
INI-like text. Comments start with `;`. Keys are case-insensitive.
- `[Info]`: name, displayname, versiondate, mugenversion (`1.0`/`1.1` changes defaults), author,
  pal.defaults, localcoord (1.0+: e.g. `640,480` for HD characters; the default is 320x240).
- `[Files]`: cmd, cns, st/st0..stN (extra state files), stcommon (`common1.cns`), sprite (.sff),
  anim (.air), sound (.snd), ai (unused), pal1..pal12 (.act palettes for SFFv1 and v2 "shared" palettes).
- `[Arcade]`: intro.storyboard and ending.storyboard (storyboard .def files).

### 1.2 SFF sprite archive
- **SFF v1** (WinMUGEN/1.0 compatible): signature `ElecbyteSpr\0`, version 1.01. The header holds the image count and
  the offset of the first subfile. Each subfile header (32 bytes) holds the next subfile offset, the length, axis x/y, group,
  image number, a "linked index" (reuse earlier image data when length=0), a "same palette as previous" flag and a comment.
  The payload is a **PCX file** (8-bit RLE PCX) with a trailing 768-byte palette. The "shared palette" means
  sprites take the character palette (.act) so that pal1..12 swaps work.
- **SFF v2** (1.0 = v2.00, 1.1 = v2.01): a header, then a **sprite node table** (group u16, number u16, w/h u16,
  axis x/y s16, linked index u16, **format u8**, color depth u8, data offset/len u32, palette index u16,
  flags u16 where bit0 selects the *ldata* (load into memory) or *tdata* (translated) block). Then a **palette node table**
  (group, item, numcols, linked, offset, len; colors stored as RGBA quads) and the ldata/tdata blobs.
  Every compressed payload starts with a u32 uncompressed size.
  - Format codes: `0` raw, `1` linked/invalid, `2` **RLE8**, `3` **RLE5**, `4` **LZ5**,
    `10` **PNG8**, `11` PNG24, `12` PNG32 (PNG appears only in v2.01/1.1).
  - RLE8: if `(b & 0xC0) == 0x40`, then `b & 0x3F` is the run length and the next byte is the color index; otherwise
    it is a literal pixel.
  - RLE5: lossless for 5-bit (32-color) sprites. Packets are a run-length byte plus a byte whose top bit flags "color
    present", followed by 3-bit run / 5-bit color pairs. Several community decoders skipped it (the Java lib) or
    never tested it against real files (the Go lib).
  - LZ5: a control byte gives 8 flags (RLE or LZ packet). It has short and long RLE packets and short and long LZ copies,
    with "recycled" bits accumulated across short LZ packets. It decompresses quickly and is the most common format in
    1.0-era characters.
  - Group 9000 is reserved for portraits (9000,0 small and 9000,1 large on the select screen). Sprites carry an axis
    (pivot) used for flipping and positioning.
- Reference decoders to port: `pkg.go.dev/github.com/openkakutou/sff` (Go, raw/RLE8/LZ5/PNG, RLE5 untested),
  Ikemen GO `src/image.go` (production-grade, MIT), `roundonejs_mugen_renderer` (TypeScript, browser).

### 1.3 AIR animations
```
[Begin Action 200]            ; action number = anim id
Clsn2Default: 2               ; hurtboxes reused by every frame until overridden
 Clsn2[0] = -10,-80, 12, 0
 Clsn2[1] = ...
Clsn1: 1                      ; attack box(es), this frame only
 Clsn1[0] = 15,-70, 55,-55
200,0, 0,0, 3                 ; group,image, xoff,yoff, ticks [,flip H|V|HV] [,trans A|A1|S|AS256D128]
200,1, 0,0, 4, H, A
Loopstart                     ; loop point; ticks = -1 means hold forever
```
**Clsn1 = hitbox (attack), Clsn2 = hurtbox.** The boxes are axis-aligned rectangles relative to the sprite axis
and are mirrored with facing. 1.1 adds per-frame scale/angle (interpolation) fields.

### 1.4 CMD input commands
- `[Remap]` maps buttons. `[Defaults]` sets `command.time = 15` (default window, in ticks) and
  `command.buffer.time = 1` (how many ticks a completed command stays "true", 1 to 30).
- `[Command]` takes `name`, `command`, `time`, `buffer.time`. Several blocks may share a name (alternatives).
- Symbols: directions `B DB D DF F UF U UB`; buttons `a b c x y z s` (1.1 adds `d w`).
  - `~` = on **release** (`~30a` means held 30+ ticks, then released: charge moves). `~D` alone = release down.
  - `/` = **held** (`/F,a`). `$` = direction **inclusive** (`$D` matches D, DB or DF). `>` = **no other input**
    between this and the previous element (strict). `+` = **simultaneous** (`a+b`), and `,` separates sequence steps.
  - Example: `command = ~D, DF, F, x` (QCF+x); `command = ~$B, $F, x` style for charge moves.
- The CMD file also contains **`[Statedef -1]`**, which runs every tick while the player has control. Its ChangeState blocks with
  `trigger1 = command = "QCF_x"` and `trigger1 = statetype = S` map inputs to moves. Command priority = order of appearance.
- The engine keeps a per-player input history buffer. Facing is resolved per tick, so F/B flip with P1's facing.

### 1.5 CNS: constants, states, triggers, controllers
- Constants: `[Data]` (life=1000, power=3000, attack, defence, fall.defence_up, liedown.time,
  airjuggle=15, sparkno, guard.sparkno, KO.echo, IntPersistIndex/FloatPersistIndex), `[Size]`
  (xscale, yscale, ground.back/front, air.back/front, height, attack.dist, head.pos, mid.pos, shadowoffset,
  draw.offset), `[Velocity]` (walk.fwd/back, run.fwd/back, jump.neu/back/fwd, runjump, airjump.*),
  `[Movement]` (airjump.num, airjump.height, yaccel ~0.44, stand.friction 0.85, crouch.friction 0.82, ...).
- `[Statedef N]` sets `type` (S stand, C crouch, A air, L lying, U unchanged), `movetype` (I idle, A attack,
  H being hit), `physics` (S, C, A applies gravity, N none), `anim`, `velset`, `ctrl`, `poweradd`,
  `juggle`, `facep2`, `hitdefpersist`, `movehitpersist`, `hitcountpersist`, `sprpriority`.
- `[State N, label]` blocks are controllers: `type = <SCTRL>`, `triggerall = ...`, `trigger1..n = ...` (an AND within
  one number, OR across numbers), plus `persistent`, `ignorehitpause`.
- Special statedefs: **-3** (every tick unless the player is in a custom state), **-2** (every tick, always), **-1** (CMD
  logic). They are evaluated before the current state's controllers, every tick.
- Triggers (expression language with int/float, `&& || ^^ !`, range `[a,b]`/`(a,b)`, `ifelse`, 1.1 `cond`):
  Time, AnimElem, AnimElemTime, AnimTime, Anim, StateNo, PrevStateNo, Command, Ctrl, StateType,
  MoveType, MoveContact/MoveHit/MoveGuarded/MoveReversed, HitCount, HitDefAttr, HitShakeOver, HitOver,
  P2BodyDist/P2Dist/P2StateType/P2MoveType, Life/LifeMax/Power, Pos/Vel/ScreenPos, BackEdgeDist,
  FrontEdgeDist, RoundState, RoundNo, GameTime, Random (0-999), NumHelper/NumExplod/NumProj/NumTarget,
  ProjHit/ProjContact/ProjGuarded(id), IsHelper, Alive, Var/FVar/SysVar/SysFVar, Const(...), GetHitVar(...).
  **Redirection**: `parent, root, helper(id), target(id), enemy(n), enemynear(n), partner, playerid(id)`.
- Core controllers: ChangeState, SelfState, TargetState, ChangeAnim/ChangeAnim2 (the latter uses the owner's AIR on P2),
  VelSet/VelAdd/VelMul, PosSet/PosAdd, CtrlSet, StateTypeSet, HitDef, ReversalDef, Projectile, Helper,
  Explod/ModifyExplod/RemoveExplod, PlaySnd/StopSnd, VarSet/VarAdd/VarRandom/VarRangeSet/ParentVarSet,
  PowerAdd, LifeAdd, TargetLifeAdd/TargetPowerAdd/TargetBind/TargetVelSet/TargetFacing/TargetDrop,
  BindToTarget/BindToParent/BindToRoot, SuperPause, Pause, EnvShake, EnvColor, PalFX/AllPalFX/BGPalFX,
  AfterImage/AfterImageTime, AssertSpecial (flags such as noautoturn, invisible, nowalk, intro, roundnotover),
  NotHitBy/HitBy, HitOverride, HitFallVel/HitFallSet/HitFallDamage, Width, PlayerPush, ScreenBound,
  PosFreeze, Turn, Trans, AngleDraw/AngleSet/AngleAdd, SprPriority, Gravity, MoveHitReset, HitAdd,
  AttackMulSet/DefenceMulSet, DestroySelf, MakeDust, GameMakeAnim, VictoryQuote, Null, DisplayToClipboard.
- **common1.cns** (shared default states): 0 stand, 10-12 stand to crouch, 20 walk, 40 jump start,
  45 air jump, 50 jump up, 52 land, 100 run, 105-106 hop back, 120-155 guard (start, stand, crouch, air,
  guard-hit), 170-175 lose/draw/time over, 180-191 win/intro, 5000-5210 **get-hit chain** (5000 stand shake,
  5001 knockback, 5010/5011 crouch, 5020 air, 5030/5035 air fall, 5040, 5050 falling, 5070/5071 trip,
  5080/5081 down, 5100/5101 bounce, 5110 lying, 5120 get-up, 5150 lying dead, 5200-5210 air recovery),
  5900 round init. Matching reserved anims: 0 stand, 5 turn, 10/11/12 crouch, 20/21 walk, 40-47 jump,
  120-155 guard, 5000-5210 get-hit (Elecbyte "required sprites" list).

### 1.6 Variables
`var(0..59)` int and `fvar(0..39)` float per player/helper; `sysvar(0..4)`/`sysfvar(0..4)` are engine-reserved by
convention. They reset between rounds except indices at or above IntPersistIndex/FloatPersistIndex. Helpers have their own sets and
read the parent's through `parent,var(n)`. There are no strings, arrays or maps (Ikemen adds map variables).

### 1.7 Hit system (HitDef)
HitDef arms an attack. Contact is tested each tick as **P1 Clsn1 vs P2 Clsn2**. The HitDef deactivates after the
first contact (multi-hit needs re-arming via new HitDefs or `MoveHitReset`). Key parameters:
- `attr = S|C|A, NA|SA|HA|NP|SP|HP|NT|ST|HT` (state type, normal/special/hyper, attack/projectile/throw).
- `hitflag = MAFD` (H high, L low, A air, M = H+L, F falling, D down, `+`/`-` modifiers), `guardflag = MA`,
  `affectteam = E|B|F`.
- `damage = hit, guard`, `pausetime = p1, p2` (hitstop), `guard.pausetime`, `ground.hittime`,
  `ground.slidetime`, `guard.hittime/slidetime/ctrltime`, `air.hittime`, `guard.dist`.
- `animtype = light|medium|hard|back|up|diagup`, `air.animtype`, `fall.animtype`, `ground.type = high|low|trip|none`.
- `ground.velocity`, `air.velocity`, `guard.velocity`, `airguard.velocity`, `ground.cornerpush.veloff`, `yaccel`.
- `fall`, `fall.recover`, `fall.recovertime`, `fall.xvelocity/yvelocity`, `fall.damage`, `air.fall`, `down.bounce`,
  `down.velocity`, `down.hittime`, `forcestand`, `forcenofall`, `air.juggle` (cost against the target's juggle points).
- `priority = 4, Hit|Miss|Dodge` (trade resolution), `id`, `chainID`, `nochainID`, `hitonce`, `kill`, `guard.kill`.
- FX: `sparkno` (an `S` prefix = character's own SFF), `sparkxy`, `guard.sparkno`, `hitsound = S5,0` (S = the character's SND,
  otherwise common `fight.snd`), `guardsound`, `palfx.*`, `envshake.*`, `fall.envshake.*`.
- Throws and custom states: `p1stateno`/`p2stateno` + `p2getp1state` put P2 into **P1's states** (a custom state using
  P1's CNS with P2's sprites, or P1's AIR via ChangeAnim2). `p1facing/p2facing`, `mindist/maxdist/snap`, `numhits`,
  `getpower/givepower`. Also `ReversalDef` (counters), `HitOverride` (custom get-hit), `NotHitBy` (invulnerability).

### 1.8 Helpers, Explods, Projectiles
- **Helper**: a full sub-player with its own state machine, vars, collision and HitDefs (`helpertype = normal|player`,
  `name`, `id`, `pos`, `postype p1|p2|front|back|left|right`, `stateno`, `keyctrl` (reads -1 commands),
  `ownpal`, `size.*` overrides, `supermovetime`, `pausemovetime`). It is used for complex projectiles, summons, AI tricks
  and global effects. There is a per-player cap of roughly 56 in 1.x.
- **Explod**: a visual-only animation (`anim` with an `F` prefix = fightfx.sff), `id`, `pos`, `postype`, `bindtime`,
  `vel`, `accel`, `removetime` (-1 never, -2 when the anim ends), `scale`, `sprpriority`, `ontop`, `removeongethit`,
  `trans`, `ignorehitpause`, `supermove/pausemovetime`. ModifyExplod/RemoveExplod change or remove one by id.
- **Projectile**: a lightweight entity with HitDef params plus `projid`, `projanim/hitanim/remanim/cancelanim`,
  `velocity/accel/velmul`, `projhits`, `projmisstime`, `projpriority` (beam clashes), `projremovetime`,
  `projedgebound/stagebound/heightbound`, `offset`, `postype`. Triggers: ProjContact/ProjHit/ProjGuarded/ProjCancelTime.

### 1.9 SND
Signature `ElecbyteSnd\0`, version, sound count, and the offset of the first subfile. Each subheader holds next offset, length,
group, sample number, then raw **WAV (RIFF)** data. PlaySnd parameters: `value = g,n`, `channel`, `volume`, `lowpriority`,
`freqmul`, `loop`, `pan/abspan`. The common `F` prefix means `fight.snd`.

### 1.10 Screenpack / system files (`data/`)
`mugen.cfg` (GameSpeed, resolution, input, 1.1 renderer OpenGL vs. System), `system.def` (`[Files]` spr/snd/fonts,
`[Title Info]` menu, `[Select Info]` grid rows/columns/cell size/cursors/portrait offsets, `[VS Screen]`,
`[Continue Screen]`, `[Option Info]`, plus backgrounds as `[TitleBG ...]`), `fight.def` (lifebars: `[Lifebar]`,
`[Powerbar]`, `[Face]`, `[Name]`, `[Time]`, `[Combo]`, `[Round]` with announcer sounds, `[WinIcon]`), `select.def`
(`[Characters]` roster lines `kfm, stages/kfm.def, music=...`, `[ExtraStages]`, `[Options]` arcade.maxmatches),
`fightfx.sff/.air` (common sparks), `common.snd`, and stages (`stages/*.def`: `[Camera]`, `[PlayerInfo]`, `[Bound]`,
`[StageInfo]` zoffset/localcoord, `[BGdef]` + `[BG *]` layers with parallax delta, tile, velocity, animated BGs, BGCtrl).

### 1.11 Game loop (fixed 60 ticks/s; the logic speed is decoupled from rendering only in Ikemen)
Per tick: poll input, then update command buffers. For each player and helper: unless paused, run the -3/-2/-1 controllers
and then the current state's controllers (a ChangeState restarts evaluation in the new state on the same tick), apply velocity,
gravity and friction according to `physics`, and advance the animation. Then push players apart (`width`), clamp to screen and stage bounds,
resolve **hit detection** (Clsn1 vs Clsn2, priority/trade, guard check: P2 holding back, in guard.dist, and the attack is
guardable), and apply hit/guard results (damage, hitpause freezes both sides for `pausetime`, P2 goes to 5000+ or a custom state).
Projectiles/explods update, the camera updates, and the frame renders (draw order by sprpriority, then layer). SuperPause/Pause freeze
everyone except movers with `supermovetime`/`pausemovetime`. Round state machine: 0 intro, 1-2 fight, 3 KO, 4 win pose.

---------------------------------------------------------------------------------------------------
## 2. Modern successors

### Ikemen GO (github.com/ikemen-engine/Ikemen-GO). The de facto successor.
- History: Suehiro's original "I.K.E.M.E.N" (written in his SSZ language) was ported to **Go**. It is now maintained by the community
  (K4thos and others). **MIT** licence for the engine; the bundled default screenpack is CC-BY 3. **v1.0.0 released 12 Sep (latest stable;
  the year is likely 2025, after 0.99.0 on 28 Oct)**. Nightlies are built from `develop`.
- Platforms: Win10 x64, macOS 13 Apple Silicon, Linux x64, **Android 13 arm64, iOS 17**. Rendering uses OpenGL 3.3/GLES 3.2 or
  **Vulkan**. There is no web build.
- Adds to MUGEN 1.1 compatibility: **Lua**-scripted menus, modes and screenpack logic (`start.lua`/`main.lua`, hooks);
  **ZSS** (Zantei State Script, a terser block syntax for states as an alternative to CNS); many new triggers and controllers (see wiki
  "State controllers (new)"); map variables; tag/simul/turns/ratio team modes, training mode; TTF fonts; 32-bit PNG sprites;
  **glTF 3D stages** (1.0); localization; resizable/aspect options; replays.
- **Rollback netcode**: the community implementation (PR #2610 by facundocameto; public "rollback open alpha" Jan 2023 on top of 0.99) was
  merged into develop and **enabled by default** in 1.0 (PR #3787, K4thos), with rollback replays (PR #3381) and sync tests.
  It is native Go, not GGPO.
- Limitations: not bug-for-bug MUGEN (some 1.x chars misbehave); Lua/screenpack APIs churn between versions; the
  content-authoring workflow is still hand-edited text plus Fighter Factory; mobile builds have no first-class touch-first UX;
  there is no browser/WASM target; and the large Go codebase is tied to native GL/Vulkan.

### Others
- **Fighter Factory (Studio / Ultimate)** by VirtuallTek: the standard MUGEN/Ikemen character IDE (SFF/AIR/CNS editors). It is closed source and runs on Windows.
- **Web**: `roundonejs_mugen_renderer` (TypeScript, parses MUGEN chars and draws them to canvas, animation only, inactive since 2021);
  `barklim/mugenTest` (WASM experiment, reported broken in Chrome). There is **no maintained full browser MUGEN engine**, which is a clear gap.
- **Non-MUGEN** fighting-game toolkits: Universal Fighting Engine (Unity, commercial), Castagne (Godot, open source),
  OpenBOR (beat 'em up). For netcode there are GGPO (open-sourced 2019, C++), **GGRS** (Rust, used with Bevy/WASM, P2P via
  matchbox/WebRTC), and backroll.

---------------------------------------------------------------------------------------------------
## 3. Pain points and what an ideal modern engine should do

Pain points in MUGEN authoring:
1. Untyped text DSLs (CNS/CMD/AIR) with weak errors: a typo silently disables a controller and runtime errors are vague.
2. Magic numbers everywhere (state 5000, anim 9000, var(17) as an "implicit API"). There are no names, structs or modules.
3. No hot reload: you restart the match after every tweak. Debug mode (Ctrl+D) is primitive.
4. Hitboxes are edited by hand as coordinates. Frame data is not visualized (startup/active/recovery, plus/minus on block).
5. Palette and indexed-color constraints. Sprite ripping and alignment are tedious, and axis offsets are error-prone.
6. Cross-version incompatibility (WinMUGEN, 1.0, 1.1, localcoord) and "cheap/god-tier" chars that abuse engine bugs.
7. Distribution is ZIP files on forums with dead links. There is no package manifest, dependency or version metadata.
8. No netplay in MUGEN, desktop only, and no touch controls.

Ideal engine (recommendations):
- **Data model**: a canonical **JSON/YAML (or TOML) schema** for characters (meta, constants, animations with
  named hit/hurt boxes, moves with inputs, states as a graph), validated with JSON Schema/Zod. Keep **MUGEN importers**
  (DEF/SFF v1+v2/AIR/CMD/CNS/SND parsing into the internal model) so legacy chars run, plus an exporter for round-tripping.
- **Deterministic sim**: fixed 60 Hz logic, integer or fixed-point math, a seeded PRNG, and state that can be serialized in a few KB.
  These are prerequisites for **rollback** (GGPO-style: input delay 1-2 frames, predict, rollback up to ~8 frames, resim)
  and for replays and desync detection (checksums per frame). Separate the sim from rendering (interpolation).
- **Expressions**: compile CNS triggers into a small bytecode/AST once at load time and avoid string evaluation per tick.
- **Rendering**: WebGL2 now, WebGPU when available (PixiJS v8 or a custom renderer). Use palette shaders (indexed texture plus
  palette LUT) to keep pal swaps cheap, and a texture atlas per character.
- **Tooling**: an in-browser **visual editor** (timeline scrubber, drag hitboxes, frame-data overlay, input-command
  tester, state graph view), **hot reload** (watch files and swap the char without restarting the match), and a debug overlay with
  state/anim/vars.
- **Input**: a unified gamepad/keyboard/**touch** layer. On mobile, a virtual stick or d-pad plus 4-6 buttons with an
  optional "simple mode" (motion inputs replaced by one-button specials) and haptics. Use a configurable input
  buffer and leniency.
- **Platforms**: a web-first **PWA** (offline cache, installable), with **Capacitor** for iOS/Android stores and
  **Tauri** for small desktop builds. Netplay uses WebRTC data channels (unreliable/unordered) with a signaling server.
- **Content safety and legal**: never bundle copyrighted rips. Ship original/CC sample chars (for example Kung Fu Man-style
  templates) and let users **import their own** ZIPs locally (File System Access API / drag-drop, sandboxed parse,
  no script execution).

---------------------------------------------------------------------------------------------------
## 4. Catalog of well-known MUGEN characters (names only; users import their own copies)

Confidence markers: (H) = author is well established in the community; (M) = likely, please verify;
(?) = the character is popular but the author is ambiguous or there are many versions. Check MUGEN Archive, Mugen Free For All,
MFG (mugenguild.com) and the MUGEN Database wiki before crediting. Most are fan works using copyrighted sprites.

**Engine / original**
- Kung Fu Man (Elecbyte, H). This is the template char and fully legal to reference.

**Street Fighter / Capcom**
- Ryu, Ken, Akuma/Gouki, Shin Akuma, Dan, Sakura, Morrigan: Phantom.of.the.Server "PotS-style" (H).
- Juri, Hugo and other SF/CvS chars: Warusaki3 (M).
- Capcom vs. SNK 2 conversions: Byakko (H, per MUGEN Database).
- "Evil Ryu", "Violent Ken", "Rare Akuma" variants exist from many authors (?).

**King of Fighters / SNK**
- Kyo Kusanagi, Iori Yagami, K', Terry Bogard, Mai Shiranui, Rugal, Orochi Iori: Ahuron and Vans (KOF2002-style) (M);
  PotS-style KOF entries also exist (M).

**Marvel / MvC**
- Spider-Man, Wolverine, Hulk, Iron Man, Sentinel, Cable, Captain America, Venom, Magneto: Infinity Mugen Team and Kong
  MvC-style conversions (M).

**Dragon Ball**
- Goku (SSJ1-4, UI), Vegeta, Gohan, Piccolo, Frieza, Cell, Broly, Majin Buu, Gogeta/Vegito: Team Z2 (Cybaster et al.) (H, per MUGEN Database).
- *Hyper Dragon Ball Z* full game: Balthazar and the Hyper DBZ team (H). Separate JUS-style DB chars also exist (?).

**Naruto**: Naruto (Sage/KCM), Sasuke, Itachi, Madara, Kakashi, Pain, Minato, Gaara: JUS-style and Storm-style versions (?).

**Bleach**: Ichigo (Bankai/Hollow/Final Getsuga), Aizen, Byakuya, Kenpachi, Ulquiorra, Grimmjow, Yamamoto: JUS/Heat-the-Soul sprite
  edits (?). Note that *Bleach vs Naruto* is a separate Flash game by 5Dplay, not MUGEN.

**One Piece**: Luffy (Gear 2/4/5), Zoro, Sanji, Ace, Whitebeard, Akainu, Law: JUS-style (?).

**JoJo's Bizarre Adventure**: Jotaro, DIO, Polnareff, Kakyoin, Joseph, Giorno: CPS3 *Heritage for the Future* conversions (?).

**Hunter x Hunter**: Gon, Killua, Hisoka, Kurapika, Meruem, Netero: JUS-style (?).

**Other popular anime and game chars**: Touhou cast (Ricepigeon, H); Melty Blood / Arcana Heart / Guilty Gear / BlazBlue
  conversions (?); *Anime Supreme Battle* and *Dengenki vs BlazBlue* JUS-style full games by Mugen_Master335 (H, itch.io).
  The meme/cheap tier (Omega Tom Hanks, "god"/"cheapie" chars) is popular on YouTube and Salty Bet but intentionally broken (?).

Recommendation: in the app, show this as a *search-hint list* only. Store the author from the imported `[Info] author=` field
rather than hard-coding credits.

---------------------------------------------------------------------------------------------------
## 5. github.com/morluto/rea

- **morluto/rea = "REA – Reverse Engineer Anything"**: an **MCP server and CLI** (Node/TypeScript, `npx rea-agents setup`) that
  lets AI coding agents (Claude Code, Codex, Cursor, Gemini CLI) inspect authorized software locally. It covers native binaries via
  Hopper/Ghidra/IDA/Kuna (morluto's Rust Ghidra port), JS/Electron, .NET, APK, Go/Rust, WASM, firmware and pcaps, plus browser
  behavior capture. Game showcases include recovering a Touhou TH04 (PC-98) bullet-ring formula and a DX-Ball sound-pan calc.
- The same account has many `rea-*` forks (lists of agent/Claude/Codex skills), plus `kuna`, `flameox`, `leantoken`,
  `awesome-reverse-engineering`.
- **Relevance**: indirect only. It has no MUGEN content. It could help an agent study `mugen.exe` behavior for edge-case
  compatibility, but Ikemen GO's MIT source plus the Elecbyte docs already cover nearly everything. Respect the MUGEN EULA if used.
  It is not a dependency for this project.

---------------------------------------------------------------------------------------------------
## Key sources
- https://github.com/ikemen-engine/Ikemen-GO (README, /wiki, /releases v1.0.0: PR #2610 rollback, #3787 default-on, #3381 replays)
- https://pkg.go.dev/github.com/openkakutou/sff ; https://github.com/kumohakasemk9/MUGENSFFReaderLibJava
- https://github.com/roundonejs/roundonejs_mugen_renderer ; barklim.github.io/mugenTest
- Elecbyte docs (sprmake2, cmd, cns, sctrls, trigger, air, spr): elecbyte.com/mugendocs (blocked here; mirrors on mugenguild/archive.org)
- https://mugen.fandom.com (Morrigan/PotS, Cybaster/Team Z2, Byakko) ; https://mugen-master335.itch.io
- https://github.com/morluto/rea ; https://github.com/morluto?tab=repositories
