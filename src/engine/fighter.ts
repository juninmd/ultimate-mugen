import type { Animation, AnimFrame, Box } from '../mugen/air';
import { compileNode, type Fn } from './expr';
import { compileParam, type CharacterDef, type StateController, type StateDef } from './chardef';
import { CommandState } from './commands';
import { InputHistory, type InputSource } from './input';
import { resolver } from './triggers';
import { runController } from './controllers';
import type { Match } from './match';

export interface HitDefData {
  attr: string; // e.g. "S,NA"
  attrState: string; // S/C/A letters
  attrKind: string[]; // NA, SA, HP...
  hitflag: string;
  guardflag: string;
  animtype: number; // 0 light 1 med 2 hard 3 back 4 up 5 diagup
  airAnimtype: number;
  fallAnimtype: number;
  priority: number;
  damage: number; guardDamage: number;
  pauseP1: number; pauseP2: number;
  guardPauseP1: number; guardPauseP2: number;
  sparkNo: number; sparkOwn: boolean; guardSparkNo: number; guardSparkOwn: boolean;
  sparkX: number; sparkY: number;
  hitSound: [number, number, boolean] | null; // group, index, own
  guardSound: [number, number, boolean] | null;
  groundType: number; // 0 none 1 high 2 low 3 trip
  airType: number;
  groundSlideTime: number; guardSlideTime: number;
  groundHitTime: number; guardHitTime: number; airHitTime: number;
  guardCtrlTime: number;
  guardDist: number;
  yaccel: number;
  groundVel: [number, number]; guardVel: number; airVel: [number, number]; airGuardVel: [number, number];
  downVel: [number, number];
  groundCornerPush: number; airCornerPush: number;
  airJuggle: number;
  p1sprpriority: number; p2sprpriority: number;
  p1facing: number; p2facing: number;
  p1stateno: number; p2stateno: number; p2getp1state: boolean;
  forcestand: boolean; forcenofall: boolean;
  fall: boolean; airFall: boolean;
  fallXvel: number | null; fallYvel: number; fallRecover: boolean; fallRecoverTime: number; fallDamage: number;
  fallKill: boolean; kill: boolean; guardKill: boolean;
  getPower: number; givePower: number; guardGetPower: number; guardGivePower: number;
  id: number; chainId: number; nochainId: number[];
  envShakeTime: number;
  hitOnce: boolean;
  numHits: number;
  palfxTime: number;
}

export interface GetHitVars {
  type: number; animtype: number; airtype: number; groundtype: number;
  damage: number; hitcount: number; fallcount: number; guardcount: number;
  hittime: number; slidetime: number; ctrltime: number; recovertime: number;
  xvel: number; yvel: number; yaccel: number;
  chainid: number; guarded: boolean;
  fall: boolean; fallDamage: number; fallXvel: number; fallYvel: number;
  fallRecover: boolean; fallRecoverTime: number; fallKill: boolean; kill: boolean;
  attackerId: number; attackerNo: number;
}

const emptyGhv = (): GetHitVars => ({
  type: 0, animtype: 0, airtype: 0, groundtype: 0, damage: 0, hitcount: 0, fallcount: 0, guardcount: 0,
  hittime: 0, slidetime: 0, ctrltime: 0, recovertime: 0, xvel: 0, yvel: 0, yaccel: 0.44, chainid: -1, guarded: false,
  fall: false, fallDamage: 0, fallXvel: 0, fallYvel: -4.5, fallRecover: true, fallRecoverTime: 4, fallKill: true, kill: true,
  attackerId: 0, attackerNo: 0,
});

