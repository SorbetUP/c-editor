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
