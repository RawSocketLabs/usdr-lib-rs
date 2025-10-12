use shared::DmrMetadata;
use bincode::{Decode, Encode};

#[derive(Decode, Encode)]
pub enum Internal {
    DeviceFreqUpdated,
    BlockMetadata(Vec<DmrMetadata>),
}
