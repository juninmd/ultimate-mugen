use ume_shell::quad::*;
use ume_shell::Config;

async fn game() {
    let font =
        load_ttf_font_from_bytes(include_bytes!("../../../assets/fonts/Rajdhani.ttf")).unwrap();
    set_default_font(font.clone());
    for i in 0..30 {
        clear_background(BLACK);
        let w = screen_width();
        draw_rectangle(
            w * 0.2,
            screen_height() * 0.5,
            w * 0.6 * i as f32 / 29.0,
            12.0,
            ORANGE,
        );
        draw_text(
            "Loading...",
            w * 0.2,
            screen_height() * 0.5 - 16.0,
            32.0,
            WHITE,
        );
        next_frame().await;
    }
    let pixels: Vec<u8> = (0..64 * 64)
        .flat_map(|i| {
            let (x, y) = (i % 64, i / 64);
            [x as u8 * 4, y as u8 * 4, 200, 255]
        })
        .collect();
    let tex = Texture2D::from_rgba8(64, 64, &pixels);
    let mut angle = 0.0f32;
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        angle += get_frame_time();
        clear_background(Color::from_rgba(20, 22, 36, 255));
        let (w, h) = (screen_width(), screen_height());
        draw_texture_ex(
            &tex,
            w * 0.25 - 96.0,
            h * 0.5 - 96.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(192.0, 192.0)),
                rotation: angle,
                ..Default::default()
            },
        );
        set_blend(Blend::Add);
        draw_circle(w * 0.5, h * 0.5, 90.0, Color::new(0.2, 0.4, 1.0, 1.0));
        draw_circle(
            w * 0.5 + 60.0,
            h * 0.5,
            90.0,
            Color::new(1.0, 0.3, 0.2, 1.0),
        );
        set_blend(Blend::Alpha);
        let c = [
            Color::new(1.0, 0.0, 0.0, 1.0),
            Color::new(0.0, 1.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 1.0, 1.0),
        ];
        draw_mesh(&Mesh {
            vertices: (0..3)
                .map(|i| {
                    let t = i as f32 * 2.094 + angle;
                    Vertex::new(
                        w * 0.78 + t.cos() * 110.0,
                        h * 0.5 + t.sin() * 110.0,
                        0.0,
                        0.0,
                        0.0,
                        c[i],
                    )
                })
                .collect(),
            indices: vec![0, 1, 2],
            texture: None,
        });
        draw_ellipse(
            w * 0.5,
            h * 0.85,
            160.0,
            18.0,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.5),
        );
        let label = "macroquad-style API on wgpu";
        let d = measure_text(label, Some(&font), 40, 1.0);
        draw_text_ex(
            label,
            (w - d.width) * 0.5,
            70.0,
            TextParams {
                font: Some(&font),
                font_size: 40,
                color: YELLOW,
                ..Default::default()
            },
        );
        draw_rectangle_lines(20.0, 20.0, w - 40.0, h - 40.0, 3.0, WHITE);
        next_frame().await;
    }
}

fn main() {
    let max_frames = std::env::var("UME_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    if let Err(e) = run_async(
        Config {
            title: "ume-shell quad".into(),
            max_frames,
            ..Default::default()
        },
        game(),
    ) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
