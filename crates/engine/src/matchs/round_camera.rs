use super::{Match, V2};

impl Match {
    pub(super) fn reset_round_camera(&mut self) {
        self.zoom = self.zoom_min;
        self.apply_zoom();
        self.camera = V2 { x: self.stage.start_x, y: self.stage.start_y };
        self.camera_target_y = self.camera.y;
        self.zoom_request = None;
    }

    pub(super) fn frame_round_opening(&mut self) {
        if self.power_arena {
            self.update_camera();
            self.camera_target_y = self.camera.y;
        }
    }
}
