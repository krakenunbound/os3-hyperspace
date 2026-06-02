use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::viewport::{WorldPoint, WorldSize};

pub type SmartObjectId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectKind {
    Note,
    App,
    Folder,
    Agent,
    Link,
}

impl ObjectKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::App => "App",
            Self::Folder => "Folder",
            Self::Agent => "Agent",
            Self::Link => "Link",
        }
    }

    pub fn accent(self) -> [u8; 3] {
        match self {
            Self::Note => [255, 196, 86],
            Self::App => [96, 165, 250],
            Self::Folder => [74, 222, 128],
            Self::Agent => [192, 132, 252],
            Self::Link => [248, 113, 113],
        }
    }

    /// Returns a unicode/emoji symbol for premium per-kind iconography in the UI.
    /// Used in draw_object headers to make Smart Objects look like distinct modern
    /// "apps" or "portals" (inspired by the high-end mockup's icon-rich cards and sidebars).
    /// Keeps us lightweight (no external icon fonts required in egui prototype).
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Note => "📝",   // or "✎" for more minimal
            Self::App => "🚀",
            Self::Folder => "📁",
            Self::Agent => "🧠",
            Self::Link => "🌀",   // or "🔗" – portal/wormhole vibe fits "best OS" multidimensional feel
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartObject {
    pub id: SmartObjectId,
    pub kind: ObjectKind,
    pub title: String,
    pub body: String,
    pub position: WorldPoint,
    pub size: WorldSize,

    /// For Link kind: the target DimensionId (stored as Uuid via the SmartObjectId alias).
    ///
    /// Set via Inspector in the shell prototype. Used by app.rs for click-to-navigate.
    /// Optional + skipped in serde when None for persistence compatibility with older saves.
    /// See docs/smart-objects.md and docs/DEVELOPMENT-LOG.md (2026-06-03) for design notes.
    /// Future: may become a richer LinkTarget enum (cross-object, URL, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_target: Option<SmartObjectId>,
}

impl SmartObject {
    pub fn new(kind: ObjectKind, title: impl Into<String>, position: WorldPoint) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            title: title.into(),
            body: String::new(),
            position,
            size: default_size(kind),
            link_target: None,
        }
    }

    pub fn note(title: impl Into<String>, position: WorldPoint) -> Self {
        Self::new(ObjectKind::Note, title, position)
    }

    pub fn app(title: impl Into<String>, position: WorldPoint) -> Self {
        Self::new(ObjectKind::App, title, position)
    }

    pub fn folder(title: impl Into<String>, position: WorldPoint) -> Self {
        Self::new(ObjectKind::Folder, title, position)
    }

    pub fn agent(title: impl Into<String>, position: WorldPoint) -> Self {
        Self::new(ObjectKind::Agent, title, position)
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn contains(&self, point: WorldPoint) -> bool {
        point.x >= self.position.x
            && point.y >= self.position.y
            && point.x <= self.position.x + self.size.width
            && point.y <= self.position.y + self.size.height
    }
}

fn default_size(kind: ObjectKind) -> WorldSize {
    match kind {
        ObjectKind::Note => WorldSize {
            width: 280.0,
            height: 180.0,
        },
        ObjectKind::App => WorldSize {
            width: 240.0,
            height: 160.0,
        },
        ObjectKind::Folder => WorldSize {
            width: 220.0,
            height: 140.0,
        },
        ObjectKind::Agent => WorldSize {
            width: 260.0,
            height: 190.0,
        },
        ObjectKind::Link => WorldSize {
            width: 200.0,
            height: 120.0,
        },
    }
}
