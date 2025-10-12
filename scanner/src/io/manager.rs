use tokio::sync::{mpsc::Sender, broadcast, watch};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, atomic::AtomicUsize};
use shared::ConnectionType;
use crate::io::{Internal, handle_client};

pub struct IOManager {
    client_count: Arc<AtomicUsize>,
}

impl IOManager {
    pub fn new() -> Self {
        Self { 
            client_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn client_count(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.client_count)
    }

    pub async fn start(
        self, 
        external_tx: broadcast::Sender<shared::External>,
        realtime_rx: watch::Receiver<shared::FreqBlock>,
        internal_tx: Sender<Internal>,
        initial_display_info: shared::External,
    ) {
        tokio::spawn(self.start_listener(external_tx, realtime_rx, internal_tx, initial_display_info));
    }

    async fn start_listener(
        self, 
        external_tx: broadcast::Sender<shared::External>,
        realtime_rx: watch::Receiver<shared::FreqBlock>,
        internal_tx: Sender<Internal>,
        initial_display_info: shared::External,
    ) {
        let _ = std::fs::remove_file("/tmp/sdrscanner");
        let listener = UnixListener::bind("/tmp/sdrscanner")
            .expect("Failed to bind Unix socket");
        
        let listener = tokio::net::UnixListener::from_std(listener)
            .expect("Failed to create async listener");

        let mut next_client_id = 1u64;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let client_id = next_client_id;
                    next_client_id += 1;

                    // For now, assume all clients are Display clients
                    let connection_type = ConnectionType::Display;

                    // Increment client count
                    self.client_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Clone channels for this client
                    let internal_tx = internal_tx.clone();
                    let external_tx = external_tx.clone();
                    let external_rx = external_tx.subscribe();
                    let realtime_rx = realtime_rx.clone();
                    let initial_display_info = initial_display_info.clone();
                    
                    // Spawn client handler directly
                    tokio::spawn(async move {
                        // Send initial display info to new client via broadcast
                        let _ = external_tx.send(initial_display_info);
                        handle_client(stream, client_id, connection_type, external_rx, realtime_rx, internal_tx).await;
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