/** fallback chains for missing animations */
const ANIM_FALLBACK: Record<number, number[]> = {
  5: [0], 6: [11, 0], 10: [11, 0], 11: [0], 12: [0], 20: [0], 21: [20, 0], 40: [41, 0], 41: [40, 0], 42: [41, 40, 0], 43: [41, 40, 0],
  44: [41, 40, 0], 45: [42, 41, 40, 0], 46: [43, 41, 40, 0], 47: [40, 0], 100: [20, 0], 105: [43, 41, 0], 106: [47, 0],
  120: [130, 0], 121: [131, 11, 0], 122: [132, 41, 0], 130: [120, 0], 131: [121, 11], 132: [122, 41],
  140: [130, 0], 141: [131, 11], 142: [132, 41], 150: [130, 5000, 0], 151: [150, 130, 0], 152: [131, 5010, 11], 153: [152, 131, 11], 154: [132, 5030, 41], 155: [154, 132, 41],
  170: [0], 175: [0], 180: [181, 0], 181: [0], 190: [0], 191: [0],
  5000: [5001, 5002, 0], 5001: [5000, 0], 5002: [5001, 5000, 0], 5005: [5000, 0], 5006: [5005, 5001, 5000], 5007: [5006, 5005, 5002, 5000],
  5010: [5000, 0], 5011: [5010, 5001, 5000], 5012: [5011, 5010, 5002, 5000], 5015: [5005, 5010, 5000], 5016: [5015, 5006, 5011], 5017: [5016, 5007, 5012],
  5020: [5010, 5000, 11], 5021: [5020, 5011, 5001], 5022: [5021, 5012, 5002], 5025: [5015, 5020], 5026: [5025, 5021], 5027: [5026, 5022],
  5030: [5000, 0], 5035: [5030, 5000], 5040: [5030, 41, 0], 5050: [5030, 5000], 5051: [5050], 5052: [5050], 5060: [5050, 5030], 5061: [5060, 5050],
  5070: [5050, 5030], 5071: [5050, 5030], 5080: [5110, 5050], 5090: [5110, 5050], 5100: [5110, 5050], 5101: [5100, 5050], 5110: [5100, 5050],
  5120: [5110, 0], 5140: [5110], 5150: [5110], 5160: [5101, 5100], 5170: [5110], 5200: [5040, 41], 5210: [5040, 41],
};

let fighterIdCounter = 56;

export class Fighter {
  id = fighterIdCounter++;
  def: CharacterDef;
  /** whose states we are running (custom states) */
  stDef: CharacterDef;
  match: Match;
  side: number;
  playerNo: number;
  isHelper = false;
  helperId = 0;
  helperType: 'normal' | 'player' | 'proj' = 'normal';
  keyCtrl = false;
  parent: Fighter | null = null;
  root: Fighter = this;
  destroyed = false;
  palNo = 1;
  aiLevel = 0;

  pos = { x: 0, y: 0 };
  vel = { x: 0, y: 0 };
  facing = 1;
  lscl = 1; // engine units per local unit

  stateNo = 0;
  prevStateNo = 0;
  stateTime = 0;
  stateType = 'S';
  moveType = 'I';
  physics = 'S';
  ctrl = true;

  life = 1000; lifeMax = 1000; power = 0; powerMax = 3000;
  alive = true;
  vars = new Int32Array(60);
  fvars = new Float64Array(40);
  sysvars = new Int32Array(5);
  sysfvars = new Float64Array(5);

  animNo = -1;
  anim: Animation | null = null;
  animDef: CharacterDef;
  elem = 0; // 0-based
  elemTime = 0;
  animElapsed = 0;
  animChangedThisTick = false;

  hitdef: HitDefData | null = null;
  hitdefActive = false;
  hitdefTargets = new Set<number>();
  moveContact = 0; moveHit = 0; moveGuarded = 0; moveReversed = 0;
  hitCount = 0;
  ghv: GetHitVars = emptyGhv();
  hitPause = 0;
  hitShake = 0;
  hitOverTime = 0;
  slideTime = 0;
  fallTime = 0;
  juggle = 0;
  comboCount = 0;
  comboDamage = 0;

