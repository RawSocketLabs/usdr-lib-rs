use bincode::{Decode, Encode};
use sdr::{FreqBlock, FreqSample};

pub enum Output {
    Metadata,
    Peaks(Vec<FreqSample>),
    Display(DisplayInfo),
    Connection(ConnectionType)
}

#[derive(Encode, Decode)]
pub enum External {
    Disconnect,
    Connection(ConnectionType),
    Display(DisplayInfo),
    Realtime(FreqBlock),
    Peaks(Vec<FreqSample>),
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionType {
    Display,
    Metadata,
}

#[derive(Encode, Decode)]
pub struct DisplayInfo {
    pub center_freq: usize,
    pub rate: usize,
}

pub enum Input {
    DeviceFreqUpdated,
    ClientAtLeastOneConnected,
    ClientNoneConnected,
}
