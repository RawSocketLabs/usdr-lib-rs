use tokio::sync::{broadcast, mpsc, watch};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::Duration;
use shared::{External, ConnectionType, FreqBlock};
use crate::io::Internal;

// Helper function to decrement client count and log disconnection
fn handle_client_disconnect(client_id: u64, client_count: &std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    let new_count = client_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
    eprintln!("Client {} disconnected, client count decremented to {}", client_id, new_count);
}

pub async fn handle_client(
    mut stream: UnixStream,
    client_id: u64,
    connection_type: ConnectionType,
    mut external_rx: broadcast::Receiver<External>,
    mut realtime_rx: watch::Receiver<FreqBlock>,
    _internal_tx: mpsc::Sender<Internal>,
    client_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {

    loop {
        tokio::select! {
            // Handle incoming messages with batching
            result = read_client_message(&mut stream) => {
                match result {
                    Ok(Some(msg)) => {
                        // Handle External messages from clients
                        match msg {
                            shared::External::Connection(conn_type) => {
                                // eprintln!("Client {} sent connection type: {:?}", client_id, conn_type);
                                // Could update connection type here if needed
                            }
                            _ => {
                                eprintln!("Client {} sent unexpected message: {:?}", client_id, msg);
                            }
                        }
                    }
                    Ok(None) => {
                        // No message available - client still connected, continue loop
                        continue;
                    }
                    Err(e) => {
                        // Check if this is a connection error or just a timeout
                        if e.to_string().contains("Broken pipe") || 
                           e.to_string().contains("Connection reset") ||
                           e.to_string().contains("End of file") {
                            handle_client_disconnect(client_id, &client_count);
                        } else {
                            eprintln!("Client {} error: {}", client_id, e);
                            handle_client_disconnect(client_id, &client_count);
                        }
                        return;
                    }
                }
            }

            // Handle realtime freq_block data (only for Display clients)
            _ = realtime_rx.changed(), if connection_type == ConnectionType::Display => {
                let freq_block = realtime_rx.borrow().clone();
                let realtime_msg = External::Realtime(freq_block);
                // eprintln!("Client {} received realtime data", client_id);
                if let Err(e) = send_to_client(&mut stream, &realtime_msg).await {
                    eprintln!("Failed to send realtime data to client {}: {}", client_id, e);
                    handle_client_disconnect(client_id, &client_count);
                    return;
                }
            }

            // Handle other outgoing messages
            result = external_rx.recv() => {
                match result {
                    Ok(msg) => {
                        // eprintln!("Client {} received external message: {:?}", client_id, msg);
                        if let Err(e) = send_to_client(&mut stream, &msg).await {
                            eprintln!("Failed to send to client {}: {}", client_id, e);
                            handle_client_disconnect(client_id, &client_count);
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Client {} lagged by {} messages", client_id, n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        handle_client_disconnect(client_id, &client_count);
                        return;
                    }
                }
            }
            
            // Periodic yield to ensure fairness
            _ = tokio::time::sleep(Duration::from_millis(20)) => {
                // Explicit yield to ensure fairness
                tokio::task::yield_now().await;
            }
        }
    }
}

async fn read_client_message(stream: &mut UnixStream) -> Result<Option<shared::External>, Box<dyn std::error::Error + Send + Sync>> {
    let config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
        
    // Non-blocking read with reasonable timeout
    match tokio::time::timeout(Duration::from_millis(10), stream.read_u32()).await {
        Ok(Ok(len)) => {
            let mut buffer = vec![0u8; len as usize];
            stream.read_exact(&mut buffer).await?;
            let (msg, _): (shared::External, _) = bincode::decode_from_slice(&buffer, config)?;
            Ok(Some(msg))
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None), // Timeout - no message available, client still connected
    }
}

pub async fn send_to_client(stream: &mut UnixStream, msg: &External) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    
    let serialized = bincode::encode_to_vec(msg, config)?;
    // println!("Sending to client message: {:?}", msg);
    
    // Write length prefix
    stream.write_u32(serialized.len() as u32).await?;
    // Write data
    stream.write_all(&serialized).await?;
    stream.flush().await?;
    
    Ok(())
}
