use super::*;
use crate::i18n::tr3;
use std::cell::RefCell;

thread_local! {
    static MONITOR: RefCell<crate::lite::Monitor> = RefCell::new(crate::lite::Monitor::default());
}

impl App {
    pub(super) fn lite_tick(&mut self) {
        let fighting = matches!(self.screen, Screen::Fight) && self.settings.lite.auto;
        MONITOR.with(|m| {
            let mut m = m.borrow_mut();
            if !fighting {
                m.reset();
                return;
            }
            if m.feed(get_frame_time()) && self.settings.lighter_step() {
                self.msg = tr3(
                    "Desempenho baixo: gráficos reduzidos automaticamente",
                    "Low frame rate: graphics lowered automatically",
                    "Rendimiento bajo: gráficos reducidos automáticamente",
                )
                .into();
                self.settings.save();
                m.reset();
            }
        });
    }
}
