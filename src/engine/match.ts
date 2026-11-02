import type { Animation, Box } from '../mugen/air';
import { AnimPlayer } from './animplayer';
import type { CharacterDef, StateController } from './chardef';
import { Fighter, type HitDefData } from './fighter';
import { buildHitDef } from './hitdef';
import type { InputSource } from './input';

export interface StageInfo {
  name: string;
  /** camera bounds (engine units) */
  boundLeft: number; boundRight: number; boundHigh: number; boundLow: number;
  p1x: number; p2x: number;
  /** screen Y of ground (engine units from top, view height 240) */
  zoffset: number;
  verticalFollow: number;
  tension: number;
  /** draw callback layers */
  drawBack?: (ctx: CanvasRenderingContext2D, m: Match, scale: number) => void;
  drawFront?: (ctx: CanvasRenderingContext2D, m: Match, scale: number) => void;
  update?: (m: Match) => void;
  music?: string;
}

export interface FxBank { def: CharacterDef }

export interface Explod {
  owner: Fighter;
  id: number;
  player: AnimPlayer;
  def: CharacterDef; // for sprites
  pos: { x: number; y: number };
  vel: { x: number; y: number };
  accel: { x: number; y: number };
  facing: number;
  vfacing: number;
  bindTime: number;
  bindTo: Fighter | null;
  bindOff: { x: number; y: number };
  removeTime: number; // -2 anim end, -1 never
  time: number;
  sprPriority: number;
  onTop: boolean;
  under: boolean;
  scale: [number, number];
  trans: string;
  alpha: [number, number];
  removeOnGetHit: boolean;
  superMove: number;
  pauseMove: number;
  screenSpace: boolean;
  lscl: number;
}

export interface Projectile {
  owner: Fighter;
  id: number;
  player: AnimPlayer;
  def: CharacterDef;
  hitAnim: number; remAnim: number; cancelAnim: number;
  pos: { x: number; y: number };
  vel: { x: number; y: number };
  accel: { x: number; y: number };
  velMul: { x: number; y: number };
  remVel: { x: number; y: number };
  facing: number;
  hits: number;
  missTime: number;
  missCooldown: number;
  priority: number;
  sprPriority: number;
  edgeBound: number; stageBound: number;
  heightBound: [number, number];
  removeTime: number;
  time: number;
  hitdef: HitDefData;
  state: 'fly' | 'dying';
  guardDist: number;
  scale: [number, number];
  remove: boolean;
  hitTargets: Set<number>;
  lscl: number;
}

export interface AudioSink {
  play(f: Fighter | null, group: number, index: number, own: boolean, volume: number, channel: number, builtin?: string): void;
  stop(f: Fighter, channel: number): void;
}

export interface HitEvent { x: number; y: number; guarded: boolean; damage: number; attacker: Fighter; defender: Fighter; heavy: boolean }

export class Match {
  fighters: Fighter[] = [];
  roots: Fighter[] = [];
  explods: Explod[] = [];
  projectiles: Projectile[] = [];
  common: CharacterDef;
  fx: CharacterDef | null;
  stage: StageInfo;
  camera = { x: 0, y: 0 };
  viewW = 426;
  viewH = 240;
  gameTime = 0;
  roundState = 0;
  roundNo = 1;
  roundTime = 0; // ticks in current round state
  timer = 99 * 60;
  timerEnabled = true;
  wins = [0, 0];
  roundsToWin = 2;
  winnerSide = -1; // -1 none, 0/1 side, -2 draw
  matchOver = false;
  training = false;
  asserts = new Set<string>();
  pauseState: { by: Fighter; time: number; moveTime: number; super: boolean; darken: boolean } | null = null;
  shakeTime = 0; shakeAmpl = 0;
  flashState: { time: number; color: [number, number, number] } | null = null;
  slowMo = 0;
  audio: AudioSink | null = null;
  hitEvents: HitEvent[] = [];
  announce: { text: string; time: number; sub?: string } | null = null;
  koTime = 0;
  onEvent?: (type: string, data?: any) => void;
  rngState = 1;

  constructor(common: CharacterDef, fx: CharacterDef | null, stage: StageInfo) {
    this.common = common;
    this.fx = fx;
    this.stage = stage;
  }

  addPlayer(def: CharacterDef, side: number, input: InputSource | null, aiLevel = 0, palNo = 1): Fighter {
    const f = new Fighter(this, def, side, this.roots.length + 1);
    f.input = input;
    f.aiLevel = aiLevel;
    f.palNo = palNo;
    this.fighters.push(f);
    this.roots.push(f);
    return f;
  }

  byId(id: number) { return this.fighters.find((f) => f.id === id && !f.destroyed); }
  countHelpers(root: Fighter, id: number) { return this.fighters.filter((f) => f.isHelper && f.root === root && !f.destroyed && (id < 0 || f.helperId === id)).length; }
  countExplods(owner: Fighter, id: number) { return this.explods.filter((e) => e.owner === owner && (id < 0 || e.id === id)).length; }
  countProjs(owner: Fighter, id: number) { return this.projectiles.filter((p) => p.owner === owner && !p.remove && (id < 0 || p.id === id)).length; }
  groundScreenY() { return this.stage.zoffset; }

