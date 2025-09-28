use sdr::{FreqSample, SlotDataType};
use comms::{ConnectionType, DisplayInfo};

#[derive(Clone, Debug)]
pub enum Output {
    Metadata((u32, SlotDataType, u32, u32)),
    Peaks(Vec<FreqSample>),
    Display(DisplayInfo),
    Connection(ConnectionType)
}
pub enum Input {
    DeviceFreqUpdated,
    ClientAtLeastOneConnected,
    ClientNoneConnected,
}
