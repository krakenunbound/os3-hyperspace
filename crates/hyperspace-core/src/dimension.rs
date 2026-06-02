use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::object::{ObjectKind, SmartObject};
use crate::viewport::{Viewport, WorldPoint};

pub type DimensionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub id: DimensionId,
    pub name: String,
    pub viewport: Viewport,
    pub objects: Vec<SmartObject>,
}

impl Dimension {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            viewport: Viewport::default(),
            objects: Vec::new(),
        }
    }

    pub fn demo(name: &str, origin_offset: (f32, f32)) -> Self {
        let mut dimension = Self::new(name);
        dimension.viewport.pan_x = origin_offset.0;
        dimension.viewport.pan_y = origin_offset.1;

        dimension.objects = vec![
            SmartObject::note("Welcome to OS/3 Hyperspace", WorldPoint::new(120.0, 140.0))
                .with_body("Scroll to zoom. Middle-drag or Space+drag to pan.\nDouble-click empty space to create a note.\nDrag corner handles on selection to resize."),
            SmartObject::agent("Local Agent", WorldPoint::new(420.0, 160.0))
                .with_body("AI runtime hooks land here. Local-first, always yours."),
            SmartObject::app("Terminal", WorldPoint::new(760.0, 220.0))
                .with_body("Future home of the Redox-compatible shell."),
            SmartObject::folder("Projects", WorldPoint::new(260.0, 420.0))
                .with_body("Hyperspace FS will mount dimensions as navigable space."),
            SmartObject::new(ObjectKind::Link, "Link to Work", WorldPoint::new(620.0, 380.0))
                .with_body("Click (future) or use inspector to target another dimension. Resize me!"),
        ];

        dimension
    }
}
