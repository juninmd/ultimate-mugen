use super::*;
use crate::stages::story_loop::{
    backdrop, grade_at, pieces, seam_mix, Backdrop, SEAM_BAND, SEGMENT, SEGMENTS, STRIP,
};

const GRADE_STEP: f32 = 44.0;
const SEAM_STEP: f32 = 5.0;

struct Target {
    x: f32,
    y: f32,
    scale: f32,
    h: f32,
    texel: f32,
    source_h: f32,
}

impl Gfx {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_backdrop(
        &mut self,
        image: &Arc<ImageData>,
        frames: &[Arc<ImageData>],
        span: f32,
        time: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let (idx_a, idx_b, mix) = crate::stages::looping::loop_sample(time, frames.len());
        let frame_a = frames.get(idx_a);
        let frame_b = frames.get(idx_b);
        if let Some(upcoming) = frames.get((idx_b + 1) % frames.len().max(1)) {
            let _ = self.texture(upcoming, true);
        }
        if span <= 0.0 {
            self.draw_wide_image(frame_a.unwrap_or(image), x, y, w, h);
            if mix > 0.01 && idx_a != idx_b {
                if let Some(next) = frame_b {
                    self.draw_wide_image_alpha(next, x, y, w, h, mix);
                }
            }
            return;
        }
        let (Some(frame), Some(layout)) = (frame_a, backdrop(image)) else {
            return self.draw_wide_image(image, x, y, w, h);
        };
        let texture = self.texture(frame, true);
        let target = Target {
            x,
            y,
            scale: w / STRIP,
            h,
            texel: frame.w as f32 / SEGMENT,
            source_h: frame.h as f32,
        };
        draw_story_pass(&texture, &layout, &target, 1.0);
        if mix > 0.01 && idx_a != idx_b {
            if let Some(next) = frame_b {
                let next_tex = self.texture(next, true);
                draw_story_pass(&next_tex, &layout, &target, mix);
            }
        }
    }
}

fn draw_story_pass(texture: &Texture2D, layout: &Backdrop, target: &Target, alpha: f32) {
    let mut at = 0.0;
    while at < STRIP {
        let end = (at + GRADE_STEP).min(STRIP);
        stamp(texture, layout, target, (at, end), at, false, alpha);
        at = end;
    }
    for seam in 1..SEGMENTS {
        let seam = seam as f32 * SEGMENT;
        let mut offset = 0.0;
        while offset < SEAM_BAND {
            let step = SEAM_STEP.min(SEAM_BAND - offset);
            let (near, far) = (offset, offset + step);
            let mid = offset + step * 0.5;
            stamp(
                texture,
                layout,
                target,
                (seam + near, seam + far),
                seam - far,
                true,
                seam_mix(-mid) * alpha,
            );
            stamp(
                texture,
                layout,
                target,
                (seam - far, seam - near),
                seam + near,
                true,
                (1.0 - seam_mix(mid)) * alpha,
            );
            offset += step;
        }
    }
}

fn stamp(
    texture: &Texture2D,
    layout: &Backdrop,
    target: &Target,
    content: (f32, f32),
    dest: f32,
    mirrored: bool,
    alpha: f32,
) {
    let left = target.x + dest * target.scale;
    let right = target.x + (dest + content.1 - content.0) * target.scale;
    if right < 0.0 || left > screen_width() {
        return;
    }
    let [r, g, b] = grade_at(layout.grade, dest + (content.1 - content.0) * 0.5);
    for piece in pieces(&layout.runs, content.0, content.1, mirrored) {
        let x0 = (target.x + (dest + piece.offset) * target.scale).floor();
        let x1 = (target.x + (dest + piece.offset + piece.width) * target.scale).floor();
        draw_texture_ex(
            texture,
            x0,
            target.y,
            Color::new(r, g, b, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(x1 - x0, target.h)),
                source: Some(Rect::new(
                    piece.source * target.texel,
                    0.0,
                    piece.width * target.texel,
                    target.source_h,
                )),
                flip_x: piece.flipped,
                ..Default::default()
            },
        );
    }
}
