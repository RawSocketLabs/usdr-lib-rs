use std::os::unix::net::UnixListener;
use tokio::sync::mpsc::Sender;
use shared::ConnectionType;
use crate::io::{Client, Internal};

pub async fn start(client_tx: Sender<Client>, internal_tx: Sender<Internal>) {
    std::thread::spawn( move || {
        std::fs::remove_file("/tmp/sdrscanner").unwrap_or(());
        let listener = UnixListener::bind("/tmp/sdrscanner").unwrap();

        for stream in listener.incoming().flatten() {
            // TODO: Handle errors
            let in_tx_clone = internal_tx.clone();
            client_tx.blocking_send(Client::new(ConnectionType::Display, stream, in_tx_clone)).unwrap();
        }
    });
}