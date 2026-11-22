use super::quality;
use super::Vfx;
use crate::i18n::tr3;
use ume_engine::matchs::Match;

impl Vfx {
    pub(super) fn on_elixir(&mut self, fighter: u32, m: &Match) {
        let Some(f) = m.fighters.iter().find(|f| f.id == fighter) else { return };
        let (x, y, _) = Self::chest_of(f);
        let green = [0.5, 1.0, 0.77];
        self.ring(x, y, 80.0, 0.6, 2.4, green);
        self.ring(x, y, 150.0, 0.8, 1.6, [1.0, 0.9, 0.5]);
        self.glow(x, y, 46.0, 0.4, green);
        self.spark_burst(x, y, 1.0, quality::count(28), 8.0, green, std::f32::consts::PI);
        let name = f.def.name.to_uppercase();
        self.banner(tr3("ELIXIR DO PODER", "POWER ELIXIR", "ELIXIR DEL PODER"), &format!("{name} · {}", tr3("VIDA x2", "LIFE x2", "VIDA x2")), green);
    }
}
