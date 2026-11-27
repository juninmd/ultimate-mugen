use super::Vfx;

impl Vfx {
    pub(super) fn reset_round_presentation(&mut self) {
        self.sheets.clear();
        self.slow_t = 0.0;
        self.ko_t = 0.0;
        self.first_hit = false;
        self.cinema.reset();
        self.shot = Default::default();
        self.punch = 0.0;
        self.sz = 0.0;
        self.super_active = false;
        self.super_t = 0.0;
        self.trauma = 0.0;
    }
}
