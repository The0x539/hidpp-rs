use bilge::prelude::*;
use tinystr::TinyAsciiStr;

use super::Feature;
use crate::{Result, encode::Decode};

feature!(DeviceFriendlyName);

pub struct FriendlyNameLengths {
    pub name_len: u8,
    pub name_max_len: u8,
    pub default_name_len: u8,
}

derive_decode!(
    FriendlyNameLengths,
    name_len,
    name_max_len,
    default_name_len,
);

impl DeviceFriendlyName {
    pub fn get_friendly_name_len(&self) -> Result<FriendlyNameLengths> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;
        Ok(Decode::decode(&mut &response[4..]))
    }

    pub fn get_friendly_name(&self, byte_index: u8) -> Result<TinyAsciiStr<15>> {
        let response = self.0.request(Self::ID, u4::new(1), &byte_index)?;
        Ok(Decode::decode(&mut &response[5..]))
    }

    pub fn get_default_friendly_name(&self, byte_index: u8) -> Result<TinyAsciiStr<15>> {
        let response = self.0.request(Self::ID, u4::new(2), &byte_index)?;
        Ok(Decode::decode(&mut &response[5..]))
    }

    pub fn set_friendly_name(&self, byte_index: u8, name_chunk: &str) -> Result<u8> {
        let params = (byte_index, name_chunk);
        let response = self.0.request(Self::ID, u4::new(3), &params)?;
        Ok(response[4])
    }

    pub fn reset_friendly_name(&self) -> Result<u8> {
        let response = self.0.request(Self::ID, u4::new(4), &())?;
        Ok(response[4])
    }
}
