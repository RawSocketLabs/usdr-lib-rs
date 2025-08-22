use sdr::{FreqBlock, IQBlock};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Sender as WatchSender,
};
use crate::device::DevMsg;
use crate::io::Input;

pub struct DevChannels {
    pub dev_rx: Receiver<DevMsg>,
    pub main_tx: Sender<Input>,
    pub process_tx: Sender<(IQBlock, FreqBlock)>,
    pub realtime_tx: WatchSender<FreqBlock>,
}

impl DevChannels {
    pub fn new(
        dev_rx: Receiver<DevMsg>,
        main_tx: Sender<Input>,
        process_tx: Sender<(IQBlock, FreqBlock)>,
        realtime_tx: WatchSender<FreqBlock>,
    ) -> Self {
        Self {
            dev_rx,
            main_tx,
            process_tx,
            realtime_tx,
        }
    }
}
