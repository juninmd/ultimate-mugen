use super::*;

/// Roster scroll thumb: the page indicator the mouse can grab.
///
/// The grid pages by rows (`sel_row`), so dragging the thumb jumps straight to that row instead of wheel-stepping
/// one page at a time. Keyboard navigation keeps working; this only adds a direct pointer route.
impl App {
    pub(super) fn roster_scroll_thumb(&mut self, gx: f32, gy: f32, gw: f32, gh: f32, total_rows: usize) {
        let rows = self.grid_rows().max(1);
        if total_rows <= rows {
            return;
        }
        let (bx, bw) = (gx + gw + 3.0, 5.0);
        let max_row = (total_rows - rows) as f32;
        let thumb_h = (gh * rows as f32 / total_rows as f32).max(18.0);
        let travel = (gh - thumb_h).max(0.0);
        let pointer = self.ui.to_virtual(mouse_position().into());
        let inside = pointer.x >= bx - 4.0 && pointer.x <= bx + bw + 4.0 && pointer.y >= gy && pointer.y <= gy + gh;
        if inside && is_mouse_button_pressed(MouseButton::Left) {
            let ratio = ((pointer.y - gy - thumb_h * 0.5) / travel.max(1.0)).clamp(0.0, 1.0);
            self.sel_row = (ratio * max_row).round() as usize;
        }
        let hover = inside || is_mouse_button_down(MouseButton::Left);
        self.ui.rect(bx, gy, bw, gh, Color::new(1.0, 1.0, 1.0, if hover { 0.14 } else { 0.1 }));
        let y = gy + travel * (if max_row > 0.0 { self.sel_row as f32 / max_row } else { 0.0 });
        self.ui.rect(bx, y, bw, thumb_h, hex("#ff4d6d"));
    }
}
