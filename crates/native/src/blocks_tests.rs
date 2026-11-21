use super::*;
use crate::types::ProjectileSpec;
use serde_json::json;

fn projectile(extra: serde_json::Value) -> ProjectileSpec {
    let mut base =
        json!({ "at": 3, "anim": "orb", "speed": [6, 0], "offset": [40, -60], "damage": 20 });
    base.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}

#[test]
fn repeat_fans_one_projectile_into_a_staggered_volley() {
    let volley = expand(
        &projectile(
            json!({ "repeat": { "count": 4, "every": 2, "offsetStep": [0, 10], "speedStep": [1, 0], "damageStep": 5 } }),
        ),
        20,
    );
    assert_eq!(volley.len(), 4);
    assert_eq!(
        volley.iter().map(|p| p.at).collect::<Vec<_>>(),
        [3, 5, 7, 9]
    );
    assert_eq!(volley[3].offset, [40.0, -30.0]);
    assert_eq!(volley[3].speed, [9.0, 0.0]);
    assert_eq!(volley[3].damage, 35.0);
    assert!(volley.iter().all(|p| p.repeat.is_none()));
}

#[test]
fn repeat_is_bounded_and_a_plain_projectile_is_untouched() {
    assert_eq!(
        expand(&projectile(json!({ "repeat": { "count": 9999 } })), 99).len(),
        MAX_REPEAT as usize
    );
    assert_eq!(expand(&projectile(json!({})), 9).len(), 1);
}

#[test]
fn blink_targets_cover_behind_front_above_and_back() {
    let spec = |to: &str| BlinkSpec {
        at: 2,
        to: Some(to.into()),
        gap: Some(30.0),
        y: None,
    };
    assert!(
        blink(1000, &spec("behind")).contains("p2dist x + 30")
            && blink(1000, &spec("behind")).contains("Turn")
    );
    assert!(
        blink(1000, &spec("front")).contains("p2dist x - 30")
            && !blink(1000, &spec("front")).contains("Turn")
    );
    assert!(blink(1000, &spec("above")).contains("y = -30"));
    assert!(blink(1000, &spec("back")).contains("x = -30"));
}

#[test]
fn vfx_resolves_the_animation_number_and_quake_shakes() {
    let anims = HashMap::from([("flash".to_string(), 7100)]);
    let text = vfx(
        1000,
        &VfxSpec {
            at: 1,
            anim: "flash".into(),
            offset: None,
            time: Some(12),
            scale: Some(1.5),
            follow: Some(true),
            behind: None,
        },
        &anims,
    );
    assert!(
        text.contains("anim = 7100")
            && text.contains("removetime = 12")
            && text.contains("bindtime = 12")
            && text.contains("scale = 1.5, 1.5")
    );
    assert!(quake(
        1000,
        &QuakeSpec {
            at: 2,
            time: None,
            amp: Some(3.0)
        }
    )
    .contains("ampl = -3"));
}

#[test]
fn a_volley_longer_than_the_animation_fans_out_on_its_last_frame() {
    let volley = expand(
        &projectile(json!({ "repeat": { "count": 5, "every": 2 } })),
        6,
    );
    assert_eq!(
        volley.iter().map(|p| p.at).collect::<Vec<_>>(),
        [3, 5, 6, 6, 6]
    );
}
