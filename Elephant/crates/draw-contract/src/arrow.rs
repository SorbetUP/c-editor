use crate::{ArrowEndpoint, DrawingScene};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arrowhead {
    Arrow,
    Bar,
    Circle,
    CircleOutline,
    Triangle,
    TriangleOutline,
    Diamond,
    DiamondOutline,
    CardinalityOne,
    CardinalityMany,
    CardinalityOneOrMany,
    CardinalityExactlyOne,
    CardinalityZeroOrOne,
    CardinalityZeroOrMany,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrowheadPrimitive {
    Line {
        from: [f32; 2],
        to: [f32; 2],
    },
    Polygon {
        points: Vec<[f32; 2]>,
        filled: bool,
    },
    Circle {
        center: [f32; 2],
        radius: f32,
        filled: bool,
    },
}

impl Arrowhead {
    pub const ALL: [Self; 14] = [
        Self::Arrow,
        Self::Bar,
        Self::Circle,
        Self::CircleOutline,
        Self::Triangle,
        Self::TriangleOutline,
        Self::Diamond,
        Self::DiamondOutline,
        Self::CardinalityOne,
        Self::CardinalityMany,
        Self::CardinalityOneOrMany,
        Self::CardinalityExactlyOne,
        Self::CardinalityZeroOrOne,
        Self::CardinalityZeroOrMany,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Arrow => "arrow",
            Self::Bar => "bar",
            Self::Circle => "circle",
            Self::CircleOutline => "circle_outline",
            Self::Triangle => "triangle",
            Self::TriangleOutline => "triangle_outline",
            Self::Diamond => "diamond",
            Self::DiamondOutline => "diamond_outline",
            Self::CardinalityOne => "cardinality_one",
            Self::CardinalityMany => "cardinality_many",
            Self::CardinalityOneOrMany => "cardinality_one_or_many",
            Self::CardinalityExactlyOne => "cardinality_exactly_one",
            Self::CardinalityZeroOrOne => "cardinality_zero_or_one",
            Self::CardinalityZeroOrMany => "cardinality_zero_or_many",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arrowhead| arrowhead.id() == id)
    }

    pub fn primitives(self, previous: [f32; 2], end: [f32; 2]) -> Vec<ArrowheadPrimitive> {
        let Some(axis) = ArrowAxis::new(previous, end) else {
            return Vec::new();
        };
        match self {
            Self::Arrow => axis.crow(12.0, 5.0),
            Self::Bar | Self::CardinalityOne => vec![axis.bar(1.5, 6.0)],
            Self::Circle => vec![axis.circle(5.0, 5.0, true)],
            Self::CircleOutline => vec![axis.circle(5.0, 5.0, false)],
            Self::Triangle => vec![axis.triangle(12.0, 6.0, true)],
            Self::TriangleOutline => vec![axis.triangle(12.0, 6.0, false)],
            Self::Diamond => vec![axis.diamond(14.0, 5.0, true)],
            Self::DiamondOutline => vec![axis.diamond(14.0, 5.0, false)],
            Self::CardinalityMany => axis.crow(10.0, 6.0),
            Self::CardinalityOneOrMany => {
                let mut shapes = axis.crow(10.0, 6.0);
                shapes.push(axis.bar(8.0, 6.0));
                shapes
            }
            Self::CardinalityExactlyOne => vec![axis.bar(1.5, 6.0), axis.bar(8.0, 6.0)],
            Self::CardinalityZeroOrOne => {
                vec![axis.circle(5.5, 4.0, false), axis.bar(13.0, 6.0)]
            }
            Self::CardinalityZeroOrMany => {
                let mut shapes = axis.crow(10.0, 6.0);
                shapes.push(axis.circle(14.0, 4.0, false));
                shapes
            }
        }
    }
}

struct ArrowAxis {
    end: [f32; 2],
    unit: [f32; 2],
    normal: [f32; 2],
}

impl ArrowAxis {
    fn new(previous: [f32; 2], end: [f32; 2]) -> Option<Self> {
        let dx = end[0] - previous[0];
        let dy = end[1] - previous[1];
        let length = (dx * dx + dy * dy).sqrt();
        if !length.is_finite() || length <= f32::EPSILON {
            return None;
        }
        let unit = [dx / length, dy / length];
        Some(Self {
            end,
            unit,
            normal: [-unit[1], unit[0]],
        })
    }

