//! WebSocket server and client for remote keystroke sync.

pub mod client;

pub use client::{start_remote_client, RemoteClientConfig};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::thread;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::models::KeystrokeEvent;
use crate::storage::Database;

/// Run the lurk server, listening for WebSocket connections.
pub async fn run_server(port: u16, db_path: &PathBuf) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Lurk server listening on ws://0.0.0.0:{}", port);
    info!("Database: {:?}", db_path);
    
    // Channel for events from all clients (async -> sync bridge)
    let (async_tx, mut async_rx) = mpsc::channel::<KeystrokeEvent>(1000);
    let (sync_tx, sync_rx) = std_mpsc::channel::<KeystrokeEvent>();
    
    // Spawn sync database writer thread (rusqlite isn't Send)
    let db_path_clone = db_path.clone();
    thread::spawn(move || {
        let db = match Database::new(&db_path_clone) {
            Ok(db) => db,
            Err(e) => {
                error!("Failed to open database: {}", e);
                return;
            }
        };
        
        let mut batch: Vec<KeystrokeEvent> = Vec::with_capacity(100);
        let mut last_flush = std::time::Instant::now();
        
        loop {
            match sync_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(event) => {
                    batch.push(event);
                    
                    // Flush if batch is large or enough time has passed
                    if batch.len() >= 100 || last_flush.elapsed().as_secs() >= 5 {
                        for event in batch.drain(..) {
                            if let Err(e) = db.insert_event(&event) {
                                error!("Failed to write event: {}", e);
                            }
                        }
                        last_flush = std::time::Instant::now();
                    }
                }
                Err(std_mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic flush
                    if !batch.is_empty() {
                        for event in batch.drain(..) {
                            if let Err(e) = db.insert_event(&event) {
                                error!("Failed to write event: {}", e);
                            }
                        }
                        last_flush = std::time::Instant::now();
                    }
                }
                Err(std_mpsc::RecvTimeoutError::Disconnected) => {
                    // Flush remaining and exit
                    for event in batch.drain(..) {
                        let _ = db.insert_event(&event);
                    }
                    info!("Database writer shutting down");
                    break;
                }
            }
        }
    });
    
    // Spawn async -> sync bridge task
    tokio::spawn(async move {
        while let Some(event) = async_rx.recv().await {
            if sync_tx.send(event).is_err() {
                error!("Database writer thread died");
                break;
            }
        }
    });
    
    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        let tx = async_tx.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, tx).await {
                error!("Connection error from {}: {}", addr, e);
            }
        });
    }
    
    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    tx: mpsc::Sender<KeystrokeEvent>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    
    info!("WebSocket connection established with {}", addr);
    
    // Send welcome message
    write.send(Message::Text(
        serde_json::json!({
            "type": "welcome",
            "message": "Connected to lurk server"
        }).to_string()
    )).await?;
    
    let mut event_count: u64 = 0;
    
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::Event(event) => {
                                if tx.send(event).await.is_err() {
                                    error!("Failed to queue event");
                                }
                                event_count += 1;
                                
                                // Ack every 100 events
                                if event_count % 100 == 0 {
                                    let ack = serde_json::json!({
                                        "type": "ack",
                                        "count": event_count
                                    });
                                    write.send(Message::Text(ack.to_string())).await?;
                                }
                            }
                            ClientMessage::Ping => {
                                write.send(Message::Text(
                                    serde_json::json!({"type": "pong"}).to_string()
                                )).await?;
                            }
                            ClientMessage::Stats => {
                                let stats = serde_json::json!({
                                    "type": "stats",
                                    "events_received": event_count
                                });
                                write.send(Message::Text(stats.to_string())).await?;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Invalid message from {}: {}", addr, e);
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                write.send(Message::Pong(data)).await?;
            }
            Ok(Message::Close(_)) => {
                info!("Client {} disconnected (received {} events)", addr, event_count);
                break;
            }
            Err(e) => {
                error!("WebSocket error from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }
    
    info!("Connection closed: {} (total events: {})", addr, event_count);
    Ok(())
}

/// Messages from client to server
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMessage {
    Event(KeystrokeEvent),
    Ping,
    Stats,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_message_parsing() {
        let event_json = r#"{
            "type": "event",
            "timestamp": 1234567890,
            "key_code": 65,
            "event_type": "press",
            "modifiers": ["shift"],
            "application": "com.test.app"
        }"#;
        
        let msg: ClientMessage = serde_json::from_str(event_json).unwrap();
        match msg {
            ClientMessage::Event(e) => {
                assert_eq!(e.key_code, 65);
            }
            _ => panic!("Expected Event"),
        }
    }
    
    #[test]
    fn test_ping_message() {
        let ping_json = r#"{"type": "ping"}"#;
        let msg: ClientMessage = serde_json::from_str(ping_json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }
}
