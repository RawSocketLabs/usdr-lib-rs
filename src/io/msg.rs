use sdr::FreqSample;
use comms::{ConnectionType, DisplayInfo};

#[derive(Clone, Debug)]
pub enum Output {
    Metadata,
    Peaks(Vec<FreqSample>),
    Display(DisplayInfo),
    Connection(ConnectionType)
}
pub enum Input {
    DeviceFreqUpdated,
    ClientAtLeastOneConnected,
    ClientNoneConnected,
}
