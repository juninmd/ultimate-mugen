import { compileParam, type StateController } from './chardef';
import type { Fighter } from './fighter';
import { buildHitDef, parseSoundRef } from './hitdef';
import { resolver } from './triggers';

type Ctl = (f: Fighter, sc: StateController) => void;

function P(f: Fighter, sc: StateController, k: string) {
  const p = sc.params.get(k);
  return p ? compileParam<Fighter>(p, resolver) : undefined;
}
function num(f: Fighter, sc: StateController, k: string, d = 0, i = 0): number {
  const c = P(f, sc, k);
  return c && c[i] ? c[i].f(f) : d;
}
function has(sc: StateController, k: string, i = 0) {
  const p = sc.params.get(k);
  return !!p && !!p.nodes && p.nodes.length > i;
}
function raw(sc: StateController, k: string) { return sc.params.get(k)?.raw.trim(); }

function setVars(f: Fighter, sc: StateController, add: boolean, target: Fighter = f) {
  // forms: v = n, value = x  |  fv = n, value = x  | var(n) = x | fvar(n) = x | sysvar(n) = x
  if (has(sc, 'v') || has(sc, 'fv')) {
    const isF = has(sc, 'fv');
    const idx = num(f, sc, isF ? 'fv' : 'v') | 0;
    const val = num(f, sc, 'value');
    if (isF) target.fvars[idx] = add ? target.fvars[idx] + val : val;
    else target.vars[idx] = add ? target.vars[idx] + (val | 0) : val | 0;
    return;
  }
  for (const [k] of sc.params) {
    const m = /^(sys)?(f)?var\((\d+)\)$/.exec(k);
    if (!m) continue;
    const idx = parseInt(m[3]);
    const val = num(f, sc, k);
    const arr: Int32Array | Float64Array = m[1] ? (m[2] ? target.sysfvars : target.sysvars) : (m[2] ? target.fvars : target.vars);
    if (idx < 0 || idx >= arr.length) continue;
    arr[idx] = add ? arr[idx] + val : val;
  }
}

function lifeAdd(t: Fighter, v: number, kill: boolean, absolute: boolean, attacker?: Fighter) {
  let dv = v;
  if (!absolute && v < 0) dv = v / t.defenceMul;
  let nl = t.life + Math.trunc(dv);
  if (!kill && nl < 1 && t.life > 0) nl = 1;
  t.life = Math.max(0, Math.min(t.lifeMax, nl));
  if (attacker && v < 0) t.lastHitBy = attacker;
}

