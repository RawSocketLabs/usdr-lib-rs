use shared::DmrMetadata;
use bincode::{Decode, Encode};
use sdr::Peaks;

#[derive(Decode, Encode)]
pub enum Internal {
    DeviceFreqUpdated,
    BlockMetadata((Vec<DmrMetadata>, Peaks)),
}