  // ------------------------------------------------------------ round flow
  startRound() {
    this.roundState = 0;
    this.roundTime = 0;
    this.timer = 99 * 60;
    this.winnerSide = -1;
    this.koTime = 0;
    this.slowMo = 0;
    this.explods = [];
    this.projectiles = [];
    this.pauseState = null;
    this.fighters = this.fighters.filter((f) => !f.isHelper);
    this.camera.x = 0; this.camera.y = 0;
    for (const f of this.roots) {
      f.destroyed = false;
      f.pos.x = f.side === 0 ? this.stage.p1x : this.stage.p2x;
      f.pos.y = 0;
      f.vel.x = f.vel.y = 0;
      f.facing = f.side === 0 ? 1 : -1;
      f.life = f.lifeMax;
      f.alive = true;
      if (this.roundNo === 1) f.power = 0;
      f.hitPause = f.hitShake = 0;
      f.targets = [];
      f.bind = null;
      f.hitdefActive = false;
      f.ctrl = false;
      f.stDef = f.def;
      f.resetGhv();
      f.sysvars.fill(0);
      f.changeState(5900, f.def);
      f.ctrl = false;
    }
    this.announce = { text: this.roundNo === 1 && this.wins[0] === 0 && this.wins[1] === 0 ? 'ROUND 1' : `ROUND ${this.roundNo}`, time: 70 };
    this.onEvent?.('round', this.roundNo);
  }

  // ------------------------------------------------------------ main tick
  tick() {
    if (this.slowMo > 0) {
      this.slowMo--;
      if (this.slowMo % 3 !== 0) { this.updateVisualTimers(); return; }
    }
    this.gameTime++;
    this.roundTime++;
    this.hitEvents.length = 0;
    this.roundLogic();

    // inputs
    for (const f of this.roots) {
      const bits = f.input ? f.input.poll() : 0;
      f.history.push(bits, f.facing);
      f.cmd.update(f.history);
    }

    const paused = this.pauseState;
    const isFrozenByPause = (f: Fighter) => {
      if (!paused) return false;
      if (f === paused.by || f.root === paused.by.root && f.isHelper && false) return paused.moveTime <= 0 ? true : false;
      return true;
    };

    const list = this.fighters.slice();
    for (const f of list) {
      if (f.destroyed) continue;
      if (isFrozenByPause(f)) {
        if (f === paused!.by && paused!.moveTime > 0) { /* unreachable */ }
        continue;
      }
      const frozen = f.hitPause > 0 || f.hitShake > 0;
      f.widthOverride = null;
      f.playerPush = true;
      if (!frozen) f.advanceAnim();
      f.runStates();
    }
    if (paused && paused.by && paused.moveTime > 0) paused.moveTime--;

    for (const f of list) {
      if (f.destroyed) continue;
      if (isFrozenByPause(f)) continue;
      if (f.hitPause > 0 || f.hitShake > 0) {
        if (f.hitPause > 0) f.hitPause--;
        if (f.hitShake > 0) f.hitShake--;
        continue;
      }
      f.physicsStep();
      f.stateTime++;
      if (f.moveType === 'H') { if (f.hitOverTime > 0) f.hitOverTime--; f.fallTime++; }
      if (f.moveContact) f.moveContact++;
      if (f.moveHit) f.moveHit++;
      if (f.moveGuarded) f.moveGuarded++;
      for (const n of f.notHitBy) n.time--;
      f.notHitBy = f.notHitBy.filter((n) => n.time > 0);
      for (const n of f.hitBy) n.time--;
      f.hitBy = f.hitBy.filter((n) => n.time > 0);
      if (f.palfx && f.palfx.time > 0 && --f.palfx.time <= 0) f.palfx = null;
      if (f.afterImage) {
        f.afterImage.trail.unshift({ x: f.pos.x, y: f.pos.y, frame: f.frame(), facing: f.facing });
        if (f.afterImage.trail.length > f.afterImage.length) f.afterImage.trail.length = f.afterImage.length;
        if (f.afterImage.time > 0 && --f.afterImage.time <= 0) f.afterImage = null;
      }
      if (f.moveType !== 'H') { f.comboCountReset(); }
    }

    this.pushAndBound();
    this.detectHits();
    this.updateProjectiles(!!paused);
    this.updateExplods(!!paused);
    this.updateCamera();
    this.updateVisualTimers();
    if (paused) { paused.time--; if (paused.time <= 0) this.pauseState = null; }
    // drop defeated targets
    for (const f of this.fighters) f.targets = f.targets.filter((t) => !t.f.destroyed && (t.f.moveType === 'H' || f.stateTime < 2 || f.moveContact > 0));
    this.stage.update?.(this);
  }

  updateVisualTimers() {
    if (this.shakeTime > 0) this.shakeTime--;
    if (this.flashState && --this.flashState.time <= 0) this.flashState = null;
    if (this.announce && --this.announce.time <= 0) this.announce = null;
  }

