pub const DEFAULT_LASER_COLOR: &str = "red";
pub const LASER_DECAY_TIME_MS: f64 = 1_000.0;
pub const LASER_DECAY_LENGTH: usize = 50;
pub const LASER_STREAMLINE: f32 = 0.4;
pub const LASER_SIMPLIFY: f32 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaserPoint {
    pub position: [f32; 2],
    pub timestamp_ms: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaserTrail {
    pub points: Vec<LaserPoint>,
    pub closed: bool,
    pub keep_head: bool,
}

impl LaserTrail {
    pub fn new(position: [f32; 2], timestamp_ms: f64) -> Self {
        Self { points: vec![LaserPoint { position, timestamp_ms }], closed: false, keep_head: true }
    }

    pub fn add_point(&mut self, position: [f32; 2], timestamp_ms: f64) -> bool {
        if self.points.last().is_some_and(|point| point.position == position) { return false; }
        self.points.push(LaserPoint { position, timestamp_ms });
        true
    }

    pub fn close(&mut self) { self.closed = true; self.keep_head = false; }

    pub fn visible_samples(&self, now_ms: f64) -> Vec<LaserSample> {
        let total = self.points.len();
        self.points.iter().enumerate().filter_map(|(index, point)| {
            let age = (now_ms - point.timestamp_ms).max(0.0);
            let temporal = (1.0 - age / LASER_DECAY_TIME_MS).clamp(0.0, 1.0) as f32;
            let remaining = total.saturating_sub(index + 1);
            let length = LASER_DECAY_LENGTH.saturating_sub(remaining.min(LASER_DECAY_LENGTH)) as f32 / LASER_DECAY_LENGTH as f32;
            let size = ease_out(length).min(ease_out(temporal));
            (size > 0.0).then_some(LaserSample { position: point.position, size })
        }).collect()
    }

    pub fn is_visible(&self, now_ms: f64) -> bool { !self.visible_samples(now_ms).is_empty() }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaserSample { pub position: [f32; 2], pub size: f32 }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LaserTrails { current: Option<LaserTrail>, past: Vec<LaserTrail> }

impl LaserTrails {
    pub fn current(&self) -> Option<&LaserTrail> { self.current.as_ref() }
    pub fn past(&self) -> &[LaserTrail] { &self.past }
    pub fn has_current_trail(&self) -> bool { self.current.is_some() }
    pub fn start_path(&mut self, x: f32, y: f32, timestamp_ms: f64) { self.current = Some(LaserTrail::new([x, y], timestamp_ms)); }
    pub fn add_point_to_path(&mut self, x: f32, y: f32, timestamp_ms: f64) -> bool { self.current.as_mut().is_some_and(|trail| trail.add_point([x, y], timestamp_ms)) }
    pub fn end_path(&mut self) -> bool { let Some(mut trail) = self.current.take() else { return false; }; trail.close(); self.past.push(trail); true }
    pub fn clear(&mut self) { self.current = None; self.past.clear(); }
    pub fn visible_samples(&self, now_ms: f64) -> Vec<LaserSample> { self.past.iter().flat_map(|trail| trail.visible_samples(now_ms)).chain(self.current.iter().flat_map(|trail| trail.visible_samples(now_ms))).collect() }
    pub fn prune(&mut self, now_ms: f64) { self.past.retain(|trail| trail.is_visible(now_ms)); }
}

pub fn ease_out(k: f32) -> f32 { let k = k.clamp(0.0, 1.0); 1.0 - (1.0 - k).powi(4) }
