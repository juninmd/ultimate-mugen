use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::fighter::Fighter;
use ume_engine::input::k;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};
use ume_render::{Blend, Canvas, Color, Quad, RGBA_SPRITE};
use ume_shell::sprites::SpriteCache;
use ume_shell::{run, Config, Frame, Game, Gpu, Key};

struct Fight { m: Match, cache: SpriteCache, acc: f32 }

fn keys(f: &Frame) -> u32 {
    let i = f.input;
    let mut b = 0;
    for (key, bit) in [(Key::ArrowUp, k::U), (Key::ArrowDown, k::D), (Key::ArrowLeft, k::L), (Key::ArrowRight, k::R), (Key::KeyA, k::A), (Key::KeyS, k::B), (Key::KeyD, k::C), (Key::KeyZ, k::X), (Key::KeyX, k::Y), (Key::KeyC, k::Z), (Key::Enter, k::S)] {
        if i.is_down(key) { b |= bit; }
    }
    b
}

fn draw_fighter(cache: &mut SpriteCache, gpu: &mut Gpu, canvas: &mut Canvas, m: &Match, fi: &Fighter, scale: f32, origin: [f32; 2]) {
    let Some(fr) = fi.frame() else { return };
    let def: &CharDef = &fi.anim_def;
    let Some(spr) = def.sprites.get(&(fr.group, fr.index)) else { return };
    let Some(img) = spr.image.as_ref() else { return };
    let Some(src) = cache.get(gpu, img.id, img.w, img.h, &img.rgba) else { return };
    let (xs, ys) = (def.c("size.xscale", 1.0) as f32, def.c("size.yscale", 1.0) as f32);
    let kk = spr.scale * fi.lscl;
    let (kx, ky) = (kk * xs * fi.draw_scale[0] * fr.xscale, kk * ys * fi.draw_scale[1] * fr.yscale);
    let flip = (fi.facing < 0.0) ^ fr.flip_h;
    let px = fi.pos.x - m.camera.x + m.view_w / 2.0 + fr.x * fi.lscl * xs * fi.facing;
    let py = fi.pos.y - m.camera.y + m.stage.zoffset + fr.y * fi.lscl * ys;
    let left = if flip { px - (spr.width - spr.axis_x) * kx } else { px - spr.axis_x * kx };
    let top = if fr.flip_v { py - (spr.height - spr.axis_y) * ky } else { py - spr.axis_y * ky };
    let mut q = Quad::new([origin[0] + left * scale, origin[1] + top * scale, spr.width * kx * scale, spr.height * ky * scale], src, [0, 0], RGBA_SPRITE);
    q.flip_h = flip;
    q.flip_v = fr.flip_v;
    q.blend = match fr.blend { ume_engine::chardef::Blend::Add | ume_engine::chardef::Blend::Add1 => Blend::Add, ume_engine::chardef::Blend::Sub => Blend::Sub, _ => Blend::Alpha };
    canvas.sprite(q);
}

impl Game for Fight {
    fn frame(&mut self, f: &mut Frame) {
        if f.input.is_pressed(Key::Escape) { f.quit = true; }
        let input = [keys(f), 0];
        self.acc += f.dt.min(0.25);
        while self.acc >= 1.0 / 60.0 { self.m.tick(&input); self.acc -= 1.0 / 60.0; }
        let m = &self.m;
        let scale = (f.size[0] / m.view_w).min(f.size[1] / m.view_h);
        let origin = [(f.size[0] - m.view_w * scale) / 2.0, (f.size[1] - m.view_h * scale) / 2.0];
        f.clear = [0.05, 0.06, 0.1, 1.0];
        let ground = origin[1] + (m.stage.zoffset - m.camera.y) * scale;
        f.canvas.rect(origin[0], origin[1], m.view_w * scale, ground - origin[1], Color::rgba(28, 34, 60, 255));
        f.canvas.rect(origin[0], ground, m.view_w * scale, origin[1] + m.view_h * scale - ground, Color::rgba(60, 52, 44, 255));
        for (side, fi) in m.fighters.iter().filter(|x| !x.is_helper).enumerate().take(2) {
            let w = 140.0 * scale;
            let x = if side == 0 { origin[0] + 8.0 * scale } else { origin[0] + m.view_w * scale - 8.0 * scale - w };
            f.canvas.rect(x, origin[1] + 8.0 * scale, w, 8.0 * scale, Color::rgba(60, 10, 10, 255));
            f.canvas.rect(x, origin[1] + 8.0 * scale, w * (fi.life / fi.life_max).clamp(0.0, 1.0), 8.0 * scale, Color::rgba(240, 200, 40, 255));
        }
        let mut order: Vec<usize> = (0..m.fighters.len()).collect();
        order.sort_by_key(|&i| m.fighters[i].spr_priority);
        for i in order {
            draw_fighter(&mut self.cache, f.gpu, f.canvas, &self.m, &self.m.fighters[i], scale, origin);
        }
    }
}

fn main() {
    let mut v = Vfs::new();
    add_zip(&mut v, include_bytes!("../../../assets/mugen/elecbyte.zip"), "").unwrap();
    let def = find_char_defs(&v).into_iter().next().unwrap();
    let d = Arc::new(load_mugen_char(&v, &def, 1, true).unwrap());
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d, 1, 4, 2);
    m.start_round();
    let max_frames = std::env::var("UME_FRAMES").ok().and_then(|v| v.parse().ok());
    match run(Config { title: "Ultimate MUGEN (wgpu)".into(), max_frames, ..Default::default() }, Fight { m, cache: SpriteCache::default(), acc: 0.0 }) {
        Ok(n) => println!("frames: {n}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}
