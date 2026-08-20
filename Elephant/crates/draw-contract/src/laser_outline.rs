use crate::{
    LaserPoint, LaserTrail, LaserTrails, LASER_DECAY_LENGTH, LASER_DECAY_TIME_MS, LASER_SIZE,
    LASER_STREAMLINE,
};

const CORNER_DETECTION_MAX_ANGLE_DEGREES: f32 = 75.0;

#[derive(Clone, Copy)]
struct WorkPoint {
    position: [f32; 2],
    timestamp_ms: f64,
}

impl From<LaserPoint> for WorkPoint {
    fn from(value: LaserPoint) -> Self {
        Self {
            position: value.position,
            timestamp_ms: value.timestamp_ms,
        }
    }
}

impl LaserTrail {
    /// Port of `@excalidraw/laser-pointer`'s `getStrokeOutline()` for the
    /// exact options used by current Excalidraw's local laser trail.
    pub fn stroke_outline(&self, now_ms: f64, size_override: Option<f32>) -> Vec<[f32; 2]> {
        let points = streamlined_points(&self.points);
        let len = points.len();
        if len == 0 {
            return Vec::new();
        }
        let base_size = size_override.unwrap_or(LASER_SIZE);
        let get_size = |point: WorkPoint, index: usize, _running_length: f32| {
            base_size * size_mapping(point.timestamp_ms, index, len, now_ms)
        };

        if len == 1 {
            let c = points[0];
            let size = get_size(c, 0, 0.0);
            if size < 0.5 {
                return Vec::new();
            }
            let mut output = circle_points(c.position, size, 0.0, std::f32::consts::TAU);
            output.push([c.position[0] + size, c.position[1]]);
            return output;
        }

        if len == 2 {
            let c = points[0];
            let n = points[1];
            let c_size = get_size(c, 0, 0.0);
            let n_size = get_size(n, 1, 0.0);
            if c_size < 0.5 || n_size < 0.5 {
                return Vec::new();
            }
            let p_angle = angle(c.position, [c.position[0], c.position[1] - 100.0], n.position);
            let mut output =
                circle_points(c.position, c_size, p_angle, std::f32::consts::PI + p_angle);
            output.extend(circle_points(
                n.position,
                n_size,
                std::f32::consts::PI + p_angle,
                std::f32::consts::TAU + p_angle,
            ));
            if let Some(first) = output.first().copied() {
                output.push(first);
            }
            return output;
        }

        let mut forward = Vec::new();
        let mut backward = Vec::new();
        let mut speed = 0.0_f32;
        let mut prev_speed = 0.0_f32;
        let mut visible_start_index = 0usize;
        let mut running_length = 0.0_f32;

        for index in 1..len - 1 {
            let p = points[index - 1];
            let c = points[index];
            let n = points[index + 1];
            let d = distance(p.position, c.position);
            running_length += d;
            speed = prev_speed + (d - prev_speed) * 0.2;
            let c_size = get_size(c, index, running_length);
            if c_size == 0.0 {
                visible_start_index = index + 1;
                continue;
            }

            let dir_pc = normalize(sub(p.position, c.position));
            let dir_nc = normalize(sub(n.position, c.position));
            let p1dir_pc = rotate(dir_pc, std::f32::consts::FRAC_PI_2);
            let p2dir_pc = rotate(dir_pc, -std::f32::consts::FRAC_PI_2);
            let p1dir_nc = rotate(dir_nc, std::f32::consts::FRAC_PI_2);
            let p2dir_nc = rotate(dir_nc, -std::f32::consts::FRAC_PI_2);

            let p1_pc = add(c.position, scale(p1dir_pc, c_size));
            let p2_pc = add(c.position, scale(p2dir_pc, c_size));
            let p1_nc = add(c.position, scale(p1dir_nc, c_size));
            let p2_nc = add(c.position, scale(p2dir_nc, c_size));
            let ft_dir = add(p1dir_pc, p2dir_nc);
            let bt_dir = add(p2dir_pc, p1dir_nc);
            let pa_pc = add(
                c.position,
                scale(
                    if magnitude(ft_dir) == 0.0 {
                        dir_pc
                    } else {
                        normalize(ft_dir)
                    },
                    c_size,
                ),
            );
            let pa_nc = add(
                c.position,
                scale(
                    if magnitude(bt_dir) == 0.0 {
                        dir_nc
                    } else {
                        normalize(bt_dir)
                    },
                    c_size,
                ),
            );

            let c_angle = norm_angle(angle(c.position, p.position, n.position));
            let variance = if speed > 35.0 { 0.5 } else { 1.0 };
            let detection_angle = CORNER_DETECTION_MAX_ANGLE_DEGREES.to_radians() * variance;
            if c_angle.abs() < detection_angle {
                let turn_angle = norm_angle(std::f32::consts::PI - c_angle).abs();
                if turn_angle == 0.0 {
                    continue;
                }
                if c_angle < 0.0 {
                    backward.push(p2_pc);
                    backward.push(pa_nc);
                    for step in 0..=4 {
                        let theta = turn_angle * step as f32 / 4.0;
                        forward.push(add(c.position, rotate(scale(p1dir_pc, c_size), theta)));
                    }
                    for step in (0..=4).rev() {
                        let theta = turn_angle * step as f32 / 4.0;
                        backward.push(add(c.position, rotate(scale(p1dir_pc, c_size), theta)));
                    }
                    backward.push(pa_nc);
                    backward.push(p1_nc);
                } else {
                    forward.push(p1_pc);
                    forward.push(pa_pc);
                    for step in 0..=4 {
                        let theta = turn_angle * step as f32 / 4.0;
                        backward.push(add(
                            c.position,
                            rotate(scale(p1dir_pc, -c_size), -theta),
                        ));
                    }
                    for step in (0..=4).rev() {
                        let theta = turn_angle * step as f32 / 4.0;
                        forward.push(add(
                            c.position,
                            rotate(scale(p1dir_pc, -c_size), -theta),
                        ));
                    }
                    forward.push(pa_pc);
                    forward.push(p2_nc);
                }
            } else {
                forward.push(pa_pc);
                backward.push(pa_nc);
            }
            prev_speed = speed;
        }

        if visible_start_index >= len - 2 {
            if self.keep_head {
                let c = points[len - 1];
                let mut output =
                    circle_points(c.position, LASER_SIZE, 0.0, std::f32::consts::TAU);
                output.push([c.position[0] + LASER_SIZE, c.position[1]]);
                return output;
            }
            return Vec::new();
        }

        let first = points[visible_start_index];
        let second = points[visible_start_index + 1];
        let penultimate = points[len - 2];
        let ultimate = points[len - 1];
        let dir_fs = normalize(sub(second.position, first.position));
        let dir_pu = normalize(sub(penultimate.position, ultimate.position));
        let ppdir_fs = rotate(dir_fs, -std::f32::consts::FRAC_PI_2);
        let ppdir_pu = rotate(dir_pu, std::f32::consts::FRAC_PI_2);
        let start_cap_size = get_size(first, 0, 0.0);
        let end_cap_size = if self.keep_head {
            LASER_SIZE
        } else {
            get_size(penultimate, len - 2, running_length)
        };

        let mut start_cap = Vec::new();
        if start_cap_size > 0.1 {
            for step in 0..=16 {
                let theta = std::f32::consts::PI * step as f32 / 16.0;
                start_cap.insert(
                    0,
                    add(
                        first.position,
                        rotate(scale(ppdir_fs, start_cap_size), -theta),
                    ),
                );
            }
            start_cap.insert(0, add(first.position, scale(ppdir_fs, -start_cap_size)));
        } else {
            start_cap.push(first.position);
        }

        let mut end_cap = Vec::new();
        for step in 0..=48 {
            let theta = std::f32::consts::PI * 3.0 * step as f32 / 48.0;
            end_cap.push(add(
                ultimate.position,
                rotate(scale(ppdir_pu, -end_cap_size), -theta),
            ));
        }
        end_cap.reverse();
        backward.reverse();

        let mut output = start_cap.clone();
        output.extend(forward);
        output.extend(end_cap);
        output.extend(backward);
        if let Some(first_cap) = start_cap.first().copied() {
            output.push(first_cap);
        }
        output
    }
}