  roundLogic() {
    const rs = this.roundState;
    if (rs === 0 && this.roundTime > 40) { this.roundState = 1; this.roundTime = 0; }
    else if (rs === 1 && this.roundTime > 60) {
      this.roundState = 2; this.roundTime = 0;
      this.announce = { text: 'FIGHT!', time: 50 };
      this.onEvent?.('fight');
      for (const f of this.roots) { if (f.stateNo === 0 || f.stateNo === 5900) f.ctrl = true; }
    } else if (rs === 2) {
      if (this.timerEnabled && !this.training && !this.pauseState && !this.asserts.has('timerfreeze')) this.timer = Math.max(0, this.timer - 1);
      const dead = this.roots.filter((f) => f.life <= 0);
      if (this.training) {
        for (const f of this.roots) if (f.life <= 0 || (f.moveType !== 'H' && f.life < f.lifeMax && f.stateTime > 60)) { if (f.life <= 0) f.life = 1; }
      } else if (dead.length || this.timer <= 0) {
        this.roundState = 3; this.roundTime = 0;
        if (dead.length) {
          for (const f of dead) f.alive = false;
          this.winnerSide = dead.length === this.roots.length ? -2 : 1 - dead[0].side;
          this.slowMo = 75;
          this.announce = { text: 'K.O.', time: 110 };
          this.onEvent?.('ko');
        } else {
          const [a, b] = this.roots;
          const ra = a.life / a.lifeMax, rb = b.life / b.lifeMax;
          this.winnerSide = ra === rb ? -2 : ra > rb ? 0 : 1;
          this.announce = { text: 'TIME OVER', time: 110 };
        }
      }
    } else if (rs === 3) {
      const settled = this.roots.every((f) => f.alive ? (f.moveType !== 'H' && (f.ctrl || f.stateNo === 0)) : (f.stateNo === 5150 || f.stateNo === 5110 || this.roundTime > 200));
      if ((settled && this.roundTime > 90) || this.roundTime > 300) {
        this.roundState = 4; this.roundTime = 0;
        for (const f of this.roots) {
          if (!f.alive) continue;
          f.ctrl = false;
          if (this.winnerSide === f.side) f.changeState(180, f.def);
          else if (this.winnerSide === -2) f.changeState(175, f.def);
          else f.changeState(170, f.def);
        }
        if (this.winnerSide >= 0) this.wins[this.winnerSide]++;
        const w = this.winnerSide >= 0 ? this.roots.find((r) => r.side === this.winnerSide) : null;
        this.announce = { text: w ? `${w.def.displayName.toUpperCase()} WINS` : 'DRAW', time: 150 };
        this.onEvent?.('roundend', this.winnerSide);
      }
    } else if (rs === 4) {
      if (this.roundTime > 170) {
        if (this.wins[0] >= this.roundsToWin || this.wins[1] >= this.roundsToWin) {
          this.matchOver = true;
          this.onEvent?.('matchover', this.wins[0] > this.wins[1] ? 0 : 1);
          this.roundTime = -100000;
        } else {
          this.roundNo++;
          this.startRound();
        }
      }
    }
  }

  // ------------------------------------------------------------ space
  pushAndBound() {
    const roots = this.fighters.filter((f) => !f.destroyed && (!f.isHelper || f.helperType === 'player'));
    for (let i = 0; i < roots.length; i++) for (let j = i + 1; j < roots.length; j++) {
      const a = roots[i], b = roots[j];
      if (a.side === b.side || !a.playerPush || !b.playerPush) continue;
      if (a.stateType === 'L' || b.stateType === 'L') continue;
      // vertical overlap using heights
      const ha = a.const('size.height', 60) * a.lscl, hb = b.const('size.height', 60) * b.lscl;
      if (a.pos.y - ha > b.pos.y || b.pos.y - hb > a.pos.y) continue;
      const aRight = a.pos.x <= b.pos.x;
      const L = aRight ? a : b, R = aRight ? b : a;
      const lw = L.facing > 0 ? L.frontWidth() : L.backWidth();
      const rw = R.facing < 0 ? R.frontWidth() : R.backWidth();
      const overlap = L.pos.x + lw - (R.pos.x - rw);
      if (overlap > 0) {
        let lp = overlap / 2, rp = overlap / 2;
        if (L.stateType === 'A' && R.stateType !== 'A') { lp = overlap; rp = 0; }
        else if (R.stateType === 'A' && L.stateType !== 'A') { rp = overlap; lp = 0; }
        L.pos.x -= lp; R.pos.x += rp;
      }
    }
    const left = this.camera.x - this.viewW / 2, right = this.camera.x + this.viewW / 2;
    for (const f of this.fighters) {
      if (f.destroyed) continue;
      if (f.isHelper && f.helperType !== 'player') continue;
      if (!f.screenBound) { f.screenBound = true; continue; }
      const lw = f.facing > 0 ? f.backWidth() : f.frontWidth();
      const rw = f.facing > 0 ? f.frontWidth() : f.backWidth();
      const minX = Math.max(left, this.stage.boundLeft - this.viewW / 2) + lw;
      const maxX = Math.min(right, this.stage.boundRight + this.viewW / 2) - rw;
      if (f.pos.x < minX) f.pos.x = minX;
      if (f.pos.x > maxX) f.pos.x = maxX;
    }
  }

  updateCamera() {
    const rs = this.roots.filter((f) => !f.destroyed);
    if (!rs.length) return;
    const minX = Math.min(...rs.map((f) => f.pos.x)), maxX = Math.max(...rs.map((f) => f.pos.x));
    let target = (minX + maxX) / 2;
    const half = this.viewW / 2;
    target = Math.max(this.stage.boundLeft, Math.min(this.stage.boundRight, target));
    // keep both within tension margins
    const t = this.stage.tension;
    if (maxX - minX < this.viewW - 2 * t) {
      if (minX < target - half + t) target = minX + half - t;
      if (maxX > target + half - t) target = maxX - half + t;
    }
    target = Math.max(this.stage.boundLeft, Math.min(this.stage.boundRight, target));
    this.camera.x += (target - this.camera.x) * 0.5;
    const highest = Math.min(...rs.map((f) => f.pos.y));
    let ty = Math.min(0, highest * this.stage.verticalFollow);
    ty = Math.max(this.stage.boundHigh, Math.min(this.stage.boundLow, ty));
    this.camera.y += (ty - this.camera.y) * 0.3;
  }

