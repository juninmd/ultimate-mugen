import { K, VirtualSource } from '../engine/input';

/** On-screen controls for phones/tablets: analog-style d-pad + 4 attack buttons + simple-mode special buttons. */
export class TouchControls {
  el: HTMLElement;
  src = new VirtualSource();
  private dirBits = 0;
  private btnBits = 0;
  getFacing: () => number = () => 1;
  isMugen: () => boolean = () => false;

  constructor(parent: HTMLElement) {
    this.el = document.createElement('div');
    this.el.className = 'touch';
    this.el.innerHTML = `
      <div class="stick"><div class="stick-base"><div class="stick-knob"></div></div></div>
      <div class="pad">
        <button data-b="x" class="tb tb-x">LP</button>
        <button data-b="y" class="tb tb-y">HP</button>
        <button data-sp="sp1" class="tb tb-sp">SP1</button>
        <button data-b="a" class="tb tb-a">LK</button>
        <button data-b="b" class="tb tb-b">HK</button>
        <button data-sp="sp2" class="tb tb-sp">SP2</button>
        <button data-sp="sp3" class="tb tb-sp">SP3</button>
        <button data-sp="super" class="tb tb-super">SUPER</button>
      </div>`;
    parent.appendChild(this.el);
    this.bindStick(this.el.querySelector('.stick') as HTMLElement);
    for (const b of Array.from(this.el.querySelectorAll<HTMLButtonElement>('.tb'))) this.bindButton(b);
  }

  private update() { this.src.bits = this.dirBits | this.btnBits; }

  private bindStick(area: HTMLElement) {
    const knob = area.querySelector('.stick-knob') as HTMLElement;
    const base = area.querySelector('.stick-base') as HTMLElement;
    let id = -1, cx = 0, cy = 0;
    const set = (dx: number, dy: number) => {
      const len = Math.hypot(dx, dy), max = base.clientWidth * 0.38;
      const k = len > max ? max / len : 1;
      knob.style.transform = `translate(${dx * k}px, ${dy * k}px)`;
      let b = 0;
      const dead = max * 0.3;
      if (len > dead) {
        const a = Math.atan2(dy, dx);
        const oct = Math.round(a / (Math.PI / 4));
        const map: Record<number, number> = { 0: K.R, 1: K.R | K.D, 2: K.D, 3: K.D | K.L, 4: K.L, [-4]: K.L, [-3]: K.U | K.L, [-2]: K.U, [-1]: K.U | K.R };
        b = map[oct] ?? 0;
      }
      if (b !== this.dirBits && navigator.vibrate && b) navigator.vibrate(5);
      this.dirBits = b;
      this.update();
    };
    area.addEventListener('pointerdown', (e) => {
      id = e.pointerId; area.setPointerCapture(id);
      const r = base.getBoundingClientRect(); cx = r.left + r.width / 2; cy = r.top + r.height / 2;
      set(e.clientX - cx, e.clientY - cy); e.preventDefault();
    });
    area.addEventListener('pointermove', (e) => { if (e.pointerId === id) set(e.clientX - cx, e.clientY - cy); });
    const up = (e: PointerEvent) => { if (e.pointerId !== id) return; id = -1; knob.style.transform = ''; this.dirBits = 0; this.update(); };
    area.addEventListener('pointerup', up); area.addEventListener('pointercancel', up);
  }

  private bindButton(b: HTMLButtonElement) {
    const bit = b.dataset.b ? (K as any)[b.dataset.b] as number : 0;
    const sp = b.dataset.sp;
    const down = (e: PointerEvent) => {
      e.preventDefault(); b.classList.add('on');
      if (navigator.vibrate) navigator.vibrate(8);
      if (bit) { this.btnBits |= bit; this.update(); }
      if (sp) this.special(sp);
    };
    const up = () => { b.classList.remove('on'); if (bit) { this.btnBits &= ~bit; this.update(); } };
    b.addEventListener('pointerdown', down);
    b.addEventListener('pointerup', up); b.addEventListener('pointercancel', up); b.addEventListener('pointerleave', up);
  }

  /** Simple mode: native chars get dedicated bits; MUGEN chars get a motion macro. */
  special(sp: string) {
    if (!this.isMugen()) {
      const bits = sp === 'sp1' ? K.d : sp === 'sp2' ? K.w : sp === 'sp3' ? K.e : K.d | K.w;
      this.src.macro = [bits, bits, 0];
      return;
    }
    const f = this.getFacing() >= 0;
    const F = f ? K.R : K.L, B = f ? K.L : K.R, D = K.D;
    const qcf = [D, D, D | F, D | F, F, F];
    const seq = sp === 'sp1' ? [...qcf, F | K.x, 0] : sp === 'sp2' ? [F, F, D, D, D | F, D | F | K.x, 0] : sp === 'sp3' ? [D, D, D | B, D | B, B, B | K.a, 0] : [...qcf, 0, ...qcf, F | K.x, 0];
    this.src.macro = seq;
  }

  show(v: boolean) { this.el.style.display = v ? '' : 'none'; }
}
