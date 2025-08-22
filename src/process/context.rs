use tokio::sync::mpsc::Sender;
use crate::io::Output;

pub struct ProcessContext {
    pub(crate) center_freq: f32,
    pub(crate) sample_rate: f32,
    pub(crate) process_type: ProcessType,
    pub(crate) out_tx: Sender<Output>,
}

impl ProcessContext {
    pub fn new(
        center_freq: f32,
        sample_rate: f32,
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