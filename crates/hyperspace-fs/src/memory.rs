use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use hyperspace_core::{Dimension, DimensionId, HyperspaceError, HyperspaceState, Result, SmartObjectId};

use crate::store::{ObjectRecord, ObjectStore};

#[derive(Debug, Default, Clone)]
pub struct InMemoryObjectStore {
    inner: Arc<RwLock<HashMap<DimensionId, HashMap<SmartObjectId, ObjectRecord>>>>,
}

impl InMemoryObjectStore {
    pub fn from_state(state: &HyperspaceState) -> Self {
        let store = Self::default();
        for dimension in &state.dimensions {
            let _ = store.sync_dimension(dimension);
        }
        store
    }
}

impl ObjectStore for InMemoryObjectStore {
    fn sync_dimension(&self, dimension: &Dimension) -> Result<usize> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| HyperspaceError::Filesystem("store lock poisoned".into()))?;

        let bucket = guard.entry(dimension.id).or_default();
        bucket.clear();

        for object in &dimension.objects {
            bucket.insert(
                object.id,
                ObjectRecord {
                    dimension_id: dimension.id,
                    object: object.clone(),
                },
            );
        }

        Ok(dimension.objects.len())
    }

    fn list_active(&self, dimension_id: DimensionId) -> Result<Vec<ObjectRecord>> {
        let guard = self
            .inner
            .read()
            .map_err(|_| HyperspaceError::Filesystem("store lock poisoned".into()))?;

        Ok(guard
            .get(&dimension_id)
            .map(|bucket| bucket.values().cloned().collect())
            .unwrap_or_default())
    }

    fn write_snapshot(&self, dimension_id: DimensionId) -> Result<String> {
        let entries = self.list_active(dimension_id)?;
        serde_json::to_string_pretty(&entries)
            .map_err(|err| HyperspaceError::Filesystem(err.to_string()))
    }
}