const C: Record<string, Ctl> = {
  null: () => {},
  changestate(f, sc) {
    const v = num(f, sc, 'value') | 0;
    if (has(sc, 'ctrl')) f.ctrl = num(f, sc, 'ctrl') !== 0;
    f.changeState(v, f.stDef);
    if (has(sc, 'anim')) f.changeAnim(num(f, sc, 'anim') | 0, f.def);
  },
  selfstate(f, sc) {
    const v = num(f, sc, 'value') | 0;
    if (has(sc, 'ctrl')) f.ctrl = num(f, sc, 'ctrl') !== 0;
    f.changeState(v, f.def);
    if (has(sc, 'anim')) f.changeAnim(num(f, sc, 'anim') | 0, f.def);
  },
  targetstate(f, sc) {
    const id = has(sc, 'id') ? num(f, sc, 'id') : -1;
    const v = num(f, sc, 'value') | 0;
    for (const t of f.targets) if (id < 0 || t.hitId === id) { t.f.changeState(v, f.def); }
  },
  changeanim(f, sc) { f.changeAnim(num(f, sc, 'value') | 0, f.def, has(sc, 'elem') ? num(f, sc, 'elem') | 0 : 1); },
  changeanim2(f, sc) { f.changeAnim(num(f, sc, 'value') | 0, f.stDef, has(sc, 'elem') ? num(f, sc, 'elem') | 0 : 1); },
  velset(f, sc) { if (has(sc, 'x')) f.vel.x = num(f, sc, 'x') * f.lscl; if (has(sc, 'y')) f.vel.y = num(f, sc, 'y') * f.lscl; },
  veladd(f, sc) { if (has(sc, 'x')) f.vel.x += num(f, sc, 'x') * f.lscl; if (has(sc, 'y')) f.vel.y += num(f, sc, 'y') * f.lscl; },
  velmul(f, sc) { if (has(sc, 'x')) f.vel.x *= num(f, sc, 'x', 1); if (has(sc, 'y')) f.vel.y *= num(f, sc, 'y', 1); },
  posset(f, sc) {
    if (has(sc, 'x')) f.pos.x = f.match.camera.x + num(f, sc, 'x') * f.lscl;
    if (has(sc, 'y')) f.pos.y = num(f, sc, 'y') * f.lscl;
  },
  posadd(f, sc) { if (has(sc, 'x')) f.pos.x += num(f, sc, 'x') * f.lscl * f.facing; if (has(sc, 'y')) f.pos.y += num(f, sc, 'y') * f.lscl; },
  posfreeze(f, sc) { f.posFreeze = num(f, sc, 'value', 1) !== 0; },
  gravity(f) { f.vel.y += f.const('movement.yaccel', 0.44) * f.lscl; },
  ctrlset(f, sc) { f.ctrl = num(f, sc, 'value') !== 0; },
  statetypeset(f, sc) {
    const st = raw(sc, 'statetype'), mt = raw(sc, 'movetype'), ph = raw(sc, 'physics');
    if (st) f.stateType = st[0].toUpperCase();
    if (mt) f.moveType = mt[0].toUpperCase();
    if (ph) f.physics = ph[0].toUpperCase();
  },
  turn(f) { f.facing = -f.facing; },
  hitdef(f, sc) {
    f.hitdef = buildHitDef(f, sc);
    f.hitdefActive = true;
    f.hitdefTargets.clear();
  },
  reversaldef() {},
  hitvelset(f, sc) {
    if (num(f, sc, 'x', 1)) f.vel.x = f.ghv.xvel;
    if (num(f, sc, 'y', 1)) f.vel.y = f.ghv.yvel;
  },
  hitfallvel(f) {
    if (!f.ghv.fall) return;
    if (f.ghv.fallXvel !== 0) f.vel.x = f.ghv.fallXvel;
    f.vel.y = f.ghv.fallYvel;
  },
  hitfallset(f, sc) {
    const v = num(f, sc, 'value', -1);
    if (v >= 0) f.ghv.fall = v !== 0;
    if (has(sc, 'xvel')) f.ghv.fallXvel = num(f, sc, 'xvel') * f.lscl;
    if (has(sc, 'yvel')) f.ghv.fallYvel = num(f, sc, 'yvel') * f.lscl;
  },
  hitfalldamage(f) { lifeAdd(f, -f.ghv.fallDamage, f.ghv.fallKill, true); f.ghv.fallDamage = 0; },
  playsnd(f, sc) {
    const r = parseSoundRef(raw(sc, 'value'), true);
    if (!r) return;
    // in MUGEN "F" prefix = common sounds; no prefix = own
    const own = !/^f/i.test(raw(sc, 'value') ?? '');
    f.match.playSound(f, r[0], r[1], own, num(f, sc, 'volumescale', 100) / 100, num(f, sc, 'channel', -1));
  },
  stopsnd(f, sc) { f.match.stopSound(f, num(f, sc, 'channel', -1)); },
  sndpan() {},
  varset(f, sc) { setVars(f, sc, false); },
  varadd(f, sc) { setVars(f, sc, true); },
  parentvarset(f, sc) { if (f.parent) setVars(f, sc, false, f.parent); },
  parentvaradd(f, sc) { if (f.parent) setVars(f, sc, true, f.parent); },
  varrandom(f, sc) {
    const idx = num(f, sc, 'v') | 0;
    const r = P(f, sc, 'range');
    let lo = 0, hi = 1000;
    if (r && r.length === 1) hi = r[0].f(f);
    else if (r && r.length >= 2) { lo = r[0].f(f); hi = r[1].f(f); }
    f.vars[idx] = lo + Math.floor(f.match.random() * (hi - lo + 1));
  },
  varrangeset(f, sc) {
    const first = num(f, sc, 'first', 0) | 0;
    if (has(sc, 'fvalue')) {
      const last = num(f, sc, 'last', 39) | 0, v = num(f, sc, 'fvalue');
      for (let i = first; i <= last && i < 40; i++) f.fvars[i] = v;
    } else {
      const last = num(f, sc, 'last', 59) | 0, v = num(f, sc, 'value') | 0;
      for (let i = first; i <= last && i < 60; i++) f.vars[i] = v;
    }
  },
  poweradd(f, sc) { f.addPower(num(f, sc, 'value')); },
  powerset(f, sc) { f.root.power = Math.max(0, Math.min(f.root.powerMax, num(f, sc, 'value'))); },
  lifeadd(f, sc) { lifeAdd(f, num(f, sc, 'value'), num(f, sc, 'kill', 1) !== 0, num(f, sc, 'absolute', 0) !== 0); },
  lifeset(f, sc) { f.life = Math.max(0, Math.min(f.lifeMax, num(f, sc, 'value') | 0)); },
  targetlifeadd(f, sc) {
    const id = has(sc, 'id') ? num(f, sc, 'id') : -1;
    for (const t of f.targets) if (id < 0 || t.hitId === id) lifeAdd(t.f, num(f, sc, 'value') * (num(f, sc, 'absolute', 0) ? 1 : f.attackMul), num(f, sc, 'kill', 1) !== 0, num(f, sc, 'absolute', 0) !== 0, f);
  },
  targetpoweradd(f, sc) { for (const t of f.targets) t.f.addPower(num(f, sc, 'value')); },
  targetvelset(f, sc) { for (const t of f.targets) { if (has(sc, 'x')) t.f.vel.x = num(f, sc, 'x') * f.lscl; if (has(sc, 'y')) t.f.vel.y = num(f, sc, 'y') * f.lscl; } },
  targetveladd(f, sc) { for (const t of f.targets) { if (has(sc, 'x')) t.f.vel.x += num(f, sc, 'x') * f.lscl; if (has(sc, 'y')) t.f.vel.y += num(f, sc, 'y') * f.lscl; } },
  targetfacing(f, sc) { const v = num(f, sc, 'value', 1); for (const t of f.targets) t.f.facing = v > 0 ? f.facing : -f.facing; },
  targetbind(f, sc) {
    const id = has(sc, 'id') ? num(f, sc, 'id') : -1;
    for (const t of f.targets) if (id < 0 || t.hitId === id) {
      t.f.bind = { to: f, time: num(f, sc, 'time', 1) | 0, x: num(f, sc, 'pos', 0, 0) * f.lscl, y: num(f, sc, 'pos', 0, 1) * f.lscl, kind: 'target' };
      t.f.updateBind();
    }
  },
  bindtotarget(f, sc) {
    const t = f.findTarget(has(sc, 'id') ? num(f, sc, 'id') : -1);
    if (!t) return;
    f.bind = { to: t, time: num(f, sc, 'time', 1) | 0, x: num(f, sc, 'pos', 0, 0) * f.lscl * (t.facing === f.facing ? 1 : -1), y: num(f, sc, 'pos', 0, 1) * f.lscl, kind: 'target' };
    f.updateBind();
  },
  bindtoparent(f, sc) {
    const p = f.parent; if (!p) return;
    f.bind = { to: p, time: num(f, sc, 'time', 1) | 0, x: num(f, sc, 'pos', 0, 0) * f.lscl, y: num(f, sc, 'pos', 0, 1) * f.lscl, kind: 'parent' };
    f.updateBind();
  },
  bindtoroot(f, sc) {
    f.bind = { to: f.root, time: num(f, sc, 'time', 1) | 0, x: num(f, sc, 'pos', 0, 0) * f.lscl, y: num(f, sc, 'pos', 0, 1) * f.lscl, kind: 'root' };
    f.updateBind();
  },
  targetdrop(f, sc) {
    const keep = has(sc, 'excludeid') ? num(f, sc, 'excludeid') : -1;
    f.targets = f.targets.filter((t) => keep >= 0 && t.hitId === keep);
  },
  explod(f, sc) { f.match.spawnExplod(f, sc, num.bind(null, f, sc) as any, has.bind(null, sc) as any, raw.bind(null, sc) as any); },
  gamemakeanim(f, sc) {
    const pos: [number, number] = [num(f, sc, 'pos', 0, 0), num(f, sc, 'pos', 0, 1)];
    f.match.spawnFx(f, num(f, sc, 'value') | 0, f.pos.x + pos[0] * f.facing * f.lscl, f.pos.y + pos[1] * f.lscl, num(f, sc, 'under', 0) !== 0);
  },
  removeexplod(f, sc) { f.match.removeExplods(f, has(sc, 'id') ? num(f, sc, 'id') : -1); },
  explodbindtime(f, sc) { f.match.modifyExplods(f, has(sc, 'id') ? num(f, sc, 'id') : -1, (e) => { e.bindTime = num(f, sc, 'time', num(f, sc, 'value', 0)); }); },
  modifyexplod(f, sc) {
    f.match.modifyExplods(f, has(sc, 'id') ? num(f, sc, 'id') : -1, (e) => {
      if (has(sc, 'vel')) { e.vel.x = num(f, sc, 'vel', 0, 0) * f.lscl; e.vel.y = num(f, sc, 'vel', 0, 1) * f.lscl; }
      if (has(sc, 'accel')) { e.accel.x = num(f, sc, 'accel', 0, 0) * f.lscl; e.accel.y = num(f, sc, 'accel', 0, 1) * f.lscl; }
      if (has(sc, 'removetime')) e.removeTime = num(f, sc, 'removetime') | 0;
      if (has(sc, 'scale')) e.scale = [num(f, sc, 'scale', 1, 0), num(f, sc, 'scale', 1, 1)];
    });
  },
  helper(f, sc) { f.match.spawnHelper(f, sc, (k: string, d = 0, i = 0) => num(f, sc, k, d, i), (k: string) => has(sc, k), (k: string) => raw(sc, k)); },
  destroyself(f) { f.match.destroyHelper(f); },
  projectile(f, sc) { f.match.spawnProjectile(f, sc, (k: string, d = 0, i = 0) => num(f, sc, k, d, i), (k: string, i = 0) => has(sc, k, i), (k: string) => raw(sc, k)); },
  width(f, sc) {
    const k = has(sc, 'player') ? 'player' : has(sc, 'edge') ? 'edge' : 'value';
    f.widthOverride = { front: num(f, sc, k, 0, 0) * f.lscl, back: num(f, sc, k, 0, 1) * f.lscl };
  },
  sprpriority(f, sc) { f.sprPriority = num(f, sc, 'value') | 0; },
  assertspecial(f, sc) {
    for (const k of ['flag', 'flag2', 'flag3', 'flag4', 'flag5']) {
      const v = raw(sc, k);
      if (!v) continue;
      const fl = v.toLowerCase();
      if (['intro', 'roundnotover', 'noko', 'nobardisplay', 'nobg', 'nofg', 'timerfreeze', 'nomusic', 'globalnoshadow', 'nokosnd', 'nokoslow'].includes(fl)) f.match.asserts.add(fl);
      else f.asserts.add(fl);
    }
  },
  playerpush(f, sc) { f.playerPush = num(f, sc, 'value', 1) !== 0; },
  screenbound(f, sc) { f.screenBound = num(f, sc, 'value', 0) !== 0; },
  envshake(f, sc) { f.match.shake(num(f, sc, 'time', 1), num(f, sc, 'ampl', -4)); },
  fallenvshake() {},
  pause(f, sc) { f.match.pause(f, num(f, sc, 'time', 0) | 0, num(f, sc, 'movetime', 0) | 0, false); },
  superpause(f, sc) {
    f.match.pause(f, num(f, sc, 'time', 30) | 0, num(f, sc, 'movetime', 0) | 0, true, num(f, sc, 'darken', 1) !== 0);
    f.addPower(num(f, sc, 'poweradd', 0));
    const anim = raw(sc, 'anim');
    const animNo = anim ? parseInt(anim.replace(/^s/i, '')) : 30;
    if (animNo >= 0) f.match.spawnSuperFx(f, num(f, sc, 'pos', 0, 0) * f.lscl, num(f, sc, 'pos', 0, 1) * f.lscl, /^s/i.test(anim ?? '') ? animNo : -1);
    const snd = parseSoundRef(raw(sc, 'sound'));
    if (snd) f.match.playSound(f, snd[0], snd[1], /^s/i.test(raw(sc, 'sound') ?? ''), 1, -1);
    else f.match.playSound(f, -1, 0, false, 1, -1, 'super');
  },
  afterimage(f, sc) { f.afterImage = { time: num(f, sc, 'time', 1) | 0, length: Math.min(30, num(f, sc, 'length', 20) | 0), trail: [] }; },
  afterimagetime(f, sc) { if (f.afterImage) { f.afterImage.time = num(f, sc, 'time', num(f, sc, 'value', 0)) | 0; } },
  palfx(f, sc) {
    f.palfx = {
      time: num(f, sc, 'time', 0) | 0,
      add: [num(f, sc, 'add', 0, 0), num(f, sc, 'add', 0, 1), num(f, sc, 'add', 0, 2)],
      mul: [num(f, sc, 'mul', 256, 0), num(f, sc, 'mul', 256, 1), num(f, sc, 'mul', 256, 2)],
      color: num(f, sc, 'color', 256),
    };
  },
  allpalfx(f, sc) { f.match.flash(num(f, sc, 'time', 0) | 0, [num(f, sc, 'add', 0, 0), num(f, sc, 'add', 0, 1), num(f, sc, 'add', 0, 2)]); },
  bgpalfx(f, sc) { f.match.flash(num(f, sc, 'time', 0) | 0, [num(f, sc, 'add', 0, 0), num(f, sc, 'add', 0, 1), num(f, sc, 'add', 0, 2)]); },
  envcolor(f, sc) { f.match.flash(num(f, sc, 'time', 1) | 0, [num(f, sc, 'value', 255, 0), num(f, sc, 'value', 255, 1), num(f, sc, 'value', 255, 2)]); },
  trans(f, sc) {
    const t = (raw(sc, 'trans') ?? 'none').toLowerCase();
    f.trans = { mode: t, src: num(f, sc, 'alpha', 256, 0), dst: num(f, sc, 'alpha', 0, 1) };
  },
  angledraw(f, sc) { f.angleDraw = true; if (has(sc, 'value')) f.angle = num(f, sc, 'value'); if (has(sc, 'scale')) f.drawScale = [num(f, sc, 'scale', 1, 0), num(f, sc, 'scale', 1, 1)]; },
  angleset(f, sc) { f.angle = num(f, sc, 'value'); },
  angleadd(f, sc) { f.angle += num(f, sc, 'value'); },
  anglemul(f, sc) { f.angle *= num(f, sc, 'value', 1); },
  nothitby(f, sc) { f.notHitBy = [{ attr: (raw(sc, 'value') ?? raw(sc, 'value2') ?? '').toUpperCase(), time: num(f, sc, 'time', 1) | 0 }]; },
  hitby(f, sc) { f.hitBy = [{ attr: (raw(sc, 'value') ?? raw(sc, 'value2') ?? '').toUpperCase(), time: num(f, sc, 'time', 1) | 0 }]; },
  hitoverride() {},
  movehitreset(f) { f.moveContact = f.moveHit = f.moveGuarded = f.moveReversed = 0; },
  hitadd(f, sc) { f.hitCount += num(f, sc, 'value') | 0; },
  attackmulset(f, sc) { f.attackMul = num(f, sc, 'value', 1); },
  defencemulset(f, sc) { f.defenceMul = num(f, sc, 'value', 1); },
  attackdist(f, sc) { if (f.hitdef) f.hitdef.guardDist = num(f, sc, 'value', 160); },
  offset() {},
  victoryquote() {},
  displaytoclipboard() {}, appendtoclipboard() {}, clearclipboard() {},
  makedust(f, sc) { f.match.spawnDust(f.pos.x + num(f, sc, 'pos', 0, 0) * f.facing, f.pos.y + num(f, sc, 'pos', 0, 1)); },
  forcefeedback() {}, remappal() {}, lifebaraction() {}, zoom() {}, mapset() {}, mapadd() {}, savefile() {}, loadfile() {},
  movehitset() {}, screenshake() {},
};

export function runController(f: Fighter, sc: StateController) {
  const fn = C[sc.type];
  if (fn) fn(f, sc);
}

export const supportedControllers = Object.keys(C);
