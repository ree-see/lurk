//! WebSocket client for sending keystroke events to a remote server.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use url::Url;

use crate::models::KeystrokeEvent;

/// Configuration for the remote client
pub struct RemoteClientConfig {
    pub url: String,
    pub token: Option<String>,
    pub reconnect_interval: Duration,
    pub buffer_size: usize,
}

impl Default for RemoteClientConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:9999".to_string(),
            token: None,
            reconnect_interval: Duration::from_secs(5),
            buffer_size: 1000,
        }
    }
}

/// Run the client that sends events to a remote server.
/// This wraps the sync mpsc receiver from the event monitor.
pub fn start_remote_client(
    config: RemoteClientConfig,
    rx: Receiver<KeystrokeEvent>,
) -> Result<()> {
    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move { run_client_loop(config, rx).await })
}

async fn run_client_loop(
    config: RemoteClientConfig,
    rx: Receiver<KeystrokeEvent>,
) -> Result<()> {
    let url_str = config.url.clone();
    let _url = Url::parse(&url_str)?; // Validate URL format

    // Bridge from sync to async
    let (async_tx, mut async_rx) = tokio_mpsc::channel::<KeystrokeEvent>(config.buffer_size);

    // Spawn thread to forward from sync receiver
    let _forward_handle = std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            if async_tx.blocking_send(event).is_err() {
                break;
            }
        }
    });

    // Buffer for events when disconnected
    let mut buffer: Vec<KeystrokeEvent> = Vec::new();

    'connect: loop {
        info!("Connecting to {}...", url_str);

        match connect_async(url_str.clone()).await {
            Ok((ws_stream, _)) => {
                info!("Connected to server");
                let (mut write, mut read) = ws_stream.split();

                // Send auth message if token configured
                if let Some(ref t) = config.token {
                    let auth_msg = serde_json::json!({"type": "auth", "token": t});
                    if write
                        .send(Message::Text(auth_msg.to_string()))
                        .await
                        .is_err()
                    {
                        warn!("Failed to send auth message");
                        continue 'connect;
                    }
                    // Read welcome (or error) response
                    match read.next().await {
                        Some(Ok(Message::Text(_))) => {
                            info!("Auth accepted by server");
                        }
                        _ => {
                            warn!("Auth failed or server closed connection");
                            continue 'connect;
                        }
                    }
                }

                // Drain buffer first
                if !buffer.is_empty() {
                    info!("Sending {} buffered events", buffer.len());
                    let events_to_send: Vec<_> = buffer.drain(..).collect();
                    let mut failed = false;
                    for event in events_to_send {
                        if failed {
                            buffer.push(event);
                            continue;
                        }
                        let msg = serde_json::json!({
                            "type": "event",
                            "timestamp": event.timestamp,
                            "key_code": event.key_code,
                            "event_type": event.event_type,
                            "modifiers": event.modifiers,
                            "application": event.application,
                        });
                        if write.send(Message::Text(msg.to_string())).await.is_err() {
                            buffer.push(event);
                            failed = true;
                        }
                    }
                    if failed {
                        continue 'connect; // Reconnect
                    }
                }

                // Main send loop
                loop {
                    tokio::select! {
                        // Handle incoming messages (acks, pongs, etc)
                        msg = read.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    // Log server responses
                                    if let Ok(resp) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if resp.get("type").and_then(|t| t.as_str()) == Some("ack") {
                                            if let Some(count) = resp.get("count").and_then(|c| c.as_u64()) {
                                                info!("Server ack: {} events received", count);
                                            }
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) | None => {
                                    warn!("Server closed connection");
                                    break;
                                }
                                Some(Err(e)) => {
                                    error!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }

                        // Send events
                        event = async_rx.recv() => {
                            match event {
                                Some(event) => {
                                    let msg = serde_json::json!({
                                        "type": "event",
                                        "timestamp": event.timestamp,
                                        "key_code": event.key_code,
                                        "event_type": event.event_type,
                                        "modifiers": event.modifiers,
                                        "application": event.application,
                                    });

                                    if write.send(Message::Text(msg.to_string())).await.is_err() {
                                        buffer.push(event);
                                        warn!("Send failed, buffering event");
                                        break;
                                    }
                                }
                                None => {
                                    // Channel closed, we're done
                                    info!("Event channel closed, shutting down");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }

        // Reconnect delay
        warn!(
            "Disconnected. Buffered {} events. Reconnecting in {:?}...",
            buffer.len(),
            config.reconnect_interval
        );
        tokio::time::sleep(config.reconnect_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RemoteClientConfig::default();
        assert_eq!(config.url, "ws://localhost:9999");
        assert_eq!(config.reconnect_interval, Duration::from_secs(5));
        assert!(config.token.is_none());
    }

    #[test]
    fn test_config_with_token() {
        let config = RemoteClientConfig {
            url: "ws://server:9999".to_string(),
            token: Some("mysecret".to_string()),
            ..Default::default()
        };
        assert_eq!(config.token.as_deref(), Some("mysecret"));
    }
}
