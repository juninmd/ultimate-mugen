import { compileParam, type StateController } from './chardef';
import type { Fighter, HitDefData } from './fighter';
import { resolver } from './triggers';

const ANIMTYPE: Record<string, number> = { light: 0, medium: 1, med: 1, hard: 2, heavy: 2, back: 3, up: 4, diagup: 5 };
const GROUNDTYPE: Record<string, number> = { none: 0, high: 1, low: 2, trip: 3 };

export function parseSoundRef(raw: string | undefined, defOwn = false): [number, number, boolean] | null {
  if (!raw) return null;
  let s = raw.trim();
  let own = defOwn;
  if (/^s/i.test(s)) { own = true; s = s.slice(1); }
  else if (/^f/i.test(s)) { own = false; s = s.slice(1); }
  const p = s.split(',').map((x) => parseInt(x));
  if (!Number.isFinite(p[0])) return null;
  return [p[0], p[1] || 0, own];
}

export function buildHitDef(f: Fighter, sc: StateController): HitDefData {
  const num = (k: string, d: number, i = 0) => {
    const p = sc.params.get(k);
    if (!p) return d;
    const c = compileParam<Fighter>(p, resolver);
    return c[i] ? c[i].f(f) : d;
  };
  const has = (k: string, i = 0) => { const p = sc.params.get(k); return !!p && !!p.nodes && p.nodes.length > i; };
  const raw = (k: string) => sc.params.get(k)?.raw.trim();
  const attr = (raw('attr') ?? 'S,NA').toUpperCase().replace(/\s/g, '');
  const [attrState, ...kinds] = attr.split(',');
  const l = f.lscl;
  const damage = num('damage', 0, 0);
  const pauseP1 = num('pausetime', 0, 0), pauseP2 = num('pausetime', 0, 1);
  const animtype = ANIMTYPE[(raw('animtype') ?? 'light').toLowerCase()] ?? 0;
  const groundType = GROUNDTYPE[(raw('ground.type') ?? 'high').toLowerCase()] ?? 1;
  const sparkRaw = raw('sparkno');
  const gsparkRaw = raw('guard.sparkno');
  const parseSpark = (r: string | undefined, d: number): [number, boolean] => {
    if (!r) return [d, false];
    const own = /^s/i.test(r);
    const n = parseInt(r.replace(/^[sf]/i, ''));
    return [Number.isFinite(n) ? n : d, own];
  };
  const [sparkNo, sparkOwn] = parseSpark(sparkRaw, f.const('data.sparkno', 2));
  const [guardSparkNo, guardSparkOwn] = parseSpark(gsparkRaw, f.const('data.guard.sparkno', 40));
  const groundVelX = num('ground.velocity', 0, 0) * l, groundVelY = num('ground.velocity', 0, 1) * l;
  const fall = num('fall', 0) !== 0;
  const groundHitTime = num('ground.hittime', 0);
  const groundSlideTime = num('ground.slidetime', 0);
  const d: HitDefData = {
    attr, attrState, attrKind: kinds,
    hitflag: (raw('hitflag') ?? 'MAF').toUpperCase(),
    guardflag: (raw('guardflag') ?? '').toUpperCase(),
    animtype,
    airAnimtype: has('air.animtype') ? ANIMTYPE[(raw('air.animtype') ?? '').toLowerCase()] ?? animtype : animtype,
    fallAnimtype: has('fall.animtype') ? ANIMTYPE[(raw('fall.animtype') ?? '').toLowerCase()] ?? 4 : (animtype >= 3 ? animtype : 4),
    priority: num('priority', 4),
    damage, guardDamage: num('damage', 0, 1),
    pauseP1, pauseP2,
    guardPauseP1: has('guard.pausetime') ? num('guard.pausetime', 0, 0) : pauseP1,
    guardPauseP2: has('guard.pausetime') ? num('guard.pausetime', 0, 1) : pauseP2,
    sparkNo, sparkOwn, guardSparkNo, guardSparkOwn,
    sparkX: num('sparkxy', 0, 0) * l, sparkY: num('sparkxy', 0, 1) * l,
    hitSound: parseSoundRef(raw('hitsound')) ?? [5, 0, false],
    guardSound: parseSoundRef(raw('guardsound')) ?? [6, 0, false],
    groundType, airType: has('air.type') ? GROUNDTYPE[(raw('air.type') ?? '').toLowerCase()] ?? groundType : groundType,
    groundSlideTime, guardSlideTime: has('guard.slidetime') ? num('guard.slidetime', 0) : groundSlideTime,
    groundHitTime, guardHitTime: has('guard.hittime') ? num('guard.hittime', 0) : groundHitTime,
    airHitTime: num('air.hittime', 20),
    guardCtrlTime: has('guard.ctrltime') ? num('guard.ctrltime', 0) : (has('guard.slidetime') ? num('guard.slidetime', 0) : groundSlideTime),
    guardDist: num('guard.dist', f.const('size.attack.dist', 160)),
    yaccel: num('yaccel', f.const('movement.yaccel', 0.44)) * l,
    groundVel: [groundVelX, groundVelY],
    guardVel: has('guard.velocity') ? num('guard.velocity', 0) * l : groundVelX,
    airVel: has('air.velocity') ? [num('air.velocity', 0, 0) * l, num('air.velocity', 0, 1) * l] : [groundVelX, groundVelY],
    airGuardVel: has('airguard.velocity') ? [num('airguard.velocity', 0, 0) * l, num('airguard.velocity', 0, 1) * l] : [groundVelX * 1.5, groundVelY / 2],
    downVel: has('down.velocity') ? [num('down.velocity', 0, 0) * l, num('down.velocity', 0, 1) * l] : [groundVelX, groundVelY],
    groundCornerPush: num('ground.cornerpush.veloff', 0) * l,
    airCornerPush: num('air.cornerpush.veloff', 0) * l,
    airJuggle: num('air.juggle', 0),
    p1sprpriority: num('p1sprpriority', 1), p2sprpriority: num('p2sprpriority', 0),
    p1facing: num('p1facing', 0), p2facing: num('p2facing', 0),
    p1stateno: has('p1stateno') ? num('p1stateno', -1) : -1,
    p2stateno: has('p2stateno') ? num('p2stateno', -1) : -1,
    p2getp1state: num('p2getp1state', 1) !== 0,
    forcestand: num('forcestand', groundVelY !== 0 ? 1 : 0) !== 0,
    forcenofall: num('forcenofall', 0) !== 0,
    fall, airFall: has('air.fall') ? num('air.fall', 0) !== 0 : fall,
    fallXvel: has('fall.xvelocity') ? num('fall.xvelocity', 0) * l : null,
    fallYvel: num('fall.yvelocity', -4.5) * l,
    fallRecover: num('fall.recover', 1) !== 0,
    fallRecoverTime: num('fall.recovertime', 4),
    fallDamage: num('fall.damage', 0),
    fallKill: num('fall.kill', 1) !== 0,
    kill: num('kill', 1) !== 0,
    guardKill: num('guard.kill', 1) !== 0,
    getPower: has('getpower') ? num('getpower', 0, 0) : damage,
    givePower: has('givepower') ? num('givepower', 0, 0) : damage,
    guardGetPower: has('getpower', 1) ? num('getpower', 0, 1) : damage / 2,
    guardGivePower: has('givepower', 1) ? num('givepower', 0, 1) : damage / 2,
    id: num('id', 0), chainId: num('chainid', -1), nochainId: [num('nochainid', -1, 0), num('nochainid', -1, 1)],
    envShakeTime: num('envshake.time', 0),
    hitOnce: num('hitonce', attrState.includes('T') ? 1 : 0) !== 0 || kinds.some((k) => k[1] === 'T'),
    numHits: num('numhits', 1),
    palfxTime: num('palfx.time', 0),
  };
  if (!d.guardflag) d.guardflag = '';
  return d;
}