  asserts = new Set<string>();
  widthOverride: { front: number; back: number } | null = null;
  playerPush = true;
  screenBound = true;
  posFreeze = false;
  sprPriority = 0;
  targets: { f: Fighter; hitId: number }[] = [];
  notHitBy: { attr: string; time: number }[] = [];
  hitBy: { attr: string; time: number }[] = [];
  attackMul = 1;
  defenceMul = 1;
  persist = new Map<number, number>();
  airJumps = 0;
  pauseMoveTime = 0;
  superMoveTime = 0;
  bind: { to: Fighter; time: number; x: number; y: number; kind: 'target' | 'parent' | 'root' } | null = null;
  palfx: { time: number; add: [number, number, number]; mul: [number, number, number]; color: number; sinadd?: [number, number, number, number] } | null = null;
  trans: { mode: string; src: number; dst: number } | null = null;
  angle = 0;
  angleDraw = false;
  drawScale: [number, number] = [1, 1];
  afterImage: { time: number; length: number; trail: { x: number; y: number; frame: AnimFrame | null; facing: number }[] } | null = null;
  projContacts = new Map<string, number>(); // `${kind}:${id}` -> gameTime
  lastHitBy: Fighter | null = null;
  winQuote = '';

  input: InputSource | null = null;
  history = new InputHistory();
  cmd: CommandState;
  pendingState: { no: number; owner: CharacterDef } | null = null;

  constructor(match: Match, def: CharacterDef, side: number, playerNo: number) {
    this.match = match;
    this.def = def;
    this.stDef = def;
    this.animDef = def;
    this.side = side;
    this.playerNo = playerNo;
    this.cmd = new CommandState(def.commands);
    this.lscl = 320 / (def.localW || 320);
    this.lifeMax = this.life = def.constants.get('data.life') ?? 1000;
    this.powerMax = def.constants.get('data.power') ?? 3000;
    this.attackMul = (def.constants.get('data.attack') ?? 100) / 100;
  }

  // ------------------------------------------------------------ helpers for triggers
  const(k: string, d = 0): number { return this.def.constants.get(k) ?? d; }
  hasCommand(name: string): boolean {
    const src = this.isHelper ? (this.keyCtrl ? this.root : null) : this;
    if (!src) return false;
    if (src.match.roundState !== 2 && src.match.roundState !== 3 && !src.match.training) return false;
    return src.cmd.active.has(name);
  }
  enemy(n = 0): Fighter | undefined {
    const list = this.match.roots.filter((r) => r.side !== this.side && !r.destroyed);
    if (!list.length) return undefined;
    if (list.length > 1) list.sort((a, b) => Math.abs(a.pos.x - this.pos.x) - Math.abs(b.pos.x - this.pos.x));
    return list[n | 0];
  }
  findHelper(id: number): Fighter | undefined {
    return this.match.fighters.find((f) => f.isHelper && f.root === this.root && !f.destroyed && (id < 0 || f.helperId === id));
  }
  findTarget(id: number): Fighter | undefined {
    const t = this.targets.find((t) => (id < 0 || t.hitId === id) && !t.f.destroyed);
    return t?.f;
  }
  countTargets(id: number) { return this.targets.filter((t) => id < 0 || t.hitId === id).length; }
  frontWidth(): number {
    if (this.widthOverride) return this.widthOverride.front;
    return (this.stateType === 'A' ? this.const('size.air.front', 12) : this.const('size.ground.front', 16)) * this.lscl;
  }
  backWidth(): number {
    if (this.widthOverride) return this.widthOverride.back;
    return (this.stateType === 'A' ? this.const('size.air.back', 12) : this.const('size.ground.back', 15)) * this.lscl;
  }
  edgeDist(front: boolean): number {
    const m = this.match;
    const left = m.camera.x - m.viewW / 2, right = m.camera.x + m.viewW / 2;
    const toRight = right - this.pos.x, toLeft = this.pos.x - left;
    return (this.facing > 0) === front ? toRight : toLeft;
  }
  inGuardDist(): boolean {
    for (const e of this.match.fighters) {
      if (e.side === this.side || e.destroyed) continue;
      if (e.moveType === 'A' && e.hitdefActive && e.hitdef) {
        if (Math.abs(e.pos.x - this.pos.x) <= e.hitdef.guardDist * e.lscl) return true;
      }
    }
    for (const p of this.match.projectiles) {
      if (p.owner.side !== this.side && Math.abs(p.pos.x - this.pos.x) <= p.guardDist) return true;
    }
    return false;
  }
  hitdefAttrMatches(vals: string[]): boolean {
    if (!this.hitdef || !this.hitdefActive && this.moveType !== 'A') return false;
    if (!this.hitdef) return false;
    const st = vals[0].toUpperCase();
    if (![...this.hitdef.attrState].some((c) => st.includes(c))) return false;
    if (vals.length === 1) return true;
    return vals.slice(1).some((v) => this.hitdef!.attrKind.includes(v.toUpperCase()));
  }
  projContactTime(kind: 'hit' | 'contact' | 'guarded', id: number): number {
    let best = -1;
    for (const [k, t] of this.projContacts) {
      const [kk, ii] = k.split(':');
      if (kind !== 'contact' && kk !== kind) continue;
      if (id && parseInt(ii) !== id) continue;
      const dt = this.match.gameTime - t;
      if (best < 0 || dt < best) best = dt;
    }
    return best;
  }

