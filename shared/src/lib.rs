// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

pub use sdr::{FreqBlock};
use std::collections::{HashSet, BTreeMap};
use std::time::SystemTime;
use bincode::{Decode, Encode};
use sdr::decode::dmr::burst::{FeatureSetID, SlotDataType};
pub use sdr::sample::{Freq, Peaks};
use sdr::SyncPattern;

#[derive(Encode, Decode, Clone, Debug)]
pub enum External {
    Disconnect,
    Connection(ConnectionType),
    Display(DisplayInfo),
    Realtime(FreqBlock),
    Peaks(Peaks),
    Metadata(BTreeMap<u32, DmrMetadata>),
    Squelch(f32),
}

#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    Display,
    Metadata,
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct DisplayInfo {
    pub center_freq: Freq,
    pub rate: usize,
    pub squelch: f32,
}

impl DisplayInfo {
    pub fn new(center_freq: Freq, rate: u32, squelch: f32) -> Self {
        Self {
            center_freq,
            rate: rate as usize,
            squelch,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct DmrMetadata {
    pub freq: Freq,
    pub rssi: f32,
    pub observation_time: SystemTime,
    pub syncs: HashSet<SyncPattern>,
    pub slot_data_types: HashSet<SlotDataType>,
    pub color_codes: HashSet<u8>,
    pub messages: HashSet<Message>,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Encode, Decode, Ord, PartialOrd)]
pub enum Message {
    GroupVoice(MetadataGroupVoice),
    CSBK(MetadataCSBK)
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Encode, Decode, Ord, PartialOrd)]
pub enum CSBKMessageType {
    BaseStationOutboundActivation,
    UnitToUnitVoiceServiceRequest,
    UnitToUnitVoiceServiceResponse,
    NegativeAcknowledgement,
    Preamble,
    ChannelTiming,
}
#[derive(Hash, PartialEq, Eq, Debug, Clone, Encode, Decode, Ord, PartialOrd)]
pub struct MetadataCSBK {
    pub fid: FeatureSetID,
    // pub mtype: CSBKMessageType,
    // // NOTE: Represents either the target or base station address depending on the type of message.
    // pub target: u32,
    // pub source: u32,
}
#[derive(Hash, PartialEq, Eq, Debug, Clone, Encode, Decode, Ord, PartialOrd)]
pub struct MetadataGroupVoice {
    pub fid: FeatureSetID,
    pub group: u32,
    pub source: u32,
}

impl DmrMetadata {
    pub fn new(freq: Freq, rssi: f32) -> Self {
        Self {
            freq,
            rssi,
            observation_time: SystemTime::now(),
            syncs: HashSet::new(),
            color_codes: HashSet::new(),
            messages: HashSet::new(),
            slot_data_types: HashSet::new(),
        }
    }
}