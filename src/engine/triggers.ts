import type { Compiled, Fn, Node, TriggerResolver } from './expr';
import type { Fighter } from './fighter';

type C = Compiled<Fighter>;
const I = (f: Fn<Fighter>): C => ({ f, float: false });
const F = (f: Fn<Fighter>): C => ({ f, float: true });
const b = (x: boolean) => (x ? 1 : 0);

const warned = new Set<string>();
function warnOnce(s: string) { if (!warned.has(s)) { warned.add(s); if (typeof console !== 'undefined') console.debug('[expr] unsupported:', s); } }

function ghvField(f: Fighter, k: string): number {
  const g = f.ghv;
  switch (k) {
    case 'xveladd': case 'yveladd': return 0;
    case 'type': return g.type;
    case 'animtype': return g.animtype;
    case 'airtype': return g.airtype;
    case 'groundtype': return g.groundtype;
    case 'damage': return g.damage;
    case 'hitcount': return g.hitcount;
    case 'fallcount': return g.fallcount;
    case 'hitshaketime': return f.hitShake;
    case 'hittime': return g.hittime;
    case 'slidetime': return g.slidetime;
    case 'ctrltime': return g.ctrltime;
    case 'recovertime': return g.recovertime;
    case 'xoff': case 'yoff': return 0;
    case 'xvel': return g.xvel / f.lscl;
    case 'yvel': return g.yvel / f.lscl;
    case 'yaccel': return g.yaccel / f.lscl;
    case 'chainid': return g.chainid;
    case 'guarded': return b(g.guarded);
    case 'isbound': return 0;
    case 'fall': return b(g.fall);
    case 'fall.damage': return g.fallDamage;
    case 'fall.xvel': return g.fallXvel / f.lscl;
    case 'fall.yvel': return g.fallYvel / f.lscl;
    case 'fall.recover': return b(g.fallRecover);
    case 'fall.time': return f.fallTime;
    case 'fall.recovertime': return g.fallRecoverTime;
    case 'fall.kill': return b(g.fallKill);
    case 'fall.envshake.time': case 'fall.envshake.freq': case 'fall.envshake.ampl': case 'fall.envshake.phase': return 0;
    case 'id': return g.attackerId;
    case 'playerno': return g.attackerNo;
    case 'kill': return b(g.kill);
    case 'down.recover': return 1;
    case 'down.recovertime': return 0;
    case 'guardcount': return g.guardcount;
    case 'score': return 0;
    case 'frame': return 0;
  }
  warnOnce('gethitvar(' + k + ')');
  return 0;
}

function constVal(f: Fighter, k: string): number {
  const c = f.def.constants;
  const v = c.get(k);
  if (v !== undefined) return v;
  // aliases
  if (k.startsWith('velocity.') && !/\.[xyz]$/.test(k)) return c.get(k + '.x') ?? 0;
  if (k === 'data.defence') return c.get('data.defense') ?? 100;
  if (k === 'data.defense') return c.get('data.defence') ?? 100;
  return 0;
}

const SPECIAL_STATE_TYPES: Record<string, string> = { s: 'S', c: 'C', a: 'A', l: 'L', u: 'U' };

