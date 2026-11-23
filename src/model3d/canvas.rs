use super::bindings::Binding;
use crate::gfx::*;
use std::cell::RefCell;
use std::collections::HashMap;
use ume_model3d::{Facing, Raster};

const EDGE: usize = 448;
const FEET: f32 = 0.84;
const FILL: f32 = 0.5;

struct Canvas {
    raster: Raster,
    samples: usize,
    pixels: Vec<u8>,
    texture: Option<Texture2D>,
}

pub(super) struct Placement {
    pub facing: Facing,
    pub screen_scale: f32,
    pub feet: [f32; 2],
    pub samples: usize,
}

thread_local! {
    static CANVASES: RefCell<HashMap<(String, bool, usize), Canvas>> = RefCell::new(HashMap::new());
}

pub(super) struct Drawn {
    pub texture: Texture2D,
    pub corner: [f32; 2],
    pub size: f32,
}

impl Drawn {
    pub fn draw(&self, offset: [f32; 2], tint: Color) {
        draw_texture_ex(
            &self.texture,
            self.corner[0] + offset[0],
            self.corner[1] + offset[1],
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(self.size, self.size)),
                ..Default::default()
            },
        );
    }
}

pub(super) fn present(b: &mut Binding, key: (&str, bool, usize), at: Placement) -> Drawn {
    let render_scale = at.screen_scale.min(EDGE as f32 * FILL / b.height);
    let mut camera = b.camera;
    camera.pixels_per_unit *= render_scale;
    let origin = [EDGE as f32 / 2.0, EDGE as f32 * FEET];
    b.model
        .project(&b.posed, &camera, at.facing, origin, &mut b.triangles);
    let drawn = CANVASES.with(|cache| {
        let mut cache = cache.borrow_mut();
        let samples = if crate::lite::scale() < 1.0 {
            1
        } else {
            at.samples
        };
        let canvas = cache
            .entry((key.0.to_string(), key.1, key.2))
            .or_insert_with(|| Canvas {
                raster: Raster::new(EDGE, EDGE, samples),
                samples,
                pixels: vec![0; EDGE * EDGE * 4],
                texture: None,
            });
        if canvas.samples != samples {
            canvas.raster = Raster::new(EDGE, EDGE, samples);
            canvas.samples = samples;
        }
        canvas.raster.draw(&b.triangles, &b.model.images);
        canvas.raster.resolve(&mut canvas.pixels);
        let texture = match &canvas.texture {
            Some(texture) => {
                texture.rewrite(&canvas.pixels);
                texture.clone()
            }
            None => {
                let texture = Texture2D::from_rgba8(EDGE as u16, EDGE as u16, &canvas.pixels);
                canvas.texture = Some(texture.clone());
                texture
            }
        };
        let dest = EDGE as f32 * at.screen_scale / render_scale;
        Drawn {
            texture,
            corner: [at.feet[0] - dest / 2.0, at.feet[1] - dest * FEET],
            size: dest,
        }
    });
    drawn
}
