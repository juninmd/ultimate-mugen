use crate::i18n::tr3;
use crate::session::Session;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    Turbo,
    GlassCannon,
    IronTitan,
    BlitzTimer,
    SuddenDeath,
    PowerSurge,
    BossOverdrive,
}

const POOL: [Modifier; 6] = [
    Modifier::Turbo,
    Modifier::GlassCannon,
    Modifier::IronTitan,
    Modifier::BlitzTimer,
    Modifier::SuddenDeath,
    Modifier::PowerSurge,
];

impl Modifier {
    pub fn for_floor(roll: u64) -> Self {
        POOL[(roll as usize) % POOL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Turbo => tr3("TURBO 125%", "TURBO 125%", "TURBO 125%"),
            Self::GlassCannon => tr3("CANHÃO DE VIDRO", "GLASS CANNON", "CAÑÓN DE CRISTAL"),
            Self::IronTitan => tr3("TITÃ DE FERRO", "IRON TITAN", "TITÁN DE HIERRO"),
            Self::BlitzTimer => tr3("ROUND RELÂMPAGO", "BLITZ ROUND", "ASALTO RELÁMPAGO"),
            Self::SuddenDeath => tr3("MORTE SÚBITA", "SUDDEN DEATH", "MUERTE SÚBITA"),
            Self::PowerSurge => tr3("SOBRECARGA", "POWER SURGE", "SOBRECARGA"),
            Self::BossOverdrive => tr3("CHEFE OVERDRIVE", "BOSS OVERDRIVE", "JEFE OVERDRIVE"),
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Self::Turbo => tr3("Velocidade de combate acelerada em 25%", "Combat speed increased by 25%", "Velocidad de combate +25%"),
            Self::GlassCannon => tr3("Todos os golpes causam +60% de dano", "All strikes deal +60% damage", "Todos los golpes causan +60% de daño"),
            Self::IronTitan => tr3("Oponente com +40% de vida máxima", "Opponent has +40% max life", "Rival con +40% de vida máxima"),
            Self::BlitzTimer => tr3("Rounds rápidos de 30 segundos", "Fast 30-second rounds", "Asaltos rápidos de 30 segundos"),
            Self::SuddenDeath => tr3("Round único · 55% de vida para ambos", "Single round · 55% life for both", "Un solo asalto · 55% de vida"),
            Self::PowerSurge => tr3("Ambos começam com a super barra cheia", "Both fighters start with full meter", "Ambos empiezan con barra llena"),
            Self::BossOverdrive => tr3("Chefe final · +35% vida e +15% dano", "Final boss · +35% life and +15% attack", "Jefe final · +35% vida y +15% ataque"),
        }
    }

    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Turbo => [0.35, 0.88, 1.0],
            Self::GlassCannon => [1.0, 0.38, 0.28],
            Self::IronTitan => [0.75, 0.82, 0.92],
            Self::BlitzTimer => [1.0, 0.82, 0.22],
            Self::SuddenDeath => [1.0, 0.22, 0.45],
            Self::PowerSurge => [0.45, 1.0, 0.6],
            Self::BossOverdrive => [0.95, 0.4, 1.0],
        }
    }

    pub fn apply(self, sess: &mut Session) {
        match self {
            Self::Turbo => {
                sess.set_speed((sess.speed() * 1.25).min(1.5));
            }
            Self::GlassCannon => {
                for f in sess.m.fighters.iter_mut().filter(|f| !f.is_helper) {
                    f.attack_mul *= 1.6;
                }
            }
            Self::IronTitan => {
                for f in sess.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1) {
                    f.life_max *= 1.4;
                    f.life = f.life_max;
                }
            }
            Self::BlitzTimer => {
                sess.m.round_ticks = 30 * 60;
                sess.m.timer = 30 * 60;
            }
            Self::SuddenDeath => {
                sess.m.rounds_to_win = 1;
                for f in sess.m.fighters.iter_mut().filter(|f| !f.is_helper) {
                    f.life_max = (f.life_max * 0.55).max(1.0);
                    f.life = f.life_max;
                }
            }
            Self::PowerSurge => {
                for f in sess.m.fighters.iter_mut().filter(|f| !f.is_helper) {
                    f.power = f.power_max;
                }
            }
            Self::BossOverdrive => {
                for f in sess.m.fighters.iter_mut().filter(|f| !f.is_helper && f.side == 1) {
                    f.life_max *= 1.35;
                    f.life = f.life_max;
                    f.attack_mul *= 1.15;
                }
            }
        }
    }
}
