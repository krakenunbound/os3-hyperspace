use hyperspace_core::{Dimension, DimensionId, Result, SmartObject};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectRecord {
    pub dimension_id: DimensionId,
    pub object: SmartObject,
}

pub trait ObjectStore {
    fn sync_dimension(&self, dimension: &Dimension) -> Result<usize>;
    fn list_active(&self, dimension_id: DimensionId) -> Result<Vec<ObjectRecord>>;
    fn write_snapshot(&self, dimension_id: DimensionId) -> Result<String>;
}