  // ------------------------------------------------------------ hit system
  static overlap(a: Box[], b: Box[]): [number, number] | null {
    for (const x of a) for (const y of b) {
      if (x[0] <= y[2] && x[2] >= y[0] && x[1] <= y[3] && x[3] >= y[1]) {
        return [(Math.max(x[0], y[0]) + Math.min(x[2], y[2])) / 2, (Math.max(x[1], y[1]) + Math.min(x[3], y[3])) / 2];
      }
    }
    return null;
  }

  canBeHit(d: Fighter, hd: HitDefData, attackerSide: number): boolean {
    if (d.destroyed || d.side === attackerSide) return false;
    if (this.roundState !== 2 && !(this.roundState === 3 && d.alive)) return false;
    if (d.stateNo === 5150) return false;
    const st = d.stateType;
    const hf = hd.hitflag;
    if (hf.includes('+') && d.moveType !== 'H') return false;
    if (hf.includes('-') && d.moveType === 'H') return false;
    const falling = d.moveType === 'H' && d.ghv.fall && st === 'A';
    if (st === 'S' && !(hf.includes('H') || hf.includes('M'))) return false;
    if (st === 'C' && !(hf.includes('L') || hf.includes('M'))) return false;
    if (st === 'A' && !hf.includes('A') && !(falling && hf.includes('F'))) return false;
    if (falling && !hf.includes('F')) return false;
    if (st === 'L' && !hf.includes('D')) return false;
    const attrMatch = (spec: string) => {
      const [s, ...k] = spec.replace(/\s/g, '').split(',');
      if (!s.includes(hd.attrState.replace(/[^SCA]/g, '')[0] ?? 'S')) return false;
      if (!k.length) return true;
      return hd.attrKind.some((ak) => k.some((kk) => kk === ak || (kk.length === 1 && ak.startsWith(kk)) || (kk[0] === 'A' && kk[1] === ak[1]) || (kk[1] === 'A' && false)));
    };
    for (const n of d.notHitBy) if (attrMatch(n.attr)) return false;
    for (const n of d.hitBy) if (!attrMatch(n.attr)) return false;
    return true;
  }

  isGuarding(d: Fighter, hd: HitDefData): boolean {
    if (d.asserts.has('unguardable') || !hd.guardflag) return false;
    const st = d.stateType;
    const gf = hd.guardflag;
    const ok = (st === 'S' && (gf.includes('H') || gf.includes('M'))) || (st === 'C' && (gf.includes('L') || gf.includes('M'))) || (st === 'A' && gf.includes('A'));
    if (!ok) return false;
    if (d.stateNo >= 120 && d.stateNo <= 155) return true;
    if (d.ctrl && d.moveType !== 'H' && d.hasCommand('holdback') && st !== 'A' && !d.isHelper) return true;
    return false;
  }

  detectHits() {
    const pending: { a: Fighter; d: Fighter; hd: HitDefData; at: [number, number] }[] = [];
    for (const a of this.fighters) {
      if (a.destroyed || !a.hitdefActive || !a.hitdef) continue;
      if (a.moveType !== 'A' && !a.isHelper) continue;
      const c1 = a.boxes(1);
      if (!c1.length) continue;
      for (const d of this.fighters) {
        if (d === a || d.root === a.root || a.hitdefTargets.has(d.id)) continue;
        if (!this.canBeHit(d, a.hitdef, a.side)) continue;
        const at = Match.overlap(c1, d.boxes(2));
        if (!at) continue;
        pending.push({ a, d, hd: a.hitdef, at });
      }
    }
    for (const p of pending) {
      p.a.hitdefTargets.add(p.d.id);
      this.applyHit(p.a, p.d, p.hd, p.at, null);
    }
  }

