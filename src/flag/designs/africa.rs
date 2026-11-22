use super::{disc, rect};
use crate::flag::Shape;

pub(super) const FLAGS: &[(&str, &[Shape])] = &[
    (
        "ng",
        &[
            rect(0.0, 0.0, 1.0, 1.0, 0x008751),
            rect(0.333, 0.0, 0.334, 1.0, 0xFFFFFF),
        ],
    ),
    (
        "eg",
        &[
            rect(0.0, 0.0, 1.0, 0.333, 0xCE1126),
            rect(0.0, 0.333, 1.0, 0.334, 0xFFFFFF),
            rect(0.0, 0.667, 1.0, 0.333, 0x000000),
            disc(0.5, 0.5, 0.09, 0xC09300),
        ],
    ),
];
