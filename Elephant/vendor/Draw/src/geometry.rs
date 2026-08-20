pub const MIN_ZOOM: f32 = 0.1;
pub const MAX_ZOOM: f32 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub zoom: f32,
    pub pan: [f32; 2],
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: [0.0, 0.0],
        }
    }
}

impl Viewport {
    pub fn to_world(self, point: [f32; 2]) -> [f32; 2] {
        let zoom = self.zoom.max(f32::EPSILON);
        [
            (point[0] - self.pan[0]) / zoom,
            (point[1] - self.pan[1]) / zoom,
        ]
    }

    pub fn to_screen(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.pan[0] + point[0] * self.zoom,
            self.pan[1] + point[1] * self.zoom,
        ]
    }

    pub fn pan_by(&mut self, delta: [f32; 2]) {
        self.pan[0] += delta[0];
        self.pan[1] += delta[1];
    }

    pub fn zoom_at(&mut self, screen_point: [f32; 2], factor: f32) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let before = self.to_world(screen_point);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = [
            screen_point[0] - before[0] * self.zoom,
            screen_point[1] - before[1] * self.zoom,
        ];
    }

    pub fn set_zoom_at(&mut self, screen_point: [f32; 2], zoom: f32) {
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        let factor = zoom / self.zoom.max(f32::EPSILON);
        self.zoom_at(screen_point, factor);
    }
}

pub fn distance_to_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return ((point[0] - start[0]).powi(2) + (point[1] - start[1]).powi(2)).sqrt();
    }
    let t = (((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / length_squared)
        .clamp(0.0, 1.0);
    let projection = [start[0] + t * dx, start[1] + t * dy];
    ((point[0] - projection[0]).powi(2) + (point[1] - projection[1]).powi(2)).sqrt()
}

pub fn rotate_point(point: [f32; 2], center: [f32; 2], angle: f32) -> [f32; 2] {
    if angle.abs() <= f32::EPSILON {
        return point;
    }
    let sin = angle.sin();
    let cos = angle.cos();
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos]
}

pub fn rgba(value: &str, opacity: f32) -> [u8; 4] {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") || value.is_empty() {
        return [0, 0, 0, 0];
    }
    let value = value.trim_start_matches('#');
    let (value, alpha) = match value.len() {
        3 => (value.chars().flat_map(|c| [c, c]).collect::<String>(), 255),
        6 => (value.to_owned(), 255),
        8 => (
            value[..6].to_owned(),
            u8::from_str_radix(&value[6..], 16).unwrap_or(255),
        ),
        _ => return [0, 0, 0, 0],
    };
    let (r, g, b) = if value.len() == 6 {
        (
            u8::from_str_radix(&value[..2], 16).unwrap_or(0),
            u8::from_str_radix(&value[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&value[4..6], 16).unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };
    [
        r,
        g,
        b,
        (f32::from(alpha) * opacity.clamp(0.0, 100.0) / 100.0).round() as u8,
    ]
}