  applyHit(a: Fighter, d: Fighter, hd: HitDefData, at: [number, number], proj: Projectile | null) {
    const guarded = this.isGuarding(d, hd);
    const away = d.facing === -a.facing || d.pos.x === a.pos.x ? 1 : -1;
    const fromProj = !!proj;
    if (!fromProj) { a.moveContact = 1; if (!a.targets.some((t) => t.f === d)) a.targets.push({ f: d, hitId: hd.id }); }
    d.lastHitBy = a;
    if (d.bind && d.bind.kind === 'target') d.bind = null;
    // P2 faces attacker
    if (!fromProj && hd.p2facing === 0 && d.stateType !== 'A') {
      // keep facing attacker for a natural knockback
      d.facing = a.pos.x >= d.pos.x ? 1 : -1;
    } else if (hd.p2facing === 1) d.facing = -a.facing; else if (hd.p2facing === -1) d.facing = a.facing;
    const dir = d.facing === -(fromProj ? proj!.facing : a.facing) ? 1 : -1; // velocity sign correction
    d.hitdefActive = false;
    const g = d.ghv;
    const wasHit = d.moveType === 'H';
    const sparkX = at[0] + hd.sparkX * (fromProj ? proj!.facing : a.facing) * 0;
    const sparkY = at[1];
    if (guarded) {
      const dmg = Math.round(hd.guardDamage * a.root.attackMul);
      d.life = Math.max(hd.guardKill ? 0 : Math.min(d.life, 1), d.life - dmg);
      if (!fromProj) a.moveGuarded = 1; else a.projContacts.set(`guarded:${proj!.id}`, this.gameTime);
      d.resetGhv();
      g.guarded = true; d.ghv.guarded = true;
      d.ghv.hittime = hd.guardHitTime; d.ghv.slidetime = hd.guardSlideTime; d.ghv.ctrltime = hd.guardCtrlTime;
      d.ghv.xvel = (d.stateType === 'A' ? hd.airGuardVel[0] : hd.guardVel) * dir;
      d.ghv.yvel = d.stateType === 'A' ? hd.airGuardVel[1] : 0;
      d.ghv.yaccel = hd.yaccel;
      d.ghv.damage = dmg;
      d.ghv.guardcount++;
      d.hitShake = hd.guardPauseP2;
      if (!fromProj) a.hitPause = hd.guardPauseP1;
      d.hitOverTime = hd.guardHitTime;
      d.slideTime = hd.guardSlideTime;
      d.ctrl = false;
      d.moveType = 'H';
      d.changeState(d.stateType === 'A' ? 154 : d.stateType === 'C' ? 152 : 150, d.def);
      a.addPower(hd.guardGetPower); d.addPower(hd.guardGivePower);
      this.spawnSpark(a, hd.guardSparkNo, hd.guardSparkOwn, sparkX, sparkY, true);
      if (hd.guardSound) this.playSound(a, hd.guardSound[0], hd.guardSound[1], hd.guardSound[2], 1, -1, 'guard');
    } else {
      const dmg = Math.round(hd.damage * a.root.attackMul / Math.max(0.01, d.defenceMul));
      let nl = d.life - dmg;
      if (!hd.kill && nl < 1) nl = Math.min(d.life, 1);
      d.life = Math.max(0, nl);
      if (!fromProj) { a.moveHit = 1; a.hitCount += hd.numHits; } else a.projContacts.set(`hit:${proj!.id}`, this.gameTime);
      const air = d.stateType === 'A' || (hd.groundVel[1] !== 0 && !hd.forcestand && false);
      const prevHitcount = wasHit ? g.hitcount : 0;
      d.resetGhv();
      const ng = d.ghv;
      ng.attackerId = a.root.id; ng.attackerNo = a.root.playerNo;
      ng.damage = dmg;
      ng.hitcount = prevHitcount + 1;
      ng.animtype = air ? hd.airAnimtype : hd.animtype;
      ng.groundtype = hd.groundType; ng.airtype = hd.airType;
      ng.type = air ? 3 : hd.groundType;
      ng.fall = air ? hd.airFall : hd.fall;
      if (wasHit && d.ghv.fall) ng.fall = true;
      ng.hittime = air ? hd.airHitTime : hd.groundHitTime;
      ng.slidetime = hd.groundSlideTime;
      ng.ctrltime = hd.groundHitTime;
      const v = air ? hd.airVel : hd.groundVel;
      ng.xvel = v[0] * dir; ng.yvel = v[1];
      ng.yaccel = hd.yaccel;
      ng.fallXvel = hd.fallXvel !== null ? hd.fallXvel * dir : ng.xvel;
      ng.fallYvel = hd.fallYvel;
      ng.fallRecover = hd.fallRecover; ng.fallRecoverTime = hd.fallRecoverTime;
      ng.fallDamage = hd.fallDamage; ng.fallKill = hd.fallKill; ng.kill = hd.kill;
      ng.chainid = hd.chainId;
      if (d.life <= 0) {
        ng.fall = true;
        if (ng.yvel === 0) { ng.yvel = -5 * d.lscl; ng.xvel = Math.min(ng.xvel, -2.5 * d.lscl); }
        d.alive = false;
      }
      if (ng.fall) ng.animtype = Math.max(ng.animtype, hd.fallAnimtype >= 3 ? ng.animtype : ng.animtype);
      d.hitShake = hd.pauseP2;
      if (!fromProj) a.hitPause = hd.pauseP1;
      d.hitOverTime = ng.hittime;
      d.slideTime = ng.slidetime;
      d.fallTime = 0;
      d.ctrl = false;
      d.moveType = 'H';
      if (hd.p2stateno >= 0) d.changeState(hd.p2stateno, hd.p2getp1state ? a.def : d.def);
      else if (d.stateType === 'A' || d.stateType === 'L') d.changeState(5020, d.def);
      else if (d.stateType === 'C' && !hd.forcestand) d.changeState(5010, d.def);
      else d.changeState(5000, d.def);
      if (hd.p1stateno >= 0 && !fromProj) a.changeState(hd.p1stateno, a.def);
      a.addPower(hd.getPower); d.addPower(hd.givePower);
      a.root.comboCount = prevHitcount + 1;
      a.root.comboDamage = (wasHit ? a.root.comboDamage : 0) + dmg;
      d.sprPriority = hd.p2sprpriority;
      if (!fromProj) a.sprPriority = hd.p1sprpriority;
      this.spawnSpark(a, hd.sparkNo, hd.sparkOwn, sparkX, sparkY, false, hd.animtype >= 2 || dmg >= 70);
      if (hd.hitSound) this.playSound(a, hd.hitSound[0], hd.hitSound[1], hd.hitSound[2], 1, -1, dmg >= 70 ? 'hitheavy' : 'hit');
      if (hd.envShakeTime) this.shake(hd.envShakeTime, 4);
      for (const e of this.explods) if (e.owner.root === d.root && e.removeOnGetHit) e.removeTime = 0;
    }
    this.hitEvents.push({ x: at[0], y: at[1], guarded, damage: d.ghv.damage, attacker: a, defender: d, heavy: hd.animtype >= 2 });
    this.onEvent?.('hit', { guarded, attacker: a, defender: d });
  }

