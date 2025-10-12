use sdr::{FreqBlock, IQBlock};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Sender as WatchSender,
};
use crate::device::DevMsg;
use crate::io::Internal;

pub struct DevChannels {
    pub dev_rx: Receiver<DevMsg>,
    pub main_tx: Sender<Internal>,
    pub process_tx: Sender<(IQBlock, FreqBlock)>,
}

impl DevChannels {
    pub fn new(
        dev_rx: Receiver<DevMsg>,
        main_tx: Sender<Internal>,
        process_tx: Sender<(IQBlock, FreqBlock)>,
    ) -> Self {
        Self {
            dev_rx,
            main_tx,
            process_tx,
        }
    }
}
