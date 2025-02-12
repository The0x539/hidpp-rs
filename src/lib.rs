use std::sync::Arc;

use bilge::prelude::*;
use crossbeam::channel::{self as mpmc, TryRecvError};
use deadpool::unmanaged::Pool;
use features::FeatureId;
use hidapi::HidDevice;
use pollster::FutureExt;
use to_params::ToParams;

mod feature_lookup;
pub mod features;
pub mod to_params;

use feature_lookup::FeatureLookup;
use tinyvec::{ArrayVec, array_vec};

#[derive(Debug, Clone)]
pub struct HidppDevice {
    device_index: u8,
    features: Arc<FeatureLookup>,
    to_device_tx: mpmc::Sender<Packet>,
    from_device_rx: mpmc::Receiver<Packet>,
    // TODO: write my own basic pool for this very basic usage
    swid_pool: Pool<u8>,
}

type Packet = ArrayVec<[u8; 20]>;

#[derive(Debug, Copy, Clone)]
pub enum HidppError {
    Protocol(ProtocolError),
}

#[repr(u8)]
#[bitsize(8)]
#[derive(FromBits, Debug, Copy, Clone)]
pub enum ProtocolError {
    NoError = 0,
    Unknown = 1,
    InvalidArgument = 2,
    OutOfRange = 3,
    HwError = 4,
    LogitechInternal = 5,
    InvalidFeatureIndex = 6,
    InvalidFunctionId = 7,
    Busy = 8,
    Unsupported = 9,
    #[fallback]
    Other(u8),
}

pub type Result<T, E = HidppError> = core::result::Result<T, E>;

impl HidppDevice {
    pub fn new(hid: HidDevice, device_index: u8) -> Self {
        let (to_device_tx, to_device_rx) = mpmc::unbounded();
        let (from_device_tx, from_device_rx) = mpmc::unbounded();

        let device = HidppDevice {
            device_index,
            features: Arc::new(FeatureLookup::new()),
            to_device_tx,
            from_device_rx,
            swid_pool: Pool::from(1..=15),
        };

        std::thread::spawn(|| receive_packets(hid, to_device_rx, from_device_tx));

        device
    }

    pub fn request(
        &self,
        feature_id: FeatureId,
        function_index: u4,
        params: &dyn ToParams,
    ) -> Result<Packet> {
        let feature_index = self.features.get_index(feature_id.into()).unwrap();

        let swid = self.swid_pool.get().block_on().unwrap();
        let byte_3 = function_index.value() << 4 | *swid;

        let mut request = array_vec![0x11, self.device_index, feature_index, byte_3];
        params.encode(&mut request);

        let rx = self.from_device_rx.clone();
        self.to_device_tx.send(request).unwrap();
        for packet in rx.iter() {
            if packet[..4] == request[..4] {
                return Ok(packet);
            } else if packet[2] == 0xFF {
                // feature indices stop at 254, so this indicates an error

                if (&packet[..2], &packet[3..5]) == (&request[..2], &request[2..4]) {
                    let protocol_error = ProtocolError::from(packet[5]);
                    return Err(HidppError::Protocol(protocol_error));
                }
            }
        }
        panic!("reached end of receiver");
    }

    pub fn root(&self) -> features::Root {
        features::Root(self.clone())
    }

    pub fn feature_set(&self) -> Result<features::FeatureSet> {
        self.root().get_feature(FeatureId::FeatureSet)?;
        Ok(features::FeatureSet(self.clone()))
    }
}

fn receive_packets(
    hid: HidDevice,
    to_device_rx: mpmc::Receiver<Packet>,
    from_device_tx: mpmc::Sender<Packet>,
) -> Result<(), hidapi::HidError> {
    // TODO: figure out exactly what to actually do with HID errors.
    // What if callers want to know that the device got disconnected?

    let mut buf = array_vec![0; 20];

    loop {
        match to_device_rx.try_recv() {
            Ok(msg) => {
                hid.write(&msg)?;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break,
        }

        buf.clear();
        buf.resize(buf.capacity(), 0);

        let len = match hid.read_timeout(&mut buf, 50) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        if len > 0 {
            buf.truncate(len);
            if let Err(e) = from_device_tx.send(buf) {
                eprintln!("{e}");
                break;
            }
        }
    }

    Ok(())
}
