// STD LIB
use std::collections::{BTreeMap};

// THIRD PARTY
use shared::DmrMetadata;
// VENDOR CRATES

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
                let existing = self.metadata.get_mut(&freq).unwrap();
                existing.messages.extend(metadata.messages);
                existing.slot_data_types.extend(metadata.slot_data_types);
                existing.color_codes.extend(metadata.color_codes);
                existing.syncs.extend(metadata.syncs);
                existing.observation_time = metadata.observation_time;
            } else {
                self.metadata.insert(metadata.freq, metadata);
            }
        }
    }
}