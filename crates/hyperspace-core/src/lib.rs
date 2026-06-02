//! Core types shared across OS/3 Hyperspace components.

mod dimension;
mod error;
mod object;
mod viewport;

pub use dimension::{Dimension, DimensionId};
pub use error::{HyperspaceError, Result};
pub use object::{ObjectKind, SmartObject, SmartObjectId};
pub use viewport::{Viewport, WorldPoint, WorldRect, WorldSize};

use serde::{Deserialize, Serialize};

/// Root workspace state: multiple dimensions, each with its own canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperspaceState {
    pub dimensions: Vec<Dimension>,
    pub active_dimension: DimensionId,
}

impl HyperspaceState {
    pub fn with_demo_content() -> Self {
        let home = Dimension::demo("Home", (0.0, 0.0));
        let work = Dimension::demo("Work", (1200.0, -400.0));
        let active_dimension = home.id;
        Self {
            dimensions: vec![home, work],
            active_dimension,
        }
    }

    pub fn active_dimension(&self) -> Option<&Dimension> {
        self.dimensions.iter().find(|d| d.id == self.active_dimension)
    }

    pub fn active_dimension_mut(&mut self) -> Option<&mut Dimension> {
        self.dimensions
            .iter_mut()
            .find(|d| d.id == self.active_dimension)
    }

    pub fn dimension_by_id(&self, id: DimensionId) -> Option<&Dimension> {
        self.dimensions.iter().find(|d| d.id == id)
    }

    pub fn dimension_by_id_mut(&mut self, id: DimensionId) -> Option<&mut Dimension> {
        self.dimensions.iter_mut().find(|d| d.id == id)
    }

    pub fn add_dimension(&mut self, name: impl Into<String>) -> DimensionId {
        let dimension = Dimension::new(name);
        let id = dimension.id;
        self.dimensions.push(dimension);
        self.active_dimension = id;
        id
    }

    pub fn remove_object(&mut self, dimension_id: DimensionId, object_id: SmartObjectId) -> bool {
        if let Some(dimension) = self.dimension_by_id_mut(dimension_id) {
            let before = dimension.objects.len();
            dimension.objects.retain(|object| object.id != object_id);
            return dimension.objects.len() < before;
        }
        false
    }

    pub fn find_object_mut(
        &mut self,
        dimension_id: DimensionId,
        object_id: SmartObjectId,
    ) -> Option<&mut SmartObject> {
        self.dimension_by_id_mut(dimension_id)?
            .objects
            .iter_mut()
            .find(|object| object.id == object_id)
    }
}
