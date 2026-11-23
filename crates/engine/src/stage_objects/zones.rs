use super::ObjectEvent;
use crate::matchs::Match;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StageZoneKind {
    Updraft { lift: f32 },
    PowerWell { meter_per_tick: f32 },
    BurnHazard { damage: f32, interval: i32 },
    IceFloor { boost: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct StageZone {
    pub kind: StageZoneKind,
    pub x: f32,
    pub half_width: f32,
    pub lane: f32,
    pub depth: f32,
    pub active: bool,
    pub tick: i32,
}

impl StageZone {
    pub fn new(kind: StageZoneKind, x: f32, half_width: f32) -> Self {
        StageZone {
            kind,
            x,
            half_width: half_width.max(4.0),
            lane: 0.0,
            depth: 12.0,
            active: true,
            tick: 0,
        }
    }

    pub fn contains_x(&self, px: f32) -> bool {
        (px - self.x).abs() <= self.half_width
    }
}

impl Match {
    pub(super) fn step_zones(&mut self) {
        let count = self.objects.zones.len();
        if count == 0 {
            return;
        }
        for zi in 0..count {
            if !self.objects.zones[zi].active {
                continue;
            }
            self.objects.zones[zi].tick = self.objects.zones[zi].tick.saturating_add(1);
            let zone_x = self.objects.zones[zi].x;
            let zone_hw = self.objects.zones[zi].half_width;
            let zone_lane = self.objects.zones[zi].lane;
            let zone_depth = self.objects.zones[zi].depth;
            let zone_kind = self.objects.zones[zi].kind;
            let zone_tick = self.objects.zones[zi].tick;

            for fi in 0..self.fighters.len() {
                if self.fighters[fi].is_helper || !self.fighters[fi].alive {
                    continue;
                }
                if !self.object_in_reach(fi, zone_lane, zone_depth) {
                    continue;
                }
                let px = self.fighters[fi].pos.x;
                if (px - zone_x).abs() > zone_hw {
                    continue;
                }
                let fid = self.fighters[fi].id;
                match zone_kind {
                    StageZoneKind::Updraft { lift } => {
                        if self.fighters[fi].pos.y < 0.0 || self.fighters[fi].vel.y < 0.0 {
                            self.fighters[fi].vel.y = (self.fighters[fi].vel.y - lift).max(-12.0);
                            if zone_tick % 12 == 0 {
                                self.objects.emit(ObjectEvent::ZoneTriggered {
                                    index: zi,
                                    fighter: fid,
                                    x: px,
                                    lane: zone_lane,
                                });
                            }
                        }
                    }
                    StageZoneKind::PowerWell { meter_per_tick } => {
                        let before = self.fighters[fi].power;
                        self.fighters[fi].add_power_local(meter_per_tick);
                        if self.fighters[fi].power > before && zone_tick % 15 == 0 {
                            self.objects.emit(ObjectEvent::ZoneTriggered {
                                index: zi,
                                fighter: fid,
                                x: px,
                                lane: zone_lane,
                            });
                        }
                    }
                    StageZoneKind::BurnHazard { damage, interval } => {
                        let step = interval.max(1);
                        if self.fighters[fi].pos.y >= -4.0 && zone_tick % step == 0 {
                            let next = (self.fighters[fi].life - damage.max(0.0)).max(1.0);
                            if next < self.fighters[fi].life {
                                self.fighters[fi].life = next;
                                self.objects.emit(ObjectEvent::ZoneTriggered {
                                    index: zi,
                                    fighter: fid,
                                    x: px,
                                    lane: zone_lane,
                                });
                            }
                        }
                    }
                    StageZoneKind::IceFloor { boost } => {
                        if self.fighters[fi].pos.y >= -2.0 && self.fighters[fi].vel.x.abs() > 0.25 {
                            let mul = 1.0 + boost.clamp(0.0, 0.25);
                            self.fighters[fi].vel.x =
                                (self.fighters[fi].vel.x * mul).clamp(-14.0, 14.0);
                        }
                    }
                }
            }
        }
    }
}