    fn behind(&self, distance: f32) -> [f32; 2] {
        [
            self.end[0] - self.unit[0] * distance,
            self.end[1] - self.unit[1] * distance,
        ]
    }

    fn offset_normal(&self, point: [f32; 2], amount: f32) -> [f32; 2] {
        [
            point[0] + self.normal[0] * amount,
            point[1] + self.normal[1] * amount,
        ]
    }

    fn crow(&self, depth: f32, half_width: f32) -> Vec<ArrowheadPrimitive> {
        let base = self.behind(depth);
        vec![
            ArrowheadPrimitive::Line {
                from: self.end,
                to: self.offset_normal(base, half_width),
            },
            ArrowheadPrimitive::Line {
                from: self.end,
                to: self.offset_normal(base, -half_width),
            },
        ]
    }

    fn bar(&self, distance: f32, half_width: f32) -> ArrowheadPrimitive {
        let center = self.behind(distance);
        ArrowheadPrimitive::Line {
            from: self.offset_normal(center, half_width),
            to: self.offset_normal(center, -half_width),
        }
    }

    fn circle(&self, distance: f32, radius: f32, filled: bool) -> ArrowheadPrimitive {
        ArrowheadPrimitive::Circle {
            center: self.behind(distance),
            radius,
            filled,
        }
    }

    fn triangle(&self, depth: f32, half_width: f32, filled: bool) -> ArrowheadPrimitive {
        let base = self.behind(depth);
        ArrowheadPrimitive::Polygon {
            points: vec![
                self.end,
                self.offset_normal(base, half_width),
                self.offset_normal(base, -half_width),
            ],
            filled,
        }
    }

    fn diamond(&self, depth: f32, half_width: f32, filled: bool) -> ArrowheadPrimitive {
        let middle = self.behind(depth / 2.0);
        ArrowheadPrimitive::Polygon {
            points: vec![
                self.end,
                self.offset_normal(middle, half_width),
                self.behind(depth),
                self.offset_normal(middle, -half_width),
            ],
            filled,
        }
    }
}

impl DrawingScene {
    pub fn set_arrowhead(
        &mut self,
        arrow_id: &str,
        endpoint: ArrowEndpoint,
        arrowhead: Option<Arrowhead>,
    ) -> bool {
        let Some(index) = self.elements.iter().position(|element| {
            element.id == arrow_id
                && element.kind == "arrow"
                && !element.is_deleted
                && !is_locked(element)
        }) else {
            return false;
        };
        let next = arrowhead.map(|value| value.id().to_owned());
        let element = &mut self.elements[index];
        let target = match endpoint {
            ArrowEndpoint::Start => &mut element.start_arrowhead,
            ArrowEndpoint::End => &mut element.end_arrowhead,
        };
        if *target == next {
            return false;
        }
        *target = next;
        mark_changed(element);
        true
    }

    pub fn set_elbowed_arrow(&mut self, arrow_id: &str, elbowed: bool) -> bool {
        let Some(index) = self.elements.iter().position(|element| {
            element.id == arrow_id
                && element.kind == "arrow"
                && !element.is_deleted
                && !is_locked(element)
        }) else {
            return false;
        };
        let element = &mut self.elements[index];
        let current = element
            .extra
            .get("elbowed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if current == elbowed {
            return false;
        }
        element.extra.insert("elbowed".to_owned(), json!(elbowed));
        if elbowed {
            element
                .extra
                .entry("fixedSegments".to_owned())
                .or_insert(Value::Null);
            element
                .extra
                .entry("startIsSpecial".to_owned())
                .or_insert(Value::Null);
            element
                .extra
                .entry("endIsSpecial".to_owned())
                .or_insert(Value::Null);
        } else {
            element.extra.remove("fixedSegments");
            element.extra.remove("startIsSpecial");
            element.extra.remove("endIsSpecial");
        }
        mark_changed(element);
        true
    }
}

fn is_locked(element: &crate::DrawingElement) -> bool {
    element
        .extra
        .get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn mark_changed(element: &mut crate::DrawingElement) {
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);
    let nonce = element
        .extra
        .get("versionNonce")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    element.extra.insert("version".to_owned(), json!(version));
    element
        .extra
        .insert("versionNonce".to_owned(), json!(nonce));
}
