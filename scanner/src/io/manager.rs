// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use tokio::sync::{mpsc::Sender, broadcast, watch};
use tokio::net::UnixListener;
use std::sync::{Arc, atomic::AtomicUsize};
use shared::ConnectionType;
use crate::io::{Internal, handle_client};
use tracing::{info, error, debug};

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
        info!("IO manager start_listener called");
        let _ = std::fs::remove_file("/tmp/sdrscanner");
        debug!("Removed old socket file");
        let listener = UnixListener::bind("/tmp/sdrscanner")
            .expect("Failed to bind Unix socket");
        info!("Unix socket listener bound to /tmp/sdrscanner");

        let mut next_client_id = 1u64;

        loop {
            debug!("Waiting for client connection...");
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let client_id = next_client_id;
                    next_client_id += 1;

                    // For now, assume all clients are Display clients
                    let connection_type = ConnectionType::Display;

                    // Clone client_count before incrementing to avoid move issues
                    let client_count = self.client_count.clone();
                    
                    // Increment client count
                    let new_count = client_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    info!("Client count incremented to {}", new_count);

                    // Clone channels for this client
                    let internal_tx = internal_tx.clone();
                    let external_tx = external_tx.clone();
                    let external_rx = external_tx.subscribe();
                    let realtime_rx = realtime_rx.clone();
                    let initial_display_info = initial_display_info.clone();
                    
                    // Spawn client handler directly
                    tokio::spawn(async move {
                        info!("Client {} connected (type: {:?})", client_id, connection_type);
                        
                        // Send initial display info directly to the client
                        use crate::io::client_handler::send_to_client;
                        if let Err(e) = send_to_client(&mut stream, &initial_display_info).await {
                            error!("Failed to send initial display info to client {}: {}", client_id, e);
                            let new_count = client_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
                            info!("Client {} disconnected during initial send, client count decremented to {}", client_id, new_count);
                            return;
                        }
                        debug!("Sent initial display info to client {}", client_id);
                        
                        handle_client(stream, client_id, connection_type, external_rx, realtime_rx, internal_tx, client_count).await;
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
