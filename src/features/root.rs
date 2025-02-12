use bilge::{arbitrary_int::u4, prelude::*};

use super::{Feature, FeatureId};
use crate::Result;

feature!(Root);

#[bitsize(8)]
#[derive(FromBits, DefaultBits, DebugBits, Copy, Clone)]
pub struct FeatureType {
    unused: u3,
    pub compliance_deactivatable: bool,
    pub manufacturing_deactivatable: bool,
    pub engineering: bool,
    pub hidden: bool,
    pub obsolete: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct FeatureDescriptor {
    pub index: u8,
    pub tags: FeatureType,
    pub version: u8,
}

impl Root {
    pub fn get_feature(&self, feature_id: FeatureId) -> Result<Option<FeatureDescriptor>> {
        let response = self.0.request(Self::ID.into(), u4::new(0u8), &feature_id)?;
        let feature_index = response[4];
        if feature_index == 0 {
            return Ok(None);
        }

        self.0.features.insert(feature_index, feature_id.into());

        let descriptor = FeatureDescriptor {
            index: feature_index,
            tags: response[5].into(),
            version: response[6],
        };

        Ok(Some(descriptor))
    }
}
