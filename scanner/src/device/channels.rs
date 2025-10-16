// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use sdr::{FreqBlock, IQBlock};
use std::sync::{Arc, atomic::AtomicUsize};
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
    pub realtime_tx: WatchSender<FreqBlock>,
    pub client_count: Arc<AtomicUsize>,
}

impl DevChannels {
    pub fn new(
        dev_rx: Receiver<DevMsg>,
        main_tx: Sender<Internal>,
        process_tx: Sender<(IQBlock, FreqBlock)>,
        realtime_tx: WatchSender<FreqBlock>,
        client_count: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            dev_rx,
            main_tx,
            process_tx,
            realtime_tx,
            client_count,
        }
    }
}
