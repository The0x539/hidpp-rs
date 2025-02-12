use bilge::{arbitrary_int::u4, prelude::*};

use super::{Feature, FeatureId};
use crate::{Result, encode::Decode};

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

decode_from_primitive!(FeatureType as u8);

#[derive(Debug, Copy, Clone)]
pub struct FeatureDescriptor {
    pub index: u8,
    pub tags: FeatureType,
    pub version: u8,
}

derive_decode!(FeatureDescriptor, index, tags, version);

impl Root {
    pub fn get_feature(&self, feature_id: FeatureId) -> Result<Option<FeatureDescriptor>> {
        let response = self.0.request(Self::ID.into(), u4::new(0u8), &feature_id)?;

        let descriptor = FeatureDescriptor::decode(&mut &response[4..]);
        if descriptor.index == 0 {
            return Ok(None);
        }

        self.0.features.insert(descriptor.index, feature_id.into());
        Ok(Some(descriptor))
    }
}
