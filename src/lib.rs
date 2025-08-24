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

#[derive(Encode, Decode)]
pub enum ConnectionType {
    Display,
    Metadata,
}

#[derive(Encode, Decode)]
pub struct DisplayInfo {
    pub center_freq: usize,
    pub rate: usize,
}
