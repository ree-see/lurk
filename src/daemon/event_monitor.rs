use anyhow::Result;
use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc::Sender;
use tracing::{debug, error};

use crate::daemon::app_tracker::AppTracker;
use crate::models::event::{EventType as KEventType, Modifier};
use crate::models::keycode::KeyCode;
use crate::models::KeystrokeEvent;

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

        listen(move |event: Event| {
            if let Some(keystroke) = Self::process_event(&event, &app_tracker) {
                if let Err(e) = event_sender.send(keystroke) {
                    error!("Failed to send event: {}", e);
                }
            }
        })
        .map_err(|e| anyhow::anyhow!("Failed to start event listener: {:?}", e))
    }

    fn process_event(event: &Event, app_tracker: &AppTracker) -> Option<KeystrokeEvent> {
        let (key, event_type) = match &event.event_type {
            EventType::KeyPress(key) => (key, KEventType::Press),
            EventType::KeyRelease(key) => (key, KEventType::Release),
            _ => return None,
        };

        let key_code = KeyCode::from_rdev_key(key);
        let modifiers = Self::extract_modifiers(key);
        let application = app_tracker.get_current_app();

        debug!("Event: {:?} app={}", event_type, application);

        Some(KeystrokeEvent::new(
            key_code.0,
            event_type,
            modifiers,
            application,
        ))
    }

    fn extract_modifiers(key: &Key) -> Vec<Modifier> {
        let mut modifiers = Vec::new();

        match key {
            Key::ShiftLeft | Key::ShiftRight => modifiers.push(Modifier::Shift),
            Key::ControlLeft | Key::ControlRight => modifiers.push(Modifier::Control),
            Key::Alt | Key::AltGr => modifiers.push(Modifier::Alt),
            Key::MetaLeft | Key::MetaRight => modifiers.push(Modifier::Command),
            Key::CapsLock => modifiers.push(Modifier::CapsLock),
            Key::Function => modifiers.push(Modifier::Function),
            _ => {}
        }

        modifiers
    }
}
