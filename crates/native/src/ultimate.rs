pub(super) const ID_PREFIX: &str = "ult_";
pub(super) const COST: f32 = 3000.0;
pub(super) const DAMAGE: f32 = 1.7;
pub(super) const FREEZE: i32 = 75;

pub(super) fn is(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}