  // ------------------------------------------------------------ effects
  playSound(f: Fighter | null, g: number, i: number, own: boolean, vol: number, ch: number, builtin?: string) {
    this.audio?.play(f, g, i, own, vol, ch, builtin);
  }
  stopSound(f: Fighter, ch: number) { this.audio?.stop(f, ch); }
  shake(time: number, ampl: number) { this.shakeTime = Math.max(this.shakeTime, time); this.shakeAmpl = Math.abs(ampl) || 4; }
  flash(time: number, color: [number, number, number]) { this.flashState = { time, color }; }
  pause(by: Fighter, time: number, moveTime: number, isSuper: boolean, darken = true) {
    this.pauseState = { by, time, moveTime, super: isSuper, darken: isSuper && darken };
  }

  private makeExplod(owner: Fighter, def: CharacterDef, anim: Animation, x: number, y: number): Explod {
    return {
      owner, id: -1, player: new AnimPlayer(anim), def, pos: { x, y }, vel: { x: 0, y: 0 }, accel: { x: 0, y: 0 },
      facing: owner.facing, vfacing: 1, bindTime: 0, bindTo: null, bindOff: { x: 0, y: 0 }, removeTime: -2, time: 0,
      sprPriority: 5, onTop: true, under: false, scale: [1, 1], trans: 'default', alpha: [256, 0], removeOnGetHit: false,
      superMove: 0, pauseMove: 0, screenSpace: false, lscl: owner.lscl,
    };
  }

  spawnSpark(a: Fighter, no: number, own: boolean, x: number, y: number, guard: boolean, heavy = false) {
    if (own) {
      const anim = a.def.anims.get(no);
      if (anim) { const e = this.makeExplod(a, a.def, anim, x, y); this.explods.push(e); return; }
    }
    if (!this.fx) return;
    // map common fightfx numbers to builtin sparks
    let n = guard ? 40 : heavy ? 2 : no;
    if (!this.fx.anims.has(n)) n = guard ? 40 : heavy ? 2 : 0;
    const anim = this.fx.anims.get(n);
    if (!anim) return;
    const e = this.makeExplod(a, this.fx, anim, x, y);
    e.lscl = 1;
    e.facing = a.facing;
    this.explods.push(e);
  }
  spawnFx(f: Fighter, no: number, x: number, y: number, under: boolean) {
    if (!this.fx) return;
    const anim = this.fx.anims.get(no) ?? this.fx.anims.get(0);
    if (!anim) return;
    const e = this.makeExplod(f, this.fx, anim, x, y);
    e.under = under; e.onTop = !under; e.lscl = 1;
    this.explods.push(e);
  }
  spawnSuperFx(f: Fighter, ox: number, oy: number, ownAnim: number) {
    const def = ownAnim >= 0 ? f.def : this.fx;
    if (!def) return;
    const anim = def.anims.get(ownAnim >= 0 ? ownAnim : 30);
    if (!anim) return;
    const e = this.makeExplod(f, def, anim, f.pos.x + ox * f.facing, f.pos.y + oy);
    e.superMove = 9999; e.lscl = ownAnim >= 0 ? f.lscl : 1;
    this.explods.push(e);
  }
  spawnDust(x: number, y: number) {
    if (!this.fx) return;
    const anim = this.fx.anims.get(120);
    if (!anim || !this.roots[0]) return;
    const e = this.makeExplod(this.roots[0], this.fx, anim, x, y);
    e.lscl = 1; e.onTop = false; e.sprPriority = -3;
    this.explods.push(e);
  }

  spawnExplod(f: Fighter, sc: StateController, num: (k: string, d?: number, i?: number) => number, has: (k: string) => boolean, raw: (k: string) => string | undefined) {
    const animRaw = raw('anim') ?? '0';
    const fromFx = /^f/i.test(animRaw);
    const animNo = parseInt(animRaw.replace(/^[fs]/i, '')) || num('anim', 0);
    const def = fromFx ? this.fx : f.def;
    if (!def) return;
    const anim = def.anims.get(animNo);
    if (!anim) return;
    const postype = (raw('postype') ?? 'p1').toLowerCase();
    const px = num('pos', 0, 0) * f.lscl, py = num('pos', 0, 1) * f.lscl;
    const fac = num('facing', 1) < 0 ? -1 : 1;
    let x = f.pos.x, y = f.pos.y, facing = f.facing * fac;
    let bindTo: Fighter | null = null;
    let screen = false;
    if (postype === 'p1') { x = f.pos.x + px * f.facing; y = f.pos.y + py; bindTo = f; }
    else if (postype === 'p2') { const e = f.enemy(0) ?? f; x = e.pos.x + px * e.facing; y = e.pos.y + py; bindTo = e; facing = e.facing * fac; }
    else if (postype === 'front') { x = this.camera.x + (f.facing > 0 ? this.viewW / 2 : -this.viewW / 2) + px * f.facing; y = py + this.camera.y; }
    else if (postype === 'back') { x = this.camera.x + (f.facing > 0 ? -this.viewW / 2 : this.viewW / 2) + px * f.facing; y = py + this.camera.y; }
    else if (postype === 'left') { x = this.camera.x - this.viewW / 2 + px; y = py + this.camera.y; facing = fac; screen = false; }
    else if (postype === 'right') { x = this.camera.x + this.viewW / 2 + px; y = py + this.camera.y; facing = fac; }
    if (has('random')) { x += (Math.random() - 0.5) * num('random', 0, 0) * f.lscl; y += (Math.random() - 0.5) * num('random', 0, 1) * f.lscl; }
    const e = this.makeExplod(f, def, anim, x, y);
    e.id = num('id', -1);
    e.facing = facing;
    e.vfacing = num('vfacing', 1) < 0 ? -1 : 1;
    e.vel = { x: num('vel', 0, 0) * f.lscl, y: num('vel', 0, 1) * f.lscl };
    e.accel = { x: num('accel', 0, 0) * f.lscl, y: num('accel', 0, 1) * f.lscl };
    e.bindTime = num('bindtime', 0);
    e.bindTo = e.bindTime !== 0 ? bindTo : null;
    if (e.bindTo) e.bindOff = { x: px, y: py };
    e.removeTime = num('removetime', -2);
    e.sprPriority = num('sprpriority', 0);
    e.onTop = num('ontop', 0) !== 0;
    e.under = num('under', 0) !== 0;
    e.scale = [num('scale', 1, 0), num('scale', 1, 1)];
    e.removeOnGetHit = num('removeongethit', 0) !== 0;
    e.superMove = num('supermove', num('supermovetime', 0));
    e.pauseMove = num('pausemovetime', 0);
    e.screenSpace = screen;
    e.lscl = fromFx ? 1 : f.lscl;
    const tr = (raw('trans') ?? 'default').toLowerCase();
    e.trans = tr;
    if (has('alpha')) e.alpha = [num('alpha', 256, 0), num('alpha', 0, 1)];
    this.explods.push(e);
  }
  removeExplods(f: Fighter, id: number) { this.explods = this.explods.filter((e) => !(e.owner === f && (id < 0 || e.id === id))); }
  modifyExplods(f: Fighter, id: number, fn: (e: Explod) => void) { for (const e of this.explods) if (e.owner === f && (id < 0 || e.id === id)) fn(e); }

