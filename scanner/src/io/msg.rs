// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use bincode::{Decode, Encode};
use sdr::sample::Peaks;
use shared::DmrMetadata;

#[derive(Decode, Encode)]
pub enum Internal {
    DeviceFreqUpdated,
    BlockMetadata((Vec<DmrMetadata>, Peaks)),
    Squelch(f32),
}
