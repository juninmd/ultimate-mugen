use super::*;

pub fn from_mugen(mut st: MugenStage) -> Stage {
    let scl = 320.0 / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    // eixo Y: cenarios 1280x720 (16:9) precisam de 240/720 para ocupar a altura toda
    let scl_y = 240.0 / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    let info = StageInfo {
        name: st.name.clone(),
        bound_left: st.bound_left * scl,
        bound_right: st.bound_right * scl,
        bound_high: st.bound_high * scl_y,
        bound_low: st.bound_low * scl_y,
        p1x: st.p1x * scl,
        p2x: st.p2x * scl,
        zoffset: st.zoffset * scl_y,
        vertical_follow: st.vertical_follow,
        tension: st.tension * scl,
        start_x: st.start_x * scl,
        start_y: st.start_y * scl_y,
        zoom_out: st.zoom_out,
        zoom_in: st.zoom_in,
        camera: ume_engine::matchs::CameraParams {
            vertical_follow_delta: st.vertical_follow_delta * scl_y,
            y_scroll_speed: st.y_scroll_speed,
            zoom_in_speed: st.zoom_in_speed,
            zoom_out_speed: st.zoom_out_speed,
            lowest_cap: st.lowest_cap,
            dynamic_bound_high: st.dynamic_bound_high,
            bound_high_delta: st.bound_high_delta * scl_y,
        },
        auto_turn: st.auto_turn,
    };
    let images = st
        .sprites
        .iter_mut()
        .filter(|(_, im)| im.w > 0 && im.h > 0)
        .map(|(k, im)| (*k, ImageData::new(im.w, im.h, std::mem::take(&mut im.rgba))))
        .collect();
    Stage {
        info,
        art: StageArt::Mugen(Box::new(MugenBg {
            stage: Arc::new(st),
            images,
            scl,
            scl_y,
        })),
        music: 0,
        kind: StageKind::Custom,
        bgm: None,
    }
}
