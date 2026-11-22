use super::{Lite, SCALES};

impl Lite {
    pub fn set(&mut self, k: &str, v: &str) -> bool {
        let n = v.parse::<u8>().unwrap_or(0);
        let on = v != "0";
        match k {
            "lite_scale" => self.scale = n.min(SCALES.len() as u8 - 1),
            "lite_particles" => self.particles = n.min(2),
            "lite_afterimage" => self.afterimage = on,
            "lite_glass" => self.glass = on,
            "lite_shadows" => self.shadows = on,
            "lite_backdrop" => self.backdrop = on,
            "lite_crt" => self.crt = on,
            "lite_auto" => self.auto = on,
            _ => return false,
        }
        true
    }

    pub fn to_text(&self) -> String {
        format!(
            "lite_scale={}\nlite_particles={}\nlite_afterimage={}\nlite_glass={}\nlite_shadows={}\nlite_backdrop={}\nlite_crt={}\nlite_auto={}\n",
            self.scale, self.particles, self.afterimage as u8, self.glass as u8, self.shadows as u8, self.backdrop as u8, self.crt as u8, self.auto as u8
        )
    }
}
