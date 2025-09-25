use std::os::unix::net::{UnixListener, UnixStream};
use bincode::config::{BigEndian, Configuration, Fixint};
use sdr::FreqBlock;
use comms::{ConnectionType, External};
use tokio::sync::{mpsc::Sender, watch::Receiver as WatchReceiver, broadcast::Sender as BroadcastSender, broadcast::Receiver as BroadcastReceiver};
use crate::io::{Input, Output};

pub async fn start(in_tx: Sender<Input>, out_tx: BroadcastSender<Output>, realtime_rx: WatchReceiver<FreqBlock>) {
    tokio::task::spawn(async move {
        // TODO: Make this better...
        std::fs::remove_file("/tmp/sdrscanner").unwrap_or(());
        let listener = UnixListener::bind("/tmp/sdrscanner").unwrap();

        for stream in listener.incoming().flatten() {
            let (in_tx, out_rx, realtime_rx) = (in_tx.clone(), out_tx.subscribe(), realtime_rx.clone());
            let _ = tokio::task::spawn(async move { handle_client(stream, in_tx, out_rx, realtime_rx).await}).await;
        }
    });
}

async fn handle_client(mut stream: UnixStream, in_tx: Sender<Input>, out_rx: BroadcastReceiver<Output>, realtime_rx: WatchReceiver<FreqBlock>) {
    let config = bincode::config::standard().with_big_endian().with_fixed_int_encoding();

    if let External::Connection(conn_type) = bincode::decode_from_std_read(&mut stream, config).unwrap() {
        // TODO: Handle errors
        in_tx.send(Input::ClientAtLeastOneConnected).await.unwrap();

        match conn_type {
            ConnectionType::Display => {handle_display_client(stream, config, in_tx, out_rx, realtime_rx).await;},
            ConnectionType::Metadata => {}
        }
    }
}

async fn handle_display_client(mut stream: UnixStream, config: Configuration<BigEndian, Fixint>, in_tx: Sender<Input>, mut out_rx: BroadcastReceiver<Output>, mut realtime_rx: WatchReceiver<FreqBlock>) {
    loop {
        tokio::select! {
            biased;

            msg = out_rx.recv() => {
                match msg {
                    Ok(Output::Display(display_info)) => {bincode::encode_into_std_write(External::Display(display_info), &mut stream, config).unwrap();},
                    Ok(Output::Peaks(peaks)) => {bincode::encode_into_std_write(External::Peaks(peaks), &mut stream, config).unwrap();},
                    _ => {},
                }
            },
            _ = realtime_rx.changed() => {
                let freq_block = realtime_rx.borrow_and_update().clone();
                bincode::encode_into_std_write(External::Realtime(freq_block), &mut stream, config).unwrap();
            },

        }
    }

}
