use bilge::prelude::*;
use tinystr::TinyAsciiStr;

use super::Feature;
use crate::{Result, encode::Decode};

feature!(DeviceTypeAndName);

#[bitsize(8)]
#[repr(u8)]
#[derive(FromBits, Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Keyboard,
    RemoteControl,
    Numpad,
    Mouse,
    Trackpad,
    Trackball,
    Presenter,
    Receiver,
    Headset,
    Webcam,
    SteeringWheel,
    Joystick,
    Gamepad,
    Dock,
    Speaker,
    Microphone,
    IlluminationLight,
    ProgrammableController,
    CarSimPedals,
    Adapter,

    #[fallback]
    Other(u8),
}

decode_from_primitive!(DeviceType as u8);

impl DeviceTypeAndName {
    pub fn get_device_name_count(&self) -> Result<u8> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;
        Ok(response[4])
    }

    pub fn get_device_name(&self, char_index: u8) -> Result<TinyAsciiStr<16>> {
        let response = self.0.request(Self::ID, u4::new(1), &char_index)?;
        let chunk = TinyAsciiStr::decode(&mut &response[4..]);
        Ok(chunk)
    }

    pub fn get_device_type(&self) -> Result<DeviceType> {
        let response = self.0.request(Self::ID, u4::new(2), &())?;
        Ok(response[4].into())
    }
}
