use super::*;

impl App {
    pub(super) fn title(&mut self) {
        if !draw_title_backdrop(&self.ui) {
            self.menu_art.select(0);
            self.menu_art.draw(&self.ui);
        }
        let tweaks = crate::tweaks::current();
        if tweaks.title_crt {
            crate::title_crt::draw(&self.ui, get_time() as f32, !tweaks.reduce_motion);
        }
        self.ui.text_bold(
            "ULTIMATE MUGEN",
            VW / 2.0,
            64.0,
            50.0,
            hex("#fff2c4"),
            hex("#140e29"),
            Align::Center,
        );
        let button_x = VW / 2.0 - 104.0;
        let hovered = self.hover_rect(button_x, 616.0, 208.0, 48.0);
        self.ui.rect(
            button_x,
            616.0,
            208.0,
            48.0,
            Color::new(0.05, 0.03, 0.09, 0.94),
        );
        self.ui.rect_lines(
            button_x,
            616.0,
            208.0,
            48.0,
            2.0,
            alpha(hex("#ffcc77"), if hovered { 1.0 } else { 0.65 }),
        );
        self.ui.text_bold(
            crate::i18n::tr3("INICIAR", "START", "INICIAR"),
            VW / 2.0,
            648.0,
            23.0,
            WHITE,
            hex("#12051f"),
            Align::Center,
        );
        let hint = if self.c.touch.enabled {
            crate::i18n::tr3("TOQUE PARA INICIAR", "TAP TO START", "TOCA PARA INICIAR")
        } else {
            "START / ENTER"
        };
        self.ui.text(
            hint,
            VW / 2.0,
            685.0,
            14.0,
            hex("#eee4ff"),
            Align::Center,
            true,
        );
        self.draw_update_status();
        if !self.update_prompt_takes_click()
            && (self.c.pressed(nav::OK | nav::START)
                || is_mouse_button_pressed(MouseButton::Left)
                || !touches().is_empty() && touches()[0].phase == TouchPhase::Started)
        {
            self.go(Screen::Menu { cur: 0 });
        }
    }
}

thread_local! {
    static TITLE_BACKDROP: Option<Texture2D> = {
        ume_formats::png::decode_png(include_bytes!("../../assets/ui/title-crt-duel.png"))
            .ok()
            .and_then(|image| {
                Some(Texture2D::from_rgba_shared(image.width as u16, image.height as u16, Arc::new(image.rgba?)))
            })
    };
}

fn draw_title_backdrop(ui: &Ui) -> bool {
    TITLE_BACKDROP.with(|backdrop| {
        let Some(texture) = backdrop else {
            return false;
        };
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), hex("#100a19"));
        draw_texture_ex(
            texture,
            ui.x(0.0),
            ui.y(0.0),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VW * ui.s, VH * ui.s)),
                ..Default::default()
            },
        );
        true
    })
}
