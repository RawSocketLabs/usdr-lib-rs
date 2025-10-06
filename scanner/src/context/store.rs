// STD LIB
use std::collections::{BTreeMap};

// THIRD PARTY
use chrono::{DateTime, Utc};
use shared::DmrMetadata;
// VENDOR CRATES
use sdr::FreqSample;

#[derive(Default)]
pub struct StoredInfo {
    pub(crate) metadata: BTreeMap<u32, DmrMetadata>,
}

impl StoredInfo {
    pub(crate) fn update_metadata(&mut self, new_metadata: Vec<DmrMetadata>) {
        for metadata in new_metadata {
            if let Some(&freq) = self.metadata.iter()
                .find_map(|(freq, _)|
                    if metadata.within_band(*freq) { Some(freq) } else { None }) {
                println!("Found metadata for freq: {}", metadata.freq);
                let existing = self.metadata.get_mut(&freq).unwrap();
                existing.messages.extend(metadata.messages);
                existing.slot_data_types.extend(metadata.slot_data_types);
                existing.color_codes.extend(metadata.color_codes);
                existing.syncs.extend(metadata.syncs);
                existing.observation_time = metadata.observation_time;
                println!("\t{:?}", self.metadata.get(&freq).unwrap());
            } else {
                println!("No metadata for freq: {}", metadata.freq);
                self.metadata.insert(metadata.freq, metadata);
            }
        }
    }
}

#[derive(Clone)]
pub struct Observation {
    pub timestamp: DateTime<Utc>,
    pub sample: FreqSample,
}
