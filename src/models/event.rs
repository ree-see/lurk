use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub timestamp: i64,
    pub key_code: u32,
    pub event_type: EventType,
    pub modifiers: Vec<Modifier>,
    pub application: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Shift,
    Control,
    Alt,
    Command,
    CapsLock,
    Function,
}

impl KeystrokeEvent {
    pub fn new(
        key_code: u32,
        event_type: EventType,
        modifiers: Vec<Modifier>,
        application: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;

        Self {
            timestamp,
            key_code,
            event_type,
            modifiers,
            application,
        }
    }
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Press => "press",
            EventType::Release => "release",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Modifier::Shift => "shift",
            Modifier::Control => "control",
            Modifier::Alt => "alt",
            Modifier::Command => "command",
            Modifier::CapsLock => "capslock",
            Modifier::Function => "function",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystroke_event_creation() {
        let event = KeystrokeEvent::new(
            0x00,
            EventType::Press,
            vec![Modifier::Shift],
            "com.test.app".to_string(),
        );

        assert_eq!(event.key_code, 0x00);
        assert_eq!(event.event_type, EventType::Press);
        assert_eq!(event.modifiers, vec![Modifier::Shift]);
        assert_eq!(event.application, "com.test.app");
        assert!(event.timestamp > 0);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Press.as_str(), "press");
        assert_eq!(EventType::Release.as_str(), "release");
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::Press), "press");
        assert_eq!(format!("{}", EventType::Release), "release");
    }

    #[test]
    fn test_modifier_display() {
        assert_eq!(format!("{}", Modifier::Shift), "shift");
        assert_eq!(format!("{}", Modifier::Control), "control");
        assert_eq!(format!("{}", Modifier::Alt), "alt");
        assert_eq!(format!("{}", Modifier::Command), "command");
    }

    #[test]
    fn test_event_serialization() {
        let event = KeystrokeEvent {
            timestamp: 1234567890,
            key_code: 0x00,
            event_type: EventType::Press,
            modifiers: vec![Modifier::Shift, Modifier::Command],
            application: "com.test.app".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event_type\":\"press\""));
        assert!(json.contains("\"modifiers\":[\"shift\",\"command\"]"));

        let deserialized: KeystrokeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key_code, event.key_code);
        assert_eq!(deserialized.event_type, event.event_type);
    }
}
