use crate::{DrawingElement, DrawingScene};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MINIMAL_CROP_SIZE: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCrop {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub natural_width: f32,
    pub natural_height: f32,
}

impl ImageCrop {
    pub fn normalized(self) -> Option<Self> {
        if ![
            self.x,
            self.y,
            self.width,
            self.height,
            self.natural_width,
            self.natural_height,
        ]
        .into_iter()
        .all(f32::is_finite)
            || self.natural_width < MINIMAL_CROP_SIZE
            || self.natural_height < MINIMAL_CROP_SIZE
        {
            return None;
        }
        let x = self.x.clamp(0.0, (self.natural_width - MINIMAL_CROP_SIZE).max(0.0));
        let y = self.y.clamp(0.0, (self.natural_height - MINIMAL_CROP_SIZE).max(0.0));
        let width = self
            .width
            .max(MINIMAL_CROP_SIZE)
            .min((self.natural_width - x).max(MINIMAL_CROP_SIZE));
        let height = self
            .height
            .max(MINIMAL_CROP_SIZE)
            .min((self.natural_height - y).max(MINIMAL_CROP_SIZE));
        Some(Self {
            x,
            y,
            width,
            height,
            natural_width: self.natural_width,
            natural_height: self.natural_height,
        })
    }
}

impl DrawingScene {
    pub fn image_crop(&self, id: &str) -> Option<ImageCrop> {
        let element = self.element_by_id(id)?;
        if element.kind != "image" || element.is_deleted {
            return None;
        }
        let value = element.extra.get("crop")?;
        if value.is_null() {
            return None;
        }
        serde_json::from_value(value.clone()).ok()
    }

    pub fn set_image_crop(&mut self, id: &str, crop: Option<ImageCrop>) -> bool {
        let Some(index) = self.element_index(id) else {
            return false;
        };
        if self.elements[index].kind != "image"
            || self.elements[index].is_deleted
            || self.elements[index].is_locked()
        {
            return false;
        }
        let next = match crop {
            Some(crop) => {
                let Some(crop) = crop.normalized() else {
                    return false;
                };
                serde_json::to_value(crop).expect("ImageCrop serializes")
            }
            None => Value::Null,
        };
        if self.elements[index].extra.get("crop") == Some(&next) {
            return false;
        }
        self.elements[index].extra.insert("crop".to_owned(), next);
        mark_changed(&mut self.elements[index]);
        true
    }

    /// Crop the visible image by moving one or more natural-image edges. The
    /// input deltas are in natural image pixels; this keeps the core independent
    /// from the native UI's zoom and transform-handle coordinate systems.
    pub fn adjust_image_crop(
        &mut self,
        id: &str,
        delta_left: f32,
        delta_top: f32,
        delta_right: f32,
        delta_bottom: f32,
        natural_size: [f32; 2],
    ) -> bool {
        if ![
            delta_left,
            delta_top,
            delta_right,
            delta_bottom,
            natural_size[0],
            natural_size[1],
        ]
        .into_iter()
        .all(f32::is_finite)
        {
            return false;
        }
        let current = self.image_crop(id).unwrap_or(ImageCrop {
            x: 0.0,
            y: 0.0,
            width: natural_size[0],
            height: natural_size[1],
            natural_width: natural_size[0],
            natural_height: natural_size[1],
        });
        let next = ImageCrop {
            x: current.x + delta_left,
            y: current.y + delta_top,
            width: current.width - delta_left + delta_right,
            height: current.height - delta_top + delta_bottom,
            natural_width: current.natural_width,
            natural_height: current.natural_height,
        };
        self.set_image_crop(id, Some(next))
    }
}

pub fn crop_source_rect(element: &DrawingElement) -> Option<ImageCrop> {
    if element.kind != "image" || element.is_deleted {
        return None;
    }
    let crop = element.extra.get("crop")?;
    if crop.is_null() {
        return None;
    }
    serde_json::from_value::<ImageCrop>(crop.clone()).ok()?.normalized()
}

fn mark_changed(element: &mut DrawingElement) {
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
    element.extra.insert("versionNonce".to_owned(), json!(nonce));
}
