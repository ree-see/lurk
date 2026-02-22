use anyhow::Result;
use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc::Sender;
use tracing::{debug, error, trace};

use crate::daemon::app_tracker::AppTracker;
use crate::models::event::{EventType as KEventType, Modifier};
use crate::models::keycode::KeyCode;
use crate::models::KeystrokeEvent;

/// Bundle IDs of sensitive applications where keystrokes should NOT be logged.
/// This prevents capturing passwords, banking credentials, and other sensitive input.
const SENSITIVE_APP_BLOCKLIST: &[&str] = &[
    // Password managers
    "com.1password.1password",
    "com.agilebits.onepassword7",
    "com.agilebits.onepassword-osx",
    "com.bitwarden.desktop",
    "com.lastpass.LastPass",
    "com.dashlane.dashlanephonefinal",
    "com.keepersecurity.keeper",
    "com.enpass.Enpass",
    "org.nickvision.keyring",
    // macOS system security
    "com.apple.keychainaccess",
    "com.apple.systempreferences",
    "com.apple.Passwords",
    // Banking apps (common examples)
    "com.chase.sig.android",
    "com.bankofamerica.bofa",
    "com.wellsfargo.mobile",
    "com.citi.mobile",
    // Crypto wallets
    "io.metamask.desktop",
    "com.ledger.live",
    "com.exodus.wallet",
    // SSH/Terminal with potential sensitive input
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "dev.warp.Warp-Stable",
    "com.microsoft.VSCode", // Often used for editing secrets
    "com.jetbrains.intellij",
    // VPN apps (may have credentials)
    "com.nordvpn.NordVPN",
    "com.expressvpn.ExpressVPN",
];

pub struct EventMonitor {
    app_tracker: AppTracker,
    event_sender: Sender<KeystrokeEvent>,
}

impl EventMonitor {
    pub fn new(event_sender: Sender<KeystrokeEvent>) -> Self {
        Self {
            app_tracker: AppTracker::new(),
            event_sender,
        }
    }

    pub fn start(self) -> Result<()> {
        let app_tracker = self.app_tracker;
        let event_sender = self.event_sender;
        let mut active_modifiers: Vec<Modifier> = Vec::new();

        listen(move |event: Event| {
            // Update modifier state before processing
            match &event.event_type {
                EventType::KeyPress(key) => {
                    if let Some(m) = Self::key_to_modifier(key) {
                        if !active_modifiers.contains(&m) {
                            active_modifiers.push(m);
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    if let Some(m) = Self::key_to_modifier(key) {
                        active_modifiers.retain(|x| x != &m);
                    }
                }
                _ => {}
            }

            if let Some(keystroke) = Self::process_event(&event, &app_tracker, &active_modifiers) {
                if let Err(e) = event_sender.send(keystroke) {
                    error!("Failed to send event: {}", e);
                }
            }
        })
        .map_err(|e| anyhow::anyhow!("Failed to start event listener: {:?}", e))
    }

    fn key_to_modifier(key: &Key) -> Option<Modifier> {
        match key {
            Key::ShiftLeft | Key::ShiftRight => Some(Modifier::Shift),
            Key::ControlLeft | Key::ControlRight => Some(Modifier::Control),
            Key::Alt | Key::AltGr => Some(Modifier::Alt),
            Key::MetaLeft | Key::MetaRight => Some(Modifier::Command),
            Key::CapsLock => Some(Modifier::CapsLock),
            Key::Function => Some(Modifier::Function),
            _ => None,
        }
    }

    fn process_event(event: &Event, app_tracker: &AppTracker, modifiers: &[Modifier]) -> Option<KeystrokeEvent> {
        let (key, event_type) = match &event.event_type {
            EventType::KeyPress(key) => (key, KEventType::Press),
            EventType::KeyRelease(key) => (key, KEventType::Release),
            _ => return None,
        };

        let application = app_tracker.get_current_app();

        if Self::is_sensitive_app(&application) {
            trace!("Skipping event from sensitive app");
            return None;
        }

        let key_code = KeyCode::from_rdev_key(key);

        debug!("Event: {:?} app={}", event_type, application);

        Some(KeystrokeEvent::new(
            key_code.0,
            event_type,
            modifiers.to_vec(),
            application,
        ))
    }

    fn is_sensitive_app(bundle_id: &str) -> bool {
        SENSITIVE_APP_BLOCKLIST
            .iter()
            .any(|blocked| bundle_id.eq_ignore_ascii_case(blocked))
    }
}
