use super::bindings;
use crate::gfx::*;
use crate::render::Gfx;
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;
use ume_model3d::Facing;

const MAX_TRIANGLES_PER_MESH: usize = 21_000;

pub(super) fn draw(g: &mut Gfx, m: &Match, f: &Fighter, shx: f32, shy: f32) -> bool {
    if crate::lite::scale() < 1.0 {
        return false;
    }
    let id = f.def.id.trim_start_matches("umf:");
    let origin = [
        f.pos.x + f.draw_offset.x * f.facing - m.camera.x + m.view_w / 2.0 + shx,
        f.pos.y + f.lane + f.draw_offset.y - m.camera.y + m.stage.zoffset + shy,
    ];
    let facing = if f.facing >= 0.0 {
        Facing::Right
    } else {
        Facing::Left
    };
    let (ox, oy, s) = (g.ox, g.oy, g.view_scale);
    bindings::with(id, |b| {
        b.model
            .pose_for_state(&b.states, f.state_no, f.state_time, &mut b.posed);
        let mut camera = b.camera;
        camera.pixels_per_unit *= f.lscl;
        b.model
            .project(&b.posed, &camera, facing, origin, &mut b.triangles);
        let mut start = 0;
        while start < b.triangles.len() {
            let texture = b.triangles[start].texture;
            let mut end = start;
            while end < b.triangles.len()
                && end - start < MAX_TRIANGLES_PER_MESH
                && b.triangles[end].texture == texture
            {
                end += 1;
            }
            let mut vertices = Vec::with_capacity((end - start) * 3);
            for t in &b.triangles[start..end] {
                let [r, gr, bl, a] = t.color;
                let color = Color::new(
                    r as f32 / 255.0,
                    gr as f32 / 255.0,
                    bl as f32 / 255.0,
                    a as f32 / 255.0,
                );
                for k in 0..3 {
                    vertices.push(Vertex::new(
                        ox + t.points[k][0] * s,
                        oy + t.points[k][1] * s,
                        0.0,
                        t.uv[k][0],
                        t.uv[k][1],
                        color,
                    ));
                }
            }
            let indices = (0..vertices.len() as u16).collect();
            let texture = texture.and_then(|i| b.textures.get(i).cloned().flatten());
            draw_mesh(&Mesh {
                vertices,
                indices,
                texture,
            });
            start = end;
        }
    })
    .is_some()
}
