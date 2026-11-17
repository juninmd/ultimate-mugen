use super::*;

pub const LARGE_PIXELS: u32 = 1024 * 1024;

thread_local! {
    static LARGE: RefCell<Vec<(Weak<TexInner>, u32)>> = const { RefCell::new(Vec::new()) };
}

/// `overflow` is set when the shared atlas has no room for the texture this frame, so any size qualifies.
pub fn try_draw(f: &mut Frame, t: &Texture2D, x: f32, y: f32, c: Color, p: &DrawTextureParams, overflow: bool) -> bool {
    let inner = &t.0;
    if (!overflow && inner.w * inner.h < LARGE_PIXELS) || p.rotation != 0.0 || p.pal_fx.is_some() { return false; }
    let Some(id) = slot_of(f, inner) else { return false };
    let (w, h) = (inner.w as f32, inner.h as f32);
    let src = p.source.unwrap_or(Rect::new(0.0, 0.0, w, h));
    let size = p.dest_size.unwrap_or(vec2(src.w, src.h));
    let (mut u0, mut v0) = (src.x / w, src.y / h);
    let (mut u1, mut v1) = ((src.x + src.w) / w, (src.y + src.h) / h);
    if p.flip_x { std::mem::swap(&mut u0, &mut u1); }
    if p.flip_y { std::mem::swap(&mut v0, &mut v1); }
    let (mul, add) = (c.to_array(), [0.0; 4]);
    let v = |pos: [f32; 2], uv: [f32; 2]| ume_render::Vertex::new(pos, uv, RGBA_SPRITE, mul, add, 0.0);
    let verts = [
        v([x, y], [u0, v0]),
        v([x + size.x, y], [u1, v0]),
        v([x + size.x, y + size.y], [u1, v1]),
        v([x, y + size.y], [u0, v1]),
    ];
    let blend = f.canvas.blend;
    f.canvas.batch.bind = id;
    f.canvas.batch.push_quad_verts(&verts, blend);
    f.canvas.batch.bind = 0;
    true
}

fn slot_of(f: &mut Frame, inner: &Rc<TexInner>) -> Option<u32> {
    // a freed texture's address can be reused, so only a live owner may claim the slot
    let known = LARGE.with(|l| l.borrow().iter().find(|(w, _)| w.upgrade().is_some_and(|rc| Rc::ptr_eq(&rc, inner))).map(|(_, id)| *id));
    if known.is_some() { return known; }
    let dead: Vec<u32> = LARGE.with(|l| {
        let mut l = l.borrow_mut();
        let dead = l.iter().filter(|(w, _)| w.strong_count() == 0).map(|(_, id)| *id).collect();
        l.retain(|(w, _)| w.strong_count() > 0);
        dead
    });
    for id in dead { f.gpu.renderer.large.free(id); }
    let id = f.gpu.renderer.large.alloc(&f.gpu.device, inner.w, inner.h);
    f.gpu.renderer.large.write(&f.gpu.queue, id, inner.w, inner.h, &inner.pixels);
    LARGE.with(|l| l.borrow_mut().push((Rc::downgrade(inner), id)));
    Some(id)
}
