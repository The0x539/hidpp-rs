use bilge::prelude::u4;

use super::{Feature, FeatureId, root::FeatureType};
use crate::{Result, encode::Decode};

feature!(FeatureSet);

#[derive(Debug, Copy, Clone)]
pub struct FeatureDescriptor {
    pub id: FeatureId,
    pub tags: FeatureType,
    pub version: u8,
}

derive_decode!(FeatureDescriptor, id, tags, version);

impl FeatureSet {
    pub fn get_count(&self) -> Result<u8> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;
        Ok(response[4])
    }

    pub fn get_feature_id(&self, feature_index: u8) -> Result<FeatureDescriptor> {
        let response = self.0.request(Self::ID, u4::new(1), &feature_index)?;
        let descriptor = FeatureDescriptor::decode(&mut &response[4..]);
        self.0.features.insert(feature_index, descriptor.id.into());
        Ok(descriptor)
    }
}
