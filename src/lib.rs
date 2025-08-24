use sdr::{FreqSample, FreqBlock};
use bincode::{Decode, Encode};


#[derive(Encode, Decode)]
pub enum External {
    Disconnect,
    Connection(ConnectionType),
    Display(DisplayInfo),
    Realtime(FreqBlock),
    Peaks(Vec<FreqSample>),
}

#[derive(Clone, Encode, Decode, Debug)]
pub enum ConnectionType {
    Display,
    Metadata,
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct DisplayInfo {
    pub center_freq: usize,
    pub rate: usize,
}
