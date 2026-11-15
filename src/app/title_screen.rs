use super::*;

impl App {
    pub(super) fn title(&mut self) {
        self.menu_art.select(0);
        self.menu_art.draw(&self.ui);
        self.ui.logo(VW / 2.0, 330.0, 200.0);
        self.ui.text(
            crate::i18n::tr3("ENGINE DE LUTA · RUST · DESKTOP · WEB · MOBILE", "FIGHTING ENGINE · RUST · DESKTOP · WEB · MOBILE", "MOTOR DE LUCHA · RUST · ESCRITORIO · WEB · MÓVIL"),
            VW / 2.0,
            420.0,
            20.0,
            hex("#a49cc0"),
            Align::Center,
            true,
        );
        if (self.ui.t * 2.0) as i32 % 2 == 0 {
            self.ui.text_bold(
                crate::i18n::tr3("PRESSIONE START / ENTER", "PRESS START / ENTER", "PULSA START / ENTER"),
                VW / 2.0,
                540.0,
                30.0,
                WHITE,
                hex("#12051f"),
                Align::Center,
            );
        }
        self.ui.text(
            crate::i18n::tr3("Compatível com personagens M.U.G.E.N  ·  DualShock 4 / DualSense com vibração", "Compatible with M.U.G.E.N characters  ·  DualShock 4 / DualSense with rumble", "Compatible con personajes M.U.G.E.N  ·  DualShock 4 / DualSense con vibración"),
            VW / 2.0,
            690.0,
            18.0,
            hex("#a49cc0"),
            Align::Center,
            false,
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
