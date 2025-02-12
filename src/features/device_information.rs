use std::ops::{AddAssign, BitAnd, MulAssign, ShrAssign};

use bilge::prelude::*;
use tinystr::TinyAsciiStr;
use tinyvec::{ArrayVec, array_vec};

use super::Feature;
use crate::Result;

feature!(DeviceInformation);

#[derive(Debug, Copy, Clone)]
pub struct DeviceInfo {
    pub entity_cnt: u8,
    pub unit_id: [u8; 4],
    pub transports: Transports,
    pub model_ids: ArrayVec<[u16; 3]>,
    pub extended_model_id: u8,
    pub capabilities: Capabilities,
}

#[bitsize(16)]
#[derive(FromBits, DebugBits, Copy, Clone)]
pub struct Transports {
    pub bluetooth: bool,
    pub ble: bool,
    pub e_quad: bool,
    pub usb: bool,
    reserved: u12,
}

#[bitsize(8)]
#[derive(FromBits, DebugBits, Copy, Clone)]
pub struct Capabilities {
    pub serial_number: bool,
    reserved: u7,
}

#[repr(u8)]
#[bitsize(8)]
#[derive(FromBits, Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
    MainApplication = 0,
    Bootloader = 1,
    Hardware = 2,
    Touchpad = 3,
    OpticalSensor = 4,
    SoftDevice = 5,
    RfCompanionMcu = 6,
    FactoryApplication = 7,
    RgbCustomEffect = 8,
    MotorDrive = 9,

    #[fallback]
    Other(u8),
}

#[derive(Debug, Copy, Clone)]
pub struct FirmwareInfo {
    pub entity_type: EntityType,
    pub fw_name: (TinyAsciiStr<3>, u8),
    pub revision: u8,
    pub build: u16,
    pub active: bool,
    pub tr_pid: u16,
    pub extra_ver: [u8; 5],
}

impl DeviceInformation {
    pub fn get_device_info(&self) -> Result<DeviceInfo> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;

        let transports: Transports = u16::from_be_bytes([response[9], response[10]]).into();

        let mut model_ids = array_vec!([u16; 3]);
        for i in 0..3 {
            let start = 11 + 2 * i as usize;
            let model_id = u16::from_be_bytes(response[start..start + 2].try_into().unwrap());
            if i < transports.value.count_ones() {
                model_ids.push(model_id);
            } else {
                assert_eq!(model_id, 0);
            }
        }

        let info = DeviceInfo {
            entity_cnt: response[4],
            unit_id: response[5..9].try_into().unwrap(),
            transports,
            model_ids,
            extended_model_id: response[17],
            capabilities: response[18].into(),
        };
        Ok(info)
    }

    pub fn get_fw_info(&self, entity_idx: u8) -> Result<FirmwareInfo> {
        let response = self.0.request(Self::ID, u4::new(1), &entity_idx)?;

        let entity_type = response[4].into();

        let mut build = u16::from_be_bytes([response[10], response[11]]);
        if entity_type != EntityType::SoftDevice {
            build = bcd(build);
        }

        let info = FirmwareInfo {
            entity_type,
            fw_name: (
                TinyAsciiStr::try_from_raw(response[5..8].try_into().unwrap()).unwrap(),
                bcd(response[8]),
            ),
            revision: response[9],
            build,
            active: response[12] != 0,
            tr_pid: u16::from_be_bytes([response[13], response[14]]),
            extra_ver: response[15..=19].try_into().unwrap(),
        };
        Ok(info)
    }

    pub fn get_device_serial_number(&self) -> Result<TinyAsciiStr<12>> {
        let response = self.0.request(Self::ID, u4::new(2), &())?;
        let payload = response[4..][..12].try_into().unwrap();
        Ok(TinyAsciiStr::try_from_raw(payload).unwrap())
    }
}

fn bcd<T>(mut packed: T) -> T
where
    T: From<u8> + Copy + Default + AddAssign + ShrAssign + BitAnd<Output = T> + MulAssign,
{
    let mut acc = 0.into();

    for i in 0..2 * std::mem::size_of::<T>() {
        let mut digit = packed & 0xF.into();
        packed >>= 4.into();

        for _ in 0..i {
            digit *= 10.into();
        }
        acc += digit;
    }

    acc
}

#[cfg(test)]
#[test]
fn test_bcd() {
    assert_eq!(bcd(0x34_u8), 34);
    assert_eq!(bcd(0x3456_u16), 3456);
}