export const resolver: TriggerResolver<Fighter> = {
  redirect(kind, arg) {
    switch (kind) {
      case 'parent': return (h) => h.parent ?? undefined;
      case 'root': return (h) => h.root;
      case 'helper': return (h) => h.findHelper(arg ? arg(h) : -1);
      case 'target': return (h) => h.findTarget(arg ? arg(h) : -1);
      case 'partner': return () => undefined;
      case 'enemy': case 'enemynear': case 'p2': return (h) => h.enemy(arg ? arg(h) : 0);
      case 'playerid': return (h) => h.match.byId(arg ? arg(h) : -1);
      case 'stateowner': return (h) => h;
    }
    return () => undefined;
  },
  cmpstr(n) {
    const vals = n.values.map((v) => v.toLowerCase());
    const neg = n.neg;
    const r = (x: boolean) => (x !== neg ? 1 : 0);
    switch (n.name) {
      case 'command': { const name = n.values[0]; return (h) => r(h.hasCommand(name)); }
      case 'statetype': { const v = SPECIAL_STATE_TYPES[vals[0]] ?? vals[0].toUpperCase(); return (h) => r(h.stateType === v); }
      case 'movetype': { const v = vals[0].toUpperCase(); return (h) => r(h.moveType === v); }
      case 'physics': { const v = vals[0].toUpperCase(); return (h) => r(h.physics === v); }
      case 'p2statetype': { const v = vals[0].toUpperCase(); return (h) => { const e = h.enemy(0); return r(!!e && e.stateType === v); }; }
      case 'p2movetype': { const v = vals[0].toUpperCase(); return (h) => { const e = h.enemy(0); return r(!!e && e.moveType === v); }; }
      case 'name': case 'p1name': return (h) => r(h.def.name.toLowerCase() === vals[0]);
      case 'p2name': return (h) => { const e = h.enemy(0); return r(!!e && e.def.name.toLowerCase() === vals[0]); };
      case 'p3name': case 'p4name': return () => r(false);
      case 'authorname': return (h) => r(h.def.author.toLowerCase() === vals[0]);
      case 'hitdefattr': return (h) => r(h.hitdefAttrMatches(vals));
    }
    return () => 0;
  },
  special(n, c) {
    const a = c(n.a).f;
    const bb = n.b ? c(n.b).f : undefined;
    const cmp = (x: number, op: string, y: number) => {
      switch (op) {
        case '=': return x === y; case '!=': return x !== y; case '<': return x < y;
        case '<=': return x <= y; case '>': return x > y; case '>=': return x >= y;
      }
      return false;
    };
    const neg = n.op === '!=';
    if (n.name === 'animelem') {
      if (!bb) return (h) => b(h.animElemStarted(a(h)) !== neg);
      const op2 = n.op2 ?? '=';
      return (h) => {
        const dt = h.animElemTime(a(h));
        if (dt === null) return 0;
        // MUGEN: if "dt" before elem start it is negative
        return b(cmp(dt, op2, bb(h)));
      };
    }
    if (n.name === 'timemod') {
      return (h) => { const m = a(h); return b(m !== 0 && cmp(h.stateTime % m, '=', bb ? bb(h) : 0) !== neg); };
    }
    // projhit / projcontact / projguarded [id]
    const mm = /^proj(hit|contact|guarded)(\d*)$/.exec(n.name);
    if (mm) {
      const kind = mm[1] as 'hit' | 'contact' | 'guarded';
      const id = mm[2] ? parseInt(mm[2]) : 0;
      return (h) => {
        const t = h.projContactTime(kind, id);
        const happened = t >= 0;
        const want = a(h) !== 0;
        if (!bb) return b((happened === want) !== neg);
        return b(happened === want && cmp(t, n.op2 ?? '=', bb(h)));
      };
    }
    return () => 0;
  },
  assign(name, idx, v) {
    return (h) => {
      const i = idx(h) | 0, val = v.f(h);
      if (name === 'var') return (h.vars[i] = val | 0);
      if (name === 'fvar') return (h.fvars[i] = val);
      if (name === 'sysvar') return (h.sysvars[i] = val | 0);
      return (h.sysfvars[i] = val);
    };
  },
  trigger(n, c): C {
    const args = n.args.map((x) => c(x));
    const a0 = args[0]?.f ?? (() => 0);
    const comp = n.comp;
    switch (n.name) {
      case '__str': return I(() => 0);
      case 'time': case 'statetime': return I((h) => h.stateTime);
      case 'animtime': return I((h) => h.animTimeTrig());
      case 'anim': return I((h) => h.animNo);
      case 'animelemno': return I((h) => h.animElemNoAt(a0(h)));
      case 'animelemtime': return I((h) => h.animElemTime(a0(h)) ?? -1);
      case 'animexist': case 'selfanimexist': return I((h) => b(h.def.anims.has(a0(h) | 0)));
      case 'stateno': return I((h) => h.stateNo);
      case 'prevstateno': return I((h) => h.prevStateNo);
      case 'ctrl': return I((h) => b(h.ctrl));
      case 'alive': return I((h) => b(h.alive));
      case 'life': return I((h) => h.life);
      case 'lifemax': return I((h) => h.lifeMax);
      case 'power': return I((h) => h.power);
      case 'powermax': return I((h) => h.powerMax);
      case 'facing': return I((h) => h.facing);
      case 'var': return I((h) => h.vars[a0(h) | 0] ?? 0);
      case 'fvar': return F((h) => h.fvars[a0(h) | 0] ?? 0);
      case 'sysvar': return I((h) => h.sysvars[a0(h) | 0] ?? 0);
      case 'sysfvar': return F((h) => h.sysfvars[a0(h) | 0] ?? 0);
      case 'vel': return F((h) => (comp === 'y' ? h.vel.y : comp === 'z' ? 0 : h.vel.x) / h.lscl);
      case 'pos': return F((h) => (comp === 'y' ? h.pos.y : comp === 'z' ? 0 : h.pos.x - h.match.camera.x) / h.lscl);
      case 'screenpos': return F((h) => (comp === 'y' ? h.pos.y - h.match.camera.y + h.match.groundScreenY() : h.pos.x - h.match.camera.x + h.match.viewW / 2));
      case 'camerapos': return F((h) => (comp === 'y' ? h.match.camera.y : h.match.camera.x));
      case 'camerazoom': return F(() => 1);
      case 'p2dist': case 'p1dist': return F((h) => { const e = h.enemy(0); if (!e) return 0; return (comp === 'y' ? e.pos.y - h.pos.y : (e.pos.x - h.pos.x) * h.facing) / h.lscl; });
      case 'p2bodydist': return F((h) => {
        const e = h.enemy(0); if (!e) return 0;
        if (comp === 'y') return (e.pos.y - h.pos.y) / h.lscl;
        const dx = (e.pos.x - h.pos.x) * h.facing;
        const myFront = h.frontWidth();
        const theirFront = (e.pos.x - h.pos.x) * e.facing < 0 ? e.frontWidth() : e.backWidth();
        return (dx - myFront - theirFront) / h.lscl;
      });
      case 'rootdist': return F((h) => (comp === 'y' ? h.root.pos.y - h.pos.y : (h.root.pos.x - h.pos.x) * h.facing) / h.lscl);
      case 'parentdist': return F((h) => { const p = h.parent ?? h; return (comp === 'y' ? p.pos.y - h.pos.y : (p.pos.x - h.pos.x) * h.facing) / h.lscl; });
      case 'frontedgedist': return F((h) => h.edgeDist(true) / h.lscl);
      case 'backedgedist': return F((h) => h.edgeDist(false) / h.lscl);
      case 'frontedgebodydist': return F((h) => (h.edgeDist(true) - h.frontWidth()) / h.lscl);
      case 'backedgebodydist': return F((h) => (h.edgeDist(false) - h.backWidth()) / h.lscl);
      case 'p2life': return I((h) => h.enemy(0)?.life ?? 0);
      case 'p2stateno': return I((h) => h.enemy(0)?.stateNo ?? 0);
      case 'p2ctrl': return I((h) => b(!!h.enemy(0)?.ctrl));
      case 'p2power': return I((h) => h.enemy(0)?.power ?? 0);
      case 'hitshakeover': return I((h) => b(h.hitShake <= 0));
      case 'hitover': return I((h) => b(h.hitOverTime <= 0));
      case 'hitfall': return I((h) => b(h.ghv.fall));
      case 'canrecover': return I((h) => b(h.ghv.fallRecover && h.fallTime >= h.ghv.fallRecoverTime));
      case 'movecontact': return I((h) => h.moveContact);
      case 'movehit': return I((h) => h.moveHit);
      case 'moveguarded': return I((h) => h.moveGuarded);
      case 'movereversed': return I((h) => h.moveReversed);
      case 'hitcount': return I((h) => h.hitCount);
      case 'uniqhitcount': return I((h) => h.hitCount);
      case 'hitpausetime': return I((h) => h.hitPause);
      case 'inguarddist': return I((h) => b(h.inGuardDist()));
      case 'gethitvar': return F((h) => ghvField(h, n.raw ?? ''));
      case 'const': return F((h) => constVal(h, n.raw ?? ''));
      case 'const240p': return F((h) => a0(h) * h.def.localW / 320);
      case 'const480p': return F((h) => a0(h) * h.def.localW / 640);
      case 'const720p': return F((h) => a0(h) * h.def.localW / 1280);
      case 'random': return I((h) => Math.floor(h.match.random() * 1000));
      case 'randomrange': return I((h) => { const lo = a0(h), hi = args[1]?.f(h) ?? lo; return lo + Math.floor(h.match.random() * (hi - lo + 1)); });
      case 'roundstate': return I((h) => h.match.roundState);
      case 'roundno': return I((h) => h.match.roundNo);
      case 'roundsexisted': return I((h) => h.match.roundNo - 1);
      case 'matchover': return I((h) => b(h.match.matchOver));
      case 'matchno': return I(() => 1);
      case 'gametime': return I((h) => h.match.gameTime);
      case 'tickspersecond': return I(() => 60);
      case 'ailevel': return I((h) => h.root.aiLevel);
      case 'ishelper': return I((h) => b(h.isHelper && (!args.length || h.helperId === a0(h))));
      case 'ishometeam': return I((h) => b(h.side === 0));
      case 'teamside': return I((h) => h.side + 1);
      case 'teammode': return I(() => 0);
      case 'id': return I((h) => h.id);
      case 'playeridexist': return I((h) => b(!!h.match.byId(a0(h))));
      case 'numhelper': return I((h) => h.match.countHelpers(h.root, args.length ? a0(h) : -1));
      case 'numexplod': return I((h) => h.match.countExplods(h, args.length ? a0(h) : -1));
      case 'numproj': return I((h) => h.match.countProjs(h, -1));
      case 'numprojid': return I((h) => h.match.countProjs(h, a0(h)));
      case 'numtarget': return I((h) => h.countTargets(args.length ? a0(h) : -1));
      case 'numenemy': return I((h) => h.match.roots.filter((r) => r.side !== h.side).length);
      case 'numpartner': return I(() => 0);
      case 'win': return I((h) => b(h.match.winnerSide === h.side));
      case 'lose': return I((h) => b(h.match.winnerSide >= 0 && h.match.winnerSide !== h.side));
      case 'drawgame': return I((h) => b(h.match.winnerSide === -2));
      case 'palno': return I((h) => h.palNo);
      case 'majorversion': return I(() => 1);
      case 'stagevar': return I(() => 0);
      case 'projcontact': case 'projhit': case 'projguarded':
        return I((h) => b(h.projContactTime(n.name.slice(4) as any, 0) >= 0));
      case 'projcontacttime': return I((h) => h.projContactTime('contact', a0(h)));
      case 'projhittime': return I((h) => h.projContactTime('hit', a0(h)));
      case 'projguardedtime': return I((h) => h.projContactTime('guarded', a0(h)));
      case 'backedge': return F((h) => h.match.camera.x - h.match.viewW / 2);
      case 'frontedge': return F((h) => h.match.camera.x + h.match.viewW / 2);
      case 'leftedge': return F((h) => h.match.camera.x - h.match.viewW / 2);
      case 'rightedge': return F((h) => h.match.camera.x + h.match.viewW / 2);
      case 'topedge': return F((h) => h.match.camera.y - h.match.groundScreenY());
      case 'bottomedge': return F((h) => h.match.camera.y + (h.match.viewH - h.match.groundScreenY()));
      case 'gameheight': return F((h) => h.match.viewH);
      case 'gamewidth': return F((h) => h.match.viewW);
      case 'p2': return I(() => 0);
      // math
      case 'abs': return { f: (h) => Math.abs(a0(h)), float: args[0]?.float ?? false };
      case 'floor': return I((h) => Math.floor(a0(h)));
      case 'ceil': return I((h) => Math.ceil(a0(h)));
      case 'sin': return F((h) => Math.sin(a0(h)));
      case 'cos': return F((h) => Math.cos(a0(h)));
      case 'tan': return F((h) => Math.tan(a0(h)));
      case 'asin': return F((h) => Math.asin(a0(h)));
      case 'acos': return F((h) => Math.acos(a0(h)));
      case 'atan': return F((h) => Math.atan(a0(h)));
      case 'exp': return F((h) => Math.exp(a0(h)));
      case 'ln': return F((h) => Math.log(a0(h)));
      case 'log': return F((h) => Math.log(args[1]?.f(h) ?? 0) / Math.log(a0(h)));
      case 'sqrt': return F((h) => Math.sqrt(a0(h)));
      case 'min': return { f: (h) => Math.min(a0(h), args[1]?.f(h) ?? 0), float: args.some((x) => x.float) };
      case 'max': return { f: (h) => Math.max(a0(h), args[1]?.f(h) ?? 0), float: args.some((x) => x.float) };
      case 'clamp': return { f: (h) => Math.min(Math.max(a0(h), args[1]?.f(h) ?? 0), args[2]?.f(h) ?? 0), float: args.some((x) => x.float) };
      case 'sign': return I((h) => Math.sign(a0(h)));
      case 'e': return F(() => Math.E);
      case 'pi': return F(() => Math.PI);
      case 'ifelse': {
        const [x, y, z] = args;
        return { f: (h) => (x.f(h) ? y.f(h) : z?.f(h) ?? 0), float: !!(y?.float || z?.float) };
      }
      case 'cond': {
        const [x, y, z] = args;
        return { f: (h) => (x.f(h) ? y.f(h) : z?.f(h) ?? 0), float: !!(y?.float || z?.float) };
      }
    }
    // tokens used as bare words in params, e.g. "S", "NA", "Heavy": evaluate to 0
    if (!n.args.length) {
      if (/^[a-z]{1,3}$/.test(n.name)) return I(() => 0);
    }
    warnOnce(n.name);
    return I(() => 0);
  },
};
