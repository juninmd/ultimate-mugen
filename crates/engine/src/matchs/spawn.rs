use super::Match;

const SLOT_WIDTH: f32 = 36.0;
const SPAN_BASE: f32 = 100.0;
const EDGE_MARGIN: f32 = 30.0;

impl Match {
    pub(super) fn random_spawns(&mut self, team_n: [usize; 2]) -> Option<Vec<(f32, f32)>> {
        let total = team_n[0] + team_n[1];
        if !self.team_simul || team_n[0].max(team_n[1]) < 2 {
            return None;
        }
        let roots: Vec<usize> = (0..self.fighters.len())
            .filter(|&i| !self.fighters[i].is_helper)
            .collect();
        let center = (self.stage.p1x + self.stage.p2x) * 0.5;
        let reach = (SLOT_WIDTH * total as f32 + SPAN_BASE) * 0.5;
        let lo = (center - reach).max(self.stage.bound_left - self.view_w * 0.5 + EDGE_MARGIN);
        let hi = (center + reach).min(self.stage.bound_right + self.view_w * 0.5 - EDGE_MARGIN);
        let slot = (hi - lo) / total as f32;
        let mut order: Vec<usize> = (0..total).collect();
        for k in (1..total).rev() {
            let j = (self.random() * (k + 1) as f32) as usize;
            order.swap(k, j.min(k));
        }
        let xs: Vec<f32> = order
            .iter()
            .map(|&s| lo + (s as f32 + 0.5) * slot + (self.random() - 0.5) * slot * 0.5)
            .collect();
        let mean = |side: usize, xs: &[f32], roots: &[usize], m: &Match| {
            let (sum, n) = roots
                .iter()
                .zip(xs)
                .filter(|(&i, _)| m.fighters[i].side.min(1) == side)
                .fold((0.0, 0.0), |a, (_, &x)| (a.0 + x, a.1 + 1.0));
            if n > 0.0 {
                sum / n
            } else {
                center
            }
        };
        let foe_mean = [mean(1, &xs, &roots, self), mean(0, &xs, &roots, self)];
        Some(
            roots
                .iter()
                .zip(&xs)
                .map(|(&i, &x)| {
                    let side = self.fighters[i].side.min(1);
                    let face = if foe_mean[side] >= x { 1.0 } else { -1.0 };
                    (x, face)
                })
                .collect(),
        )
    }
}