impl LaserTrails {
    pub fn stroke_outlines(
        &self,
        now_ms: f64,
        size_override: Option<f32>,
    ) -> Vec<Vec<[f32; 2]>> {
        self.past()
            .iter()
            .chain(self.current().into_iter())
            .map(|trail| trail.stroke_outline(now_ms, size_override))
            .filter(|outline| !outline.is_empty())
            .collect()
    }
}

fn streamlined_points(points: &[LaserPoint]) -> Vec<WorkPoint> {
    let Some(first) = points.first().copied() else {
        return Vec::new();
    };
    let mut output = vec![WorkPoint::from(first)];
    for point in points.iter().copied().skip(1) {
        let previous = *output.last().unwrap();
        let target = WorkPoint::from(point);
        let t = 1.0 - LASER_STREAMLINE;
        output.push(WorkPoint {
            position: [
                previous.position[0] + (target.position[0] - previous.position[0]) * t,
                previous.position[1] + (target.position[1] - previous.position[1]) * t,
            ],
            timestamp_ms: previous.timestamp_ms
                + (target.timestamp_ms - previous.timestamp_ms) * f64::from(t),
        });
    }
    output
}

fn size_mapping(timestamp_ms: f64, index: usize, total: usize, now_ms: f64) -> f32 {
    let temporal = (1.0 - (now_ms - timestamp_ms).max(0.0) / LASER_DECAY_TIME_MS)
        .clamp(0.0, 1.0) as f32;
    let remaining = total.saturating_sub(index);
    let length = (LASER_DECAY_LENGTH - remaining.min(LASER_DECAY_LENGTH)) as f32
        / LASER_DECAY_LENGTH as f32;
    ease_out(length).min(ease_out(temporal))
}