  updateExplods(paused: boolean) {
    const ps = this.pauseState;
    this.explods = this.explods.filter((e) => {
      if (paused && ps) {
        const allowed = ps.super ? e.superMove > 0 : e.pauseMove > 0;
        if (!allowed) return true;
        if (ps.super) e.superMove--; else e.pauseMove--;
      }
      if (e.owner.hitPause > 0 && e.bindTo === e.owner) return true;
      e.time++;
      e.player.advance();
      if (e.bindTo && (e.bindTime > 0 || e.bindTime === -1)) {
        e.pos.x = e.bindTo.pos.x + e.bindOff.x * e.bindTo.facing;
        e.pos.y = e.bindTo.pos.y + e.bindOff.y;
        if (e.bindTime > 0) e.bindTime--;
      } else {
        e.pos.x += e.vel.x * e.facing;
        e.pos.y += e.vel.y;
        e.vel.x += e.accel.x; e.vel.y += e.accel.y;
      }
      if (e.removeTime === -2) return !e.player.done();
      if (e.removeTime === -1) return true;
      return e.time < e.removeTime;
    });
  }

  spawnHelper(f: Fighter, sc: StateController, num: (k: string, d?: number, i?: number) => number, has: (k: string) => boolean, raw: (k: string) => string | undefined) {
    if (this.fighters.length > 60) return;
    const h = new Fighter(this, f.root.def, f.side, f.playerNo);
    h.isHelper = true;
    h.helperId = num('id', 0);
    h.parent = f;
    h.root = f.root;
    h.keyCtrl = num('keyctrl', 0) !== 0;
    h.helperType = (raw('helpertype') ?? 'normal').toLowerCase() === 'player' ? 'player' : 'normal';
    h.history = f.root.history;
    h.cmd = f.root.cmd;
    h.palNo = f.palNo;
    h.aiLevel = f.root.aiLevel;
    h.life = h.lifeMax = f.root.lifeMax;
    const postype = (raw('postype') ?? 'p1').toLowerCase();
    const px = num('pos', 0, 0) * f.lscl, py = num('pos', 0, 1) * f.lscl;
    const fac = num('facing', 1) < 0 ? -1 : 1;
    h.facing = f.facing * fac;
    if (postype === 'p2') { const e = f.enemy(0) ?? f; h.pos = { x: e.pos.x + px * e.facing, y: e.pos.y + py }; }
    else if (postype === 'front' || postype === 'back' || postype === 'left' || postype === 'right') {
      const edge = postype === 'left' ? -1 : postype === 'right' ? 1 : (postype === 'front' ? f.facing : -f.facing);
      h.pos = { x: this.camera.x + edge * this.viewW / 2 + px, y: py };
    } else h.pos = { x: f.pos.x + px * f.facing, y: f.pos.y + py };
    h.stateNo = -1;
    h.ctrl = false;
    this.fighters.push(h);
    h.changeState(num('stateno', 0) | 0, f.stDef === f.def ? h.def : f.stDef);
  }
  destroyHelper(f: Fighter) {
    if (!f.isHelper) return;
    f.destroyed = true;
    this.fighters = this.fighters.filter((x) => x !== f);
    for (const e of this.explods) if (e.bindTo === f) e.bindTo = null;
  }

