use tokio::sync::broadcast::Sender;
use crate::io::Output;

pub struct ProcessContext {
    pub(crate) center_freq: u32,
    pub(crate) sample_rate: u32,
    pub(crate) process_type: ProcessType,
    pub(crate) out_tx: Sender<Output>,
}

impl ProcessContext {
    pub fn new(
        center_freq: u32,
        sample_rate: u32,
        process_type: ProcessType,
        out_tx: Sender<Output>,
    ) -> Self {
        Self {
            center_freq,
            sample_rate,
            process_type,
            out_tx,
        }
    }
}

pub enum ProcessType {
    RawIQ,
    PreProcess,
    Metadata,
}