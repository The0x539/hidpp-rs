use bilge::prelude::*;

macro_rules! feature {
    ($name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name(pub(crate) crate::HidppDevice);

        impl super::Feature for $name {
            const ID: super::FeatureId = super::FeatureId::$name;

            fn open(device: &crate::HidppDevice) -> crate::Result<Option<Self>> {
                let Some(descriptor) = device.root().get_feature(Self::ID)? else {
                    return Ok(None);
                };

                let _ = descriptor.index;
                Ok(Some(Self(device.clone())))
            }
        }
    };
}

pub mod root;
pub use root::Root;

pub mod feature_set;
pub use feature_set::FeatureSet;

pub mod device_information;
pub use device_information::DeviceInformation;

pub mod device_type_and_name;
pub use device_type_and_name::DeviceTypeAndName;

use crate::{HidppDevice, Result};

pub trait Feature: Sized {
    const ID: FeatureId;
    fn open(device: &HidppDevice) -> Result<Option<Self>>;
}

#[repr(u16)]
#[bitsize(16)]
#[derive(FromBits, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeatureId {
    Root = 0x0000,
    FeatureSet = 0x0001,
    DeviceInformation = 0x0003,
    DeviceTypeAndName = 0x0005,

    #[fallback]
    Other(u16),
}

encode_to_primitive!(FeatureId as u16);
decode_from_primitive!(FeatureId as u16);
