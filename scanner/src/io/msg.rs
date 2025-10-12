use std::collections::{BTreeMap};
use sdr::{Peaks};
use shared::{ConnectionType, DisplayInfo, DmrMetadata};

#[derive(Clone, Debug)]
pub enum Output {
    Metadata(BTreeMap<u32, DmrMetadata>),
    Peaks(Peaks),
    Display(DisplayInfo),
    Connection(ConnectionType)
}
pub enum Input {
    DeviceFreqUpdated,
    ClientAtLeastOneConnected,
    ClientNoneConnected,
}
