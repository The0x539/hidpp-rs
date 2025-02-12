use bilge::prelude::u4;

use super::{Feature, FeatureId, root::FeatureType};
use crate::Result;

feature!(FeatureSet);

#[derive(Debug, Copy, Clone)]
pub struct FeatureDescriptor {
    pub id: FeatureId,
    pub tags: FeatureType,
    pub version: u8,
}

impl FeatureSet {
    pub fn get_count(&self) -> Result<u8> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;
        Ok(response[4])
    }

    pub fn get_feature_id(&self, feature_index: u8) -> Result<FeatureDescriptor> {
        let response = self.0.request(Self::ID, u4::new(1), &feature_index)?;
        let feature_id = u16::from_be_bytes([response[4], response[5]]).into();

        self.0.features.insert(feature_index, feature_id);

        let descriptor = FeatureDescriptor {
            id: feature_id.into(),
            tags: response[6].into(),
            version: response[7],
        };

        Ok(descriptor)
    }
}
