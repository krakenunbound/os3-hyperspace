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
            SmartObject::note("Welcome to OS/3 Hyperspace", WorldPoint::new(80.0, 100.0))
                .with_body("Scroll to zoom • Middle/Space-drag to pan • Drag corners to resize\nDouble-click empty to spawn Note • Click Link to jump dimensions\nPremium glass cards with window chrome on selected objects."),
            SmartObject::agent("Local Agent", WorldPoint::new(380.0, 140.0))
                .with_body("AI runtime hooks. Local-first, always yours.\nClick to invoke stub agent.\nGlows to show it's alive."),
            SmartObject::app("Terminal", WorldPoint::new(720.0, 180.0))
                .with_body("main@HYPERION-7X ~ %\nOS/3 Hyperspace 0.3.0 (Nebula)\n> help\n  zoom, pan, spawn, link, ai\n> _"),
            SmartObject::folder("Projects", WorldPoint::new(220.0, 380.0))
                .with_body("alpha-build/\ndesign-system/\nmodules/\n  kernel.os3\n  hyperspace.dll\n  README.md\n(Zoom in for details • Future: real contents)"),
            SmartObject::new(ObjectKind::Link, "Link to Work", WorldPoint::new(580.0, 340.0))
                .with_body("Portal to another dimension.\nSet target in Inspector • Click to navigate.\nRenders as glowing wormhole."),
            SmartObject::app("System / About", WorldPoint::new(920.0, 420.0))
                .with_body("OS/3 HYPERSPACE\nNebula 0.3.0\nHyperion Quantum 8-Core\nLocal-first • Infinite canvas\nBuilt for the AI era."),
        ];

        dimension
    }
}