fn ease_out(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    1.0 - (1.0 - k).powi(4)
}

fn circle_points(center: [f32; 2], radius: f32, start: f32, end: f32) -> Vec<[f32; 2]> {
    let steps = (((end - start).abs() / (std::f32::consts::PI / 16.0)).round() as usize).max(1);
    (0..=steps)
        .map(|step| {
            let theta = start + (end - start) * step as f32 / steps as f32;
            [
                center[0] + theta.cos() * radius,
                center[1] + theta.sin() * radius,
            ]
        })
        .collect()
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn scale(a: [f32; 2], scalar: f32) -> [f32; 2] {
    [a[0] * scalar, a[1] * scalar]
}

fn magnitude(a: [f32; 2]) -> f32 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}

fn normalize(a: [f32; 2]) -> [f32; 2] {
    let magnitude = magnitude(a);
    if magnitude == 0.0 {
        [f32::NAN, f32::NAN]
    } else {
        [a[0] / magnitude, a[1] / magnitude]
    }
}

fn rotate(a: [f32; 2], radians: f32) -> [f32; 2] {
    [
        radians.cos() * a[0] - radians.sin() * a[1],
        radians.sin() * a[0] + radians.cos() * a[1],
    ]
}

fn angle(p: [f32; 2], p1: [f32; 2], p2: [f32; 2]) -> f32 {
    (p2[1] - p[1]).atan2(p2[0] - p[0]) - (p1[1] - p[1]).atan2(p1[0] - p[0])
}

fn norm_angle(angle: f32) -> f32 {
    angle.sin().atan2(angle.cos())
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}