  // ------------------------------------------------------------ animation
  animTimeTrig(): number {
    if (!this.anim) return 0;
    if (this.anim.totalTime < 0) return -1;
    return this.animElapsed - this.anim.totalTime;
  }
  elemStart(n: number): number {
    if (!this.anim) return 0;
    let t = 0;
    for (let i = 0; i < n - 1 && i < this.anim.frames.length; i++) t += Math.max(0, this.anim.frames[i].time);
    return t;
  }
  animElemStarted(n: number): boolean {
    return this.elem + 1 === n && this.elemTime === 0 && (n > 1 || this.animElapsed === 0 || this.anim?.loopStart === 0);
  }
  animElemTime(n: number): number | null {
    if (!this.anim || n < 1 || n > this.anim.frames.length) return null;
    return this.animElapsed - this.elemStart(n);
  }
  animElemNoAt(off: number): number {
    if (!this.anim) return 0;
    let t = this.animElapsed + off;
    const tot = this.anim.totalTime;
    if (tot > 0 && t >= tot) {
      const ls = this.elemStart(this.anim.loopStart + 1);
      const loopLen = tot - ls;
      t = loopLen > 0 ? ls + ((t - ls) % loopLen) : ls;
    }
    let acc = 0;
    for (let i = 0; i < this.anim.frames.length; i++) {
      const ft = this.anim.frames[i].time;
      if (ft < 0 || t < acc + ft) return i + 1;
      acc += ft;
    }
    return this.anim.frames.length;
  }
  changeAnim(no: number, from: CharacterDef = this.def, elem = 1) {
    let a = from.anims.get(no);
    if (!a) {
      for (const alt of ANIM_FALLBACK[no] ?? []) { a = from.anims.get(alt); if (a) break; }
    }
    if (!a) {
      if (this.anim && from === this.animDef) { this.animNo = no; return; }
      a = from.anims.get(0);
      if (!a) return;
    }
    this.anim = a;
    this.animDef = from;
    this.animNo = no;
    this.elem = 0; this.elemTime = 0; this.animElapsed = 0;
    if (elem > 1) {
      this.animElapsed = this.elemStart(elem);
      this.elem = Math.min(elem - 1, a.frames.length - 1);
    }
    this.animChangedThisTick = true;
  }
  advanceAnim() {
    const a = this.anim;
    if (!a || !a.frames.length) return;
    if (this.animChangedThisTick) { this.animChangedThisTick = false; return; }
    if (a.totalTime >= 0 && this.animElapsed >= a.totalTime) this.animElapsed = this.elemStart(a.loopStart + 1);
    const fr = a.frames[this.elem];
    if (fr.time < 0) { this.animElapsed++; return; }
    this.animElapsed++;
    this.elemTime++;
    if (this.elemTime >= fr.time) {
      this.elemTime = 0;
      this.elem++;
      if (this.elem >= a.frames.length) this.elem = a.loopStart;
      // skip zero-length frames
      let guard = 0;
      while (a.frames[this.elem].time === 0 && guard++ < a.frames.length) {
        this.elem++;
        if (this.elem >= a.frames.length) this.elem = a.loopStart;
      }
    }
  }
  frame(): AnimFrame | null { return this.anim?.frames[this.elem] ?? null; }