  spawnProjectile(f: Fighter, sc: StateController, num: (k: string, d?: number, i?: number) => number, has: (k: string, i?: number) => boolean, raw: (k: string) => string | undefined) {
    const animNo = num('projanim', 0) | 0;
    const anim = f.def.anims.get(animNo);
    if (!anim) return;
    const hd = buildHitDef(f, sc);
    const postype = (raw('postype') ?? 'p1').toLowerCase();
    const ox = num('offset', 0, 0) * f.lscl, oy = num('offset', 0, 1) * f.lscl;
    let x = f.pos.x + ox * f.facing, y = f.pos.y + oy;
    if (postype === 'p2') { const e = f.enemy(0) ?? f; x = e.pos.x + ox * f.facing; y = e.pos.y + oy; }
    const p: Projectile = {
      owner: f, id: num('projid', 0) | 0, player: new AnimPlayer(anim), def: f.def,
      hitAnim: has('projhitanim') ? num('projhitanim', -1) | 0 : -1,
      remAnim: has('projremanim') ? num('projremanim', -1) | 0 : (has('projhitanim') ? num('projhitanim', -1) | 0 : -1),
      cancelAnim: has('projcancelanim') ? num('projcancelanim', -1) | 0 : -1,
      pos: { x, y },
      vel: { x: num('velocity', 0, 0) * f.lscl, y: num('velocity', 0, 1) * f.lscl },
      accel: { x: num('accel', 0, 0) * f.lscl, y: num('accel', 0, 1) * f.lscl },
      velMul: { x: num('velmul', 1, 0), y: num('velmul', 1, 1) },
      remVel: { x: num('remvelocity', 0, 0) * f.lscl, y: num('remvelocity', 0, 1) * f.lscl },
      facing: f.facing,
      hits: num('projhits', 1) | 0,
      missTime: num('projmisstime', 0) | 0,
      missCooldown: 0,
      priority: num('projpriority', 1) | 0,
      sprPriority: num('projsprpriority', 3) | 0,
      edgeBound: num('projedgebound', 40) * f.lscl,
      stageBound: num('projstagebound', 40) * f.lscl,
      heightBound: [num('projheightbound', -240, 0) * f.lscl, num('projheightbound', 1, 1) * f.lscl],
      removeTime: num('projremovetime', -1) | 0,
      time: 0, hitdef: hd, state: 'fly', guardDist: hd.guardDist * f.lscl,
      scale: [num('projscale', 1, 0), num('projscale', 1, 1)], remove: false, hitTargets: new Set(), lscl: f.lscl,
    };
    this.projectiles.push(p);
  }

  updateProjectiles(paused: boolean) {
    if (paused) return;
    for (const p of this.projectiles) {
      p.player.advance();
      p.time++;
      if (p.state === 'dying') {
        p.pos.x += p.remVel.x * p.facing; p.pos.y += p.remVel.y;
        if (p.player.done() || p.player.anim.totalTime < 0) p.remove = true;
        continue;
      }
      if (p.owner.hitPause > 0 && false) continue;
      p.pos.x += p.vel.x * p.facing; p.pos.y += p.vel.y;
      p.vel.x = p.vel.x * p.velMul.x + p.accel.x; p.vel.y = p.vel.y * p.velMul.y + p.accel.y;
      if (p.missCooldown > 0) p.missCooldown--;
      const left = this.camera.x - this.viewW / 2, right = this.camera.x + this.viewW / 2;
      if (p.pos.x < left - p.edgeBound || p.pos.x > right + p.edgeBound) p.remove = true;
      if (p.pos.y < p.heightBound[0] - 240 || p.pos.y > p.heightBound[1] + 20) p.remove = true;
      if (p.removeTime >= 0 && p.time >= p.removeTime) this.killProj(p, p.remAnim);
    }
    // proj vs proj
    for (const a of this.projectiles) for (const b of this.projectiles) {
      if (a === b || a.state !== 'fly' || b.state !== 'fly' || a.owner.side === b.owner.side) continue;
      if (Match.overlap(this.projBoxes(a, 1).concat(this.projBoxes(a, 2)), this.projBoxes(b, 1).concat(this.projBoxes(b, 2)))) {
        if (a.priority <= b.priority) this.killProj(a, a.cancelAnim >= 0 ? a.cancelAnim : a.remAnim);
        if (b.priority <= a.priority) this.killProj(b, b.cancelAnim >= 0 ? b.cancelAnim : b.remAnim);
      }
    }
    // proj vs fighters
    for (const p of this.projectiles) {
      if (p.state !== 'fly' || p.missCooldown > 0) continue;
      const c1 = this.projBoxes(p, 1);
      if (!c1.length) continue;
      for (const d of this.fighters) {
        if (d.root === p.owner.root || p.state !== 'fly') continue;
        if (!this.canBeHit(d, p.hitdef, p.owner.side)) continue;
        const at = Match.overlap(c1, d.boxes(2));
        if (!at) continue;
        this.applyHit(p.owner, d, p.hitdef, at, p);
        p.owner.projContacts.set(`contact:${p.id}`, this.gameTime);
        p.hits--;
        if (p.hits <= 0) this.killProj(p, p.hitAnim);
        else p.missCooldown = Math.max(1, p.missTime);
      }
    }
    this.projectiles = this.projectiles.filter((p) => !p.remove);
  }
  killProj(p: Projectile, anim: number) {
    const a = anim >= 0 ? p.def.anims.get(anim) : undefined;
    if (a) { p.player = new AnimPlayer(a); p.state = 'dying'; }
    else p.remove = true;
  }
  projBoxes(p: Projectile, kind: 1 | 2): Box[] {
    const fr = p.player.frame();
    if (!fr) return [];
    const list = kind === 1 ? fr.clsn1 : fr.clsn2;
    return list.map((b) => {
      let x1 = b[0] * p.lscl * p.scale[0], x2 = b[2] * p.lscl * p.scale[0];
      if (p.facing < 0) { const t = -x2; x2 = -x1; x1 = t; }
      return [p.pos.x + x1, p.pos.y + b[1] * p.lscl * p.scale[1], p.pos.x + x2, p.pos.y + b[3] * p.lscl * p.scale[1]] as Box;
    });
  }
}
