use std::ops::{AddAssign, BitAnd, MulAssign, ShrAssign};

use bilge::prelude::*;
use tinystr::TinyAsciiStr;
use tinyvec::ArrayVec;

use super::Feature;
use crate::{Result, encode::Decode};

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

impl Decode for DeviceInfo {
    fn decode(buf: &mut &[u8]) -> Self {
        let (entity_cnt, unit_id) = Decode::decode(buf);

        let transports: Transports = Decode::decode(buf);
        let len = transports.value.count_ones() as usize;
        let model_ids = ArrayVec::from_array_len(Decode::decode(buf), len);

        assert!(model_ids.grab_spare_slice().iter().all(|n| *n == 0));

        Self {
            entity_cnt,
            unit_id,
            transports,
            model_ids,
            extended_model_id: Decode::decode(buf),
            capabilities: Decode::decode(buf),
        }
    }
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

decode_from_primitive!(Transports as u16);

#[bitsize(8)]
#[derive(FromBits, DebugBits, Copy, Clone)]
pub struct Capabilities {
    pub serial_number: bool,
    reserved: u7,
}

decode_from_primitive!(Capabilities as u8);

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

decode_from_primitive!(EntityType as u8);

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

impl Decode for FirmwareInfo {
    fn decode(buf: &mut &[u8]) -> Self {
        let entity_type = Decode::decode(buf);
        Self {
            entity_type,
            fw_name: (Decode::decode(buf), bcd(Decode::decode(buf))),
            revision: Decode::decode(buf),
            build: {
                let mut build = Decode::decode(buf);
                if entity_type != EntityType::SoftDevice {
                    build = bcd(build);
                }
                build
            },
            active: Decode::decode(buf),
            tr_pid: Decode::decode(buf),
            extra_ver: Decode::decode(buf),
        }
    }
}

impl DeviceInformation {
    pub fn get_device_info(&self) -> Result<DeviceInfo> {
        let response = self.0.request(Self::ID, u4::new(0), &())?;
        let info = DeviceInfo::decode(&mut &response[4..]);
        Ok(info)
    }

    pub fn get_fw_info(&self, entity_idx: u8) -> Result<FirmwareInfo> {
        let response = self.0.request(Self::ID, u4::new(1), &entity_idx)?;
        let info = FirmwareInfo::decode(&mut &response[4..]);
        Ok(info)
    }

    pub fn get_device_serial_number(&self) -> Result<TinyAsciiStr<12>> {
        let response = self.0.request(Self::ID, u4::new(2), &())?;
        Ok(TinyAsciiStr::decode(&mut &response[4..16]))
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
