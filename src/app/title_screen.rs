use super::*;

impl App {
    pub(super) fn title(&mut self) {
        self.menu_art.select(0);
        self.menu_art.draw(&self.ui);
        self.ui.gradient_v(
            0.0,
            300.0,
            VW,
            VH - 300.0,
            Color::new(0.03, 0.02, 0.07, 0.0),
            Color::new(0.03, 0.02, 0.07, 0.92),
        );
        self.ui.logo(VW / 2.0, 330.0, 230.0);
        let button_x = VW / 2.0 - 206.0;
        let hovered = self.hover_rect(button_x, 490.0, 412.0, 60.0);
        self.ui.adapted_row(button_x, 490.0, 412.0, 60.0, true);
        self.ui.rect_lines(
            button_x,
            490.0,
            412.0,
            60.0,
            2.0,
            alpha(hex("#ffcc33"), if hovered { 1.0 } else { 0.48 }),
        );
        self.ui.text_bold(
            crate::i18n::tr3("INICIAR", "START", "INICIAR"),
            VW / 2.0,
            530.0,
            30.0,
            WHITE,
            hex("#12051f"),
            Align::Center,
        );
        self.ui.text(
            "START / ENTER",
            VW / 2.0,
            578.0,
            16.0,
            hex("#e0d8ef"),
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