  // ------------------------------------------------------------ state machine
  stateDef(no: number, owner = this.stDef): StateDef | undefined {
    return owner.states.get(no) ?? this.match.common.states.get(no);
  }

  changeState(no: number, owner: CharacterDef = this.stDef) {
    const sd = owner.states.get(no) ?? this.match.common.states.get(no);
    this.prevStateNo = this.stateNo;
    this.stateNo = no;
    this.stDef = owner;
    this.stateTime = 0;
    this.persist.clear();
    if (!sd) return;
    const g = (k: string) => sd.params.get(k);
    const ev = (k: string) => { const p = g(k); if (!p) return undefined; const c = compileParam<Fighter>(p, resolver); return c; };
    const raw = (k: string) => g(k)?.raw.trim().toUpperCase();
    const t = raw('type'); if (t && t !== 'U') this.stateType = t[0] === 'L' ? 'L' : t[0];
    else if (!t) this.stateType = 'S';
    const mt = raw('movetype'); if (mt && mt !== 'U') this.moveType = mt[0]; else if (!mt) this.moveType = 'I';
    const ph = raw('physics'); if (ph && ph !== 'U') this.physics = ph[0]; else if (!ph) this.physics = 'N';
    const anim = ev('anim'); if (anim && anim[0]) this.changeAnim(anim[0].f(this) | 0, this.def);
    const vs = ev('velset'); if (vs && vs.length) { this.vel.x = vs[0].f(this) * this.lscl; if (vs[1]) this.vel.y = vs[1].f(this) * this.lscl; }
    const ct = ev('ctrl'); if (ct && ct[0]) this.ctrl = ct[0].f(this) !== 0;
    const pa = ev('poweradd'); if (pa && pa[0]) this.addPower(pa[0].f(this));
    const sp = ev('sprpriority'); if (sp && sp[0]) this.sprPriority = sp[0].f(this) | 0;
    const fp = ev('facep2'); if (fp && fp[0] && fp[0].f(this)) this.faceEnemy();
    const jg = ev('juggle'); if (jg && jg[0]) this.juggle = jg[0].f(this) | 0;
    const hdp = ev('hitdefpersist'); if (!(hdp && hdp[0] && hdp[0].f(this))) { this.hitdefActive = false; }
    const mhp = ev('movehitpersist'); if (!(mhp && mhp[0] && mhp[0].f(this))) { this.moveContact = this.moveHit = this.moveGuarded = this.moveReversed = 0; }
    const hcp = ev('hitcountpersist'); if (!(hcp && hcp[0] && hcp[0].f(this))) { if (this.moveType !== 'A' || true) this.hitCount = 0; }
    if (this.moveType !== 'H') { /* leaving hit states */ }
  }

  faceEnemy() {
    const e = this.enemy(0);
    if (e && e.pos.x !== this.pos.x) this.facing = e.pos.x > this.pos.x ? 1 : -1;
  }

  addPower(v: number) { this.root.power = Math.max(0, Math.min(this.root.powerMax, this.root.power + v)); }

