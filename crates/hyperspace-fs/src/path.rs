use std::fmt;

use hyperspace_core::DimensionId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HyperspacePath {
    pub dimension_id: DimensionId,
    pub segments: Vec<String>,
}

impl HyperspacePath {
    pub fn root(dimension_id: DimensionId) -> Self {
        Self {
            dimension_id,
            segments: vec!["/".into()],
        }
    }

    pub fn push(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }
}

impl fmt::Display for HyperspacePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hs://{}/", self.dimension_id)?;
        for segment in &self.segments {
            if segment != "/" {
                write!(f, "{segment}/")?;
            }
        }
        Ok(())
    }
}
