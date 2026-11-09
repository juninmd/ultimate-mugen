use super::regions::{Mask, Region};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AxisMode {
    Feet,
    RowFeet,
    BottomCenter,
    Center,
}

impl AxisMode {
    pub fn parse(text: &str) -> Result<AxisMode, String> {
        match text {
            "feet" => Ok(AxisMode::Feet),
            "row-feet" => Ok(AxisMode::RowFeet),
            "bottom-center" => Ok(AxisMode::BottomCenter),
            "center" => Ok(AxisMode::Center),
            other => Err(format!("axis mode must be feet, row-feet, bottom-center or center, got {other}")),
        }
    }
}

pub fn assign_axes(regions: &mut [Region], mask: &Mask, mode: AxisMode, feet_band: usize) {
    let mut baseline: HashMap<usize, usize> = HashMap::new();
    for region in regions.iter() {
        let entry = baseline.entry(region.row).or_default();
        *entry = (*entry).max(region.bottom - 1);
    }
    for region in regions.iter_mut() {
        if region.fixed_axis {
            continue;
        }
        let (width, height) = (region.width() as i32, region.height() as i32);
        match mode {
            AxisMode::Center => {
                region.axis_x = width / 2;
                region.axis_y = height / 2;
            }
            AxisMode::BottomCenter => {
                region.axis_x = width / 2;
                region.axis_y = height - 1;
            }
            AxisMode::Feet | AxisMode::RowFeet => {
                let first = region.bottom.saturating_sub(feet_band).max(region.top);
                let columns: Vec<usize> = (region.left..region.right)
                    .filter(|&x| (first..region.bottom).any(|y| mask.at(x, y)))
                    .map(|x| x - region.left)
                    .collect();
                region.axis_x = match (columns.first(), columns.last()) {
                    (Some(a), Some(b)) => ((*a + *b) as f64 / 2.0).round() as i32,
                    _ => width / 2,
                };
                region.axis_y = baseline[&region.row] as i32 - region.top as i32;
            }
        }
    }
}