  /** Evaluate triggers of a controller. */
  triggered(sc: StateController): boolean {
    if (!sc._tAll) {
      sc._tAll = sc.triggerAll.map((n) => compileNode<Fighter>(n, resolver).f);
      sc._t = sc.triggers.map((list) => list.map((n) => compileNode<Fighter>(n, resolver).f));
    }
    for (const f of sc._tAll) if (!f(this)) return false;
    const ts = sc._t!;
    if (!ts.length) return sc.triggerAll.length > 0 && false;
    for (const group of ts) {
      let ok = true;
      for (const f of group) if (!f(this)) { ok = false; break; }
      if (ok) return true;
    }
    return false;
  }

  /** Runs one state's controllers. Returns true if state changed. */
  runStateControllers(sd: StateDef, inHitPause: boolean): boolean {
    for (const sc of sd.controllers) {
      if (inHitPause && !sc.ignoreHitPause) continue;
      if (!this.triggered(sc)) continue;
      // persistence
      if (sc.persistent !== 1) {
        const last = this.persist.get(sc.uid);
        if (sc.persistent === 0) { if (last !== undefined) continue; }
        else if (last !== undefined && this.stateTime - last < sc.persistent) continue;
        this.persist.set(sc.uid, this.stateTime);
      }
      const startNo = this.stateNo, startT = this.stateTime, startDef = this.stDef;
      runController(this, sc);
      if (this.destroyed) return true;
      if (this.stateNo !== startNo || this.stDef !== startDef || (this.stateTime === 0 && startT !== 0)) return true;
      if (this.pendingState) return true;
    }
    return false;
  }

  /** Full per-tick state processing (-3, -2, -1, current). */
  runStates() {
    const inPause = this.hitPause > 0;
    this.asserts.clear();
    const own = this.def;
    // -3 & -2 run from own states (not when in a custom state for -3)
    if (!this.isHelper && this.stDef === own) {
      const s3 = own.states.get(-3);
      if (s3) this.runStateControllers(s3, inPause);
    }
    const s2 = own.states.get(-2);
    if (s2) this.runStateControllers(s2, inPause);
    if ((!this.isHelper || this.keyCtrl) && this.stDef === own) {
      const s1 = own.states.get(-1);
      if (s1) {
        const before = this.stateNo;
        this.runStateControllers(s1, inPause);
        if (this.stateNo !== before) { /* new state starts fresh below */ }
      }
      if (!inPause) this.builtinTransitions();
    }
    // current state; changes restart processing of the new state (bounded)
    for (let guard = 0; guard < 16; guard++) {
      const sd = this.stateDef(this.stateNo);
      if (!sd) break;
      const changed = this.runStateControllers(sd, inPause);
      if (this.destroyed) return;
      if (!changed) break;
    }
  }

