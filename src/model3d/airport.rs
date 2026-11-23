use crate::gfx::*;
use std::cell::RefCell;
use std::sync::{Arc, OnceLock};
use ume_engine::chardef::ImageData;
use ume_model3d::model::Posed;
use ume_model3d::{Lens, Look, Model, Raster, Triangle};

const GLB: &[u8] = include_bytes!("../../assets/model3d/airport.glb");
const LOOP_SECONDS: f32 = 40.0;
const STILL_SECONDS: f32 = 7.5;
const FULL: (usize, usize) = (480, 270);
const STILL_SIZE: (usize, usize) = (640, 360);
const LITE: (usize, usize) = (320, 180);
const TICKS_PER_FRAME: i64 = 2;

const LOOK: Look = Look {
    toward_light: [-0.5, 0.7, 0.5],
    light: [0.62, 0.52, 0.38],
    ambient: [0.55, 0.58, 0.72],
    fog: [0.99, 0.74, 0.60],
    fog_density: 0.0016,
    sky: [[0.20, 0.24, 0.52], [0.99, 0.72, 0.52]],
};

struct Backdrop {
    model: Model,
    clip: Option<usize>,
    posed: Posed,
    triangles: Vec<Triangle>,
}

struct Target {
    size: (usize, usize),
    raster: Raster,
    pixels: Vec<u8>,
    texture: Option<Texture2D>,
    stamp: i64,
}

thread_local! {
    static BACKDROP: RefCell<Option<Option<Backdrop>>> = const { RefCell::new(None) };
    static TARGET: RefCell<Option<Target>> = const { RefCell::new(None) };
}

fn lens(size: (usize, usize)) -> Lens {
    Lens {
        eye: [0.0, 12.0, 105.0],
        target: [0.0, 15.0, -60.0],
        fov_y: 0.66,
        width: size.0 as f32,
        height: size.1 as f32,
    }
}

impl Backdrop {
    fn load() -> Option<Backdrop> {
        let model = Model::from_glb(GLB).ok()?;
        Some(Backdrop {
            clip: model.clip_index("ambient"),
            model,
            posed: Posed::default(),
            triangles: Vec::new(),
        })
    }

    fn render(
        &mut self,
        seconds: f32,
        raster: &mut Raster,
        size: (usize, usize),
        pixels: &mut [u8],
    ) {
        let t = seconds.rem_euclid(LOOP_SECONDS);
        self.model.pose(self.clip, t, &mut self.posed);
        self.model
            .project_scene(&self.posed, &lens(size), &LOOK, &mut self.triangles);
        raster.draw(&self.triangles, &self.model.images);
        raster.resolve(pixels);
    }
}

fn with_backdrop<R>(run: impl FnOnce(&mut Backdrop) -> R) -> Option<R> {
    BACKDROP.with(|slot| {
        let mut slot = slot.borrow_mut();
        slot.get_or_insert_with(Backdrop::load).as_mut().map(run)
    })
}

fn target_size() -> (usize, usize) {
    if crate::lite::scale() < 1.0 {
        LITE
    } else {
        FULL
    }
}

pub(super) fn draw(time: i64, x: f32, y: f32, w: f32, h: f32) -> bool {
    let size = target_size();
    let stamp = time / TICKS_PER_FRAME;
    let texture = TARGET.with(|cell| {
        let mut cell = cell.borrow_mut();
        if cell.as_ref().is_some_and(|t| t.size != size) {
            *cell = None;
        }
        let target = cell.get_or_insert_with(|| Target {
            size,
            raster: Raster::new(size.0, size.1, 1),
            pixels: vec![0; size.0 * size.1 * 4],
            texture: None,
            stamp: i64::MIN,
        });
        if target.stamp != stamp || target.texture.is_none() {
            let rendered = with_backdrop(|b| {
                b.render(
                    time as f32 / 60.0,
                    &mut target.raster,
                    size,
                    &mut target.pixels,
                )
            });
            rendered?;
            target.stamp = stamp;
            match &target.texture {
                Some(texture) => texture.rewrite(&target.pixels),
                None => {
                    target.texture = Some(Texture2D::from_rgba8(
                        size.0 as u16,
                        size.1 as u16,
                        &target.pixels,
                    ))
                }
            }
        }
        target.texture.clone()
    });
    let Some(texture) = texture else {
        return false;
    };
    draw_texture_ex(
        &texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
    true
}

pub(super) fn still() -> Option<Arc<ImageData>> {
    static STILL: OnceLock<Option<Arc<ImageData>>> = OnceLock::new();
    STILL
        .get_or_init(|| {
            Some(ImageData::lazy(
                STILL_SIZE.0 as u32,
                STILL_SIZE.1 as u32,
                || {
                    let (w, h) = STILL_SIZE;
                    let mut raster = Raster::new(w, h, 2);
                    let mut pixels = vec![0u8; w * h * 4];
                    if let Some(mut backdrop) = Backdrop::load() {
                        backdrop.render(STILL_SECONDS, &mut raster, (w, h), &mut pixels);
                    }
                    Arc::new(pixels)
                },
            ))
        })
        .clone()
}
