use shared::DmrMetadata;

pub enum Internal {
    DeviceFreqUpdated,
    BlockMetadata(Vec<DmrMetadata>),
}