  /** Hardcoded MUGEN engine behaviours: walk, crouch, jump, guard, turn. */
  builtinTransitions() {
    if (!this.ctrl || this.match.roundState !== 2 && !this.match.training) return;
    const cmd = (n: string) => this.cmd.active.has(n);
    const s = this.stateNo;
    const enemy = this.enemy(0);
    // auto-turn
    if (enemy && !this.asserts.has('noautoturn') && (s === 0 || s === 11 || s === 20 || s === 52) && this.stateType !== 'A') {
      const shouldFace = enemy.pos.x >= this.pos.x ? 1 : -1;
      if (shouldFace !== this.facing && Math.abs(enemy.pos.x - this.pos.x) > 1) {
        this.facing = shouldFace;
        if (s === 0 && this.def.anims.has(5)) this.changeAnim(5);
        if (s === 11 && this.def.anims.has(6)) this.changeAnim(6);
      }
    }
    if (this.stateType === 'S' || this.stateType === 'C') {
      // guard
      if (cmd('holdback') && this.inGuardDist() && !(s >= 120 && s <= 155)) {
        this.changeState(120, this.def);
        return;
      }
      if (this.stateType === 'S') {
        if (cmd('holdup') && !this.asserts.has('nojump')) { this.changeState(40, this.def); return; }
        if (cmd('holddown') && s !== 10 && s !== 11) { this.changeState(10, this.def); return; }
        if ((cmd('holdfwd') || cmd('holdback')) && s !== 20 && !this.asserts.has('nowalk') && (s === 0 || s === 52)) { this.changeState(20, this.def); return; }
      } else if (this.stateType === 'C') {
        if (!cmd('holddown') && s === 11) { this.changeState(12, this.def); return; }
      }
    } else if (this.stateType === 'A') {
      // air jump
      const maxJ = this.const('movement.airjump.num', 1);
      if (s === 50 && this.airJumps < maxJ && this.history && this.pressedUp() && this.pos.y < -this.const('movement.airjump.height', 35) * this.lscl) {
        this.airJumps++;
        this.changeState(45, this.def);
      }
    }
  }
  private pressedUp(): boolean {
    const h = this.history;
    return (h.at(h.tick) & 1) !== 0 && (h.at(h.tick - 1) & 1) === 0;
  }

  // ------------------------------------------------------------ physics
  physicsStep() {
    if (this.posFreeze) { this.posFreeze = false; this.updateBind(); return; }
    if (this.bind) { this.updateBind(); return; }
    // movement
    this.pos.x += this.vel.x * this.facing;
    this.pos.y += this.vel.y;
    const ph = this.physics;
    if (ph === 'S' || ph === 'C') {
      const sliding = this.moveType === 'H' && this.slideTime > 0;
      if (!sliding) {
        const fr = ph === 'S' ? this.const('movement.stand.friction', 0.85) : this.const('movement.crouch.friction', 0.82);
        this.vel.x *= fr;
        if (Math.abs(this.vel.x) < 0.05) this.vel.x = 0;
      }
      if (this.pos.y > 0) this.pos.y = 0;
      if (this.stateType !== 'A') this.vel.y = 0;
    } else if (ph === 'A') {
      this.vel.y += this.const('movement.yaccel', 0.44) * this.lscl;
      if (this.pos.y >= 0 && this.vel.y > 0) {
        this.pos.y = 0;
        this.vel.y = 0;
        this.airJumps = 0;
        if (this.moveType !== 'H') this.changeState(52, this.def);
      }
    }
    if (this.slideTime > 0) this.slideTime--;
  }
  updateBind() {
    const b = this.bind;
    if (!b) return;
    if (b.to.destroyed || b.time === 0) { this.bind = null; return; }
    this.pos.x = b.to.pos.x + b.x * b.to.facing;
    this.pos.y = b.to.pos.y + b.y;
    if (b.time > 0) b.time--;
  }

  // ------------------------------------------------------------ boxes
  boxes(kind: 1 | 2): Box[] {
    const fr = this.frame();
    if (!fr) return [];
    const list = kind === 1 ? fr.clsn1 : fr.clsn2;
    if (!list.length) return list;
    const out: Box[] = [];
    const s = this.lscl * (this.drawScale[0] || 1);
    const sx = this.def.constants.get('size.xscale') ?? 1, sy = this.def.constants.get('size.yscale') ?? 1;
    for (const b of list) {
      let x1 = b[0] * sx * s, x2 = b[2] * sx * s;
      if (fr.flipH) { const t = -x2; x2 = -x1; x1 = t; }
      if (this.facing < 0) { const t = -x2; x2 = -x1; x1 = t; }
      out.push([this.pos.x + x1, this.pos.y + b[1] * sy * s, this.pos.x + x2, this.pos.y + b[3] * sy * s]);
    }
    return out;
  }

  resetGhv() { this.ghv = emptyGhv(); }
  comboCountReset() { /* combo display handled by HUD via enemy moveType */ }
}
