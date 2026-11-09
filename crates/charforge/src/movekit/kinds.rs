pub const KINDS: [&str; 8] = ["normal", "crouch", "air", "special", "air_special", "super", "projectile", "throw"];

pub struct KindDefaults {
    pub damage: &'static str,
    pub pausetime: &'static str,
    pub ground_velocity: &'static str,
    pub fall: bool,
    pub guardflag: &'static str,
    pub ground_type: &'static str,
    pub attribute: &'static str,
}

pub fn is_air(kind: &str) -> bool {
    kind == "air" || kind == "air_special"
}

pub fn kind_defaults(kind: &str) -> Result<KindDefaults, String> {
    let (damage, pausetime, velocity, fall, attribute) = match kind {
        "normal" => ("50, 6", "8, 9", "-3.5, 0", false, "NA"),
        "crouch" => ("45, 5", "8, 9", "-3, 0", false, "NA"),
        "air" => ("55, 6", "8, 9", "-3, 0", false, "NA"),
        "special" => ("100, 15", "10, 12", "-5, 0", true, "SA"),
        "air_special" => ("100, 15", "10, 12", "-4.5, 0", true, "SA"),
        "super" => ("280, 40", "14, 16", "-6.5, -2.5", true, "HA"),
        "projectile" => ("70, 10", "6, 8", "-4, 0", false, "SP"),
        "throw" => ("90, 0", "8, 8", "-4, -4", true, "NT"),
        other => return Err(format!("kind must be one of {KINDS:?}, got {other:?}")),
    };
    Ok(KindDefaults {
        damage,
        pausetime,
        ground_velocity: velocity,
        fall,
        guardflag: match kind {
            "crouch" => "L",
            "throw" => "",
            _ => "MA",
        },
        ground_type: if kind == "crouch" { "Low" } else { "High" },
        attribute,
    })
}

pub fn power_cost(kind: &str) -> i64 {
    match kind {
        "special" | "air_special" | "projectile" => 1000,
        "super" => 3000,
        _ => 0,
    }
}
