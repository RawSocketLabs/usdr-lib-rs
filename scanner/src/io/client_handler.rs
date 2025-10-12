use tokio::sync::{broadcast, mpsc, watch};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::Duration;
use shared::{External, ConnectionType, FreqBlock};
use crate::io::Internal;

pub async fn handle_client(
    mut stream: UnixStream,
    client_id: u64,
    connection_type: ConnectionType,
    mut external_rx: broadcast::Receiver<External>,
    mut realtime_rx: watch::Receiver<FreqBlock>,
    internal_tx: mpsc::Sender<Internal>,
) {
    let mut message_batch = Vec::new();

    loop {
        tokio::select! {
            // Handle incoming messages with batching
            result = read_client_message(&mut stream) => {
                match result {
                    Ok(Some(msg)) => {
                        message_batch.push(msg);
                        
                        // Process batch when it reaches size limit
                        if message_batch.len() >= 5 {
                            for msg in message_batch.drain(..) {
                                if let Err(_) = internal_tx.send(msg).await {
                                    eprintln!("Failed to send message from client {}", client_id);
                                    return;
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        // Process remaining batch and disconnect
                        for msg in message_batch.drain(..) {
                            let _ = internal_tx.send(msg).await;
                        }
                        eprintln!("Client {} disconnected", client_id);
                        return;
                    }
                    Err(e) => {
                        eprintln!("Client {} error: {}", client_id, e);
                        return;
                    }
                }
            }

            // Handle realtime freq_block data (only for Display clients)
            _ = realtime_rx.changed(), if connection_type == ConnectionType::Display => {
                let freq_block = realtime_rx.borrow().clone();
                let realtime_msg = External::Realtime(freq_block);
                if let Err(e) = send_to_client(&mut stream, &realtime_msg).await {
                    eprintln!("Failed to send realtime data to client {}: {}", client_id, e);
                    return;
                }
            }

            // Handle other outgoing messages
            result = external_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = send_to_client(&mut stream, &msg).await {
                            eprintln!("Failed to send to client {}: {}", client_id, e);
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Client {} lagged by {} messages", client_id, n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return;
                    }
                }
            }
            
            // Periodic batch flush and yield
            _ = tokio::time::sleep(Duration::from_millis(20)) => {
                if !message_batch.is_empty() {
                    for msg in message_batch.drain(..) {
                        if let Err(_) = internal_tx.send(msg).await {
                            eprintln!("Failed to send batched message from client {}", client_id);
                            return;
                        }
                    }
                }
                // Explicit yield to ensure fairness
                tokio::task::yield_now().await;
            }
        }
    }
}

async fn read_client_message(stream: &mut UnixStream) -> Result<Option<Internal>, Box<dyn std::error::Error + Send + Sync>> {
    // Non-blocking read with very short timeout - let tokio::select! handle main timeouts
    match tokio::time::timeout(Duration::from_millis(1), stream.read_u32()).await {
        Ok(Ok(len)) => {
            let mut buffer = vec![0u8; len as usize];
            stream.read_exact(&mut buffer).await?;
            let (msg, _): (Internal, _) = bincode::decode_from_slice(&buffer, bincode::config::standard())?;
            Ok(Some(msg))
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None), // Timeout - no message available
    }
}

async fn send_to_client(stream: &mut UnixStream, msg: &External) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let serialized = bincode::encode_to_vec(msg, bincode::config::standard())?;
    
    let len = serialized.len() as u32;
    stream.write_u32(len).await?;
    stream.write_all(&serialized).await?;
    stream.flush().await?;
    
    Ok(())
}
