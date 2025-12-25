use rdev::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub u32);

impl KeyCode {
    pub fn from_rdev_key(key: &Key) -> Self {
        let code = match key {
            Key::Alt => 0x3A,
            Key::AltGr => 0x3D,
            Key::Backspace => 0x33,
            Key::CapsLock => 0x39,
            Key::ControlLeft => 0x3B,
            Key::ControlRight => 0x3E,
            Key::Delete => 0x75,
            Key::DownArrow => 0x7D,
            Key::End => 0x77,
            Key::Escape => 0x35,
            Key::F1 => 0x7A,
            Key::F2 => 0x78,
            Key::F3 => 0x63,
            Key::F4 => 0x76,
            Key::F5 => 0x60,
            Key::F6 => 0x61,
            Key::F7 => 0x62,
            Key::F8 => 0x64,
            Key::F9 => 0x65,
            Key::F10 => 0x6D,
            Key::F11 => 0x67,
            Key::F12 => 0x6F,
            Key::Home => 0x73,
            Key::LeftArrow => 0x7B,
            Key::MetaLeft => 0x37,
            Key::MetaRight => 0x36,
            Key::PageDown => 0x79,
            Key::PageUp => 0x74,
            Key::Return => 0x24,
            Key::RightArrow => 0x7C,
            Key::ShiftLeft => 0x38,
            Key::ShiftRight => 0x3C,
            Key::Space => 0x31,
            Key::Tab => 0x30,
            Key::UpArrow => 0x7E,
            Key::PrintScreen => 0x69,
            Key::ScrollLock => 0x6B,
            Key::Pause => 0x71,
            Key::NumLock => 0x47,
            Key::BackQuote => 0x32,
            Key::Num1 => 0x12,
            Key::Num2 => 0x13,
            Key::Num3 => 0x14,
            Key::Num4 => 0x15,
            Key::Num5 => 0x17,
            Key::Num6 => 0x16,
            Key::Num7 => 0x1A,
            Key::Num8 => 0x1C,
            Key::Num9 => 0x19,
            Key::Num0 => 0x1D,
            Key::Minus => 0x1B,
            Key::Equal => 0x18,
            Key::KeyQ => 0x0C,
            Key::KeyW => 0x0D,
            Key::KeyE => 0x0E,
            Key::KeyR => 0x0F,
            Key::KeyT => 0x11,
            Key::KeyY => 0x10,
            Key::KeyU => 0x20,
            Key::KeyI => 0x22,
            Key::KeyO => 0x1F,
            Key::KeyP => 0x23,
            Key::LeftBracket => 0x21,
            Key::RightBracket => 0x1E,
            Key::KeyA => 0x00,
            Key::KeyS => 0x01,
            Key::KeyD => 0x02,
            Key::KeyF => 0x03,
            Key::KeyG => 0x05,
            Key::KeyH => 0x04,
            Key::KeyJ => 0x26,
            Key::KeyK => 0x28,
            Key::KeyL => 0x25,
            Key::SemiColon => 0x29,
            Key::Quote => 0x27,
            Key::BackSlash => 0x2A,
            Key::IntlBackslash => 0x0A,
            Key::KeyZ => 0x06,
            Key::KeyX => 0x07,
            Key::KeyC => 0x08,
            Key::KeyV => 0x09,
            Key::KeyB => 0x0B,
            Key::KeyN => 0x2D,
            Key::KeyM => 0x2E,
            Key::Comma => 0x2B,
            Key::Dot => 0x2F,
            Key::Slash => 0x2C,
            Key::Insert => 0x72,
            Key::KpReturn => 0x4C,
            Key::KpMinus => 0x4E,
            Key::KpPlus => 0x45,
            Key::KpMultiply => 0x43,
            Key::KpDivide => 0x4B,
            Key::Kp0 => 0x52,
            Key::Kp1 => 0x53,
            Key::Kp2 => 0x54,
            Key::Kp3 => 0x55,
            Key::Kp4 => 0x56,
            Key::Kp5 => 0x57,
            Key::Kp6 => 0x58,
            Key::Kp7 => 0x59,
            Key::Kp8 => 0x5B,
            Key::Kp9 => 0x5C,
            Key::KpDelete => 0x41,
            Key::Function => 0x3F,
            Key::Unknown(code) => *code as u32,
        };
        KeyCode(code)
    }

    pub fn to_name(&self) -> String {
        match self.0 {
            0x00 => "A".to_string(),
            0x01 => "S".to_string(),
            0x02 => "D".to_string(),
            0x03 => "F".to_string(),
            0x04 => "H".to_string(),
            0x05 => "G".to_string(),
            0x06 => "Z".to_string(),
            0x07 => "X".to_string(),
            0x08 => "C".to_string(),
            0x09 => "V".to_string(),
            0x0B => "B".to_string(),
            0x0C => "Q".to_string(),
            0x0D => "W".to_string(),
            0x0E => "E".to_string(),
            0x0F => "R".to_string(),
            0x10 => "Y".to_string(),
            0x11 => "T".to_string(),
            0x12 => "1".to_string(),
            0x13 => "2".to_string(),
            0x14 => "3".to_string(),
            0x15 => "4".to_string(),
            0x16 => "6".to_string(),
            0x17 => "5".to_string(),
            0x18 => "=".to_string(),
            0x19 => "9".to_string(),
            0x1A => "7".to_string(),
            0x1B => "-".to_string(),
            0x1C => "8".to_string(),
            0x1D => "0".to_string(),
            0x1E => "]".to_string(),
            0x1F => "O".to_string(),
            0x20 => "U".to_string(),
            0x21 => "[".to_string(),
            0x22 => "I".to_string(),
            0x23 => "P".to_string(),
            0x24 => "Return".to_string(),
            0x25 => "L".to_string(),
            0x26 => "J".to_string(),
            0x27 => "'".to_string(),
            0x28 => "K".to_string(),
            0x29 => ";".to_string(),
            0x2A => "\\".to_string(),
            0x2B => ",".to_string(),
            0x2C => "/".to_string(),
            0x2D => "N".to_string(),
            0x2E => "M".to_string(),
            0x2F => ".".to_string(),
            0x30 => "Tab".to_string(),
            0x31 => "Space".to_string(),
            0x32 => "`".to_string(),
            0x33 => "Backspace".to_string(),
            0x35 => "Escape".to_string(),
            0x36 => "RightCommand".to_string(),
            0x37 => "LeftCommand".to_string(),
            0x38 => "LeftShift".to_string(),
            0x39 => "CapsLock".to_string(),
            0x3A => "LeftAlt".to_string(),
            0x3B => "LeftControl".to_string(),
            0x3C => "RightShift".to_string(),
            0x3D => "RightAlt".to_string(),
            0x3E => "RightControl".to_string(),
            0x3F => "Function".to_string(),
            0x7A => "F1".to_string(),
            0x78 => "F2".to_string(),
            0x63 => "F3".to_string(),
            0x76 => "F4".to_string(),
            0x60 => "F5".to_string(),
            0x61 => "F6".to_string(),
            0x62 => "F7".to_string(),
            0x64 => "F8".to_string(),
            0x65 => "F9".to_string(),
            0x6D => "F10".to_string(),
            0x67 => "F11".to_string(),
            0x6F => "F12".to_string(),
            0x73 => "Home".to_string(),
            0x74 => "PageUp".to_string(),
            0x75 => "Delete".to_string(),
            0x77 => "End".to_string(),
            0x79 => "PageDown".to_string(),
            0x7B => "LeftArrow".to_string(),
            0x7C => "RightArrow".to_string(),
            0x7D => "DownArrow".to_string(),
            0x7E => "UpArrow".to_string(),
            code => format!("Unknown(0x{:02X})", code),
        }
    }
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_to_name_letters() {
        assert_eq!(KeyCode(0x00).to_name(), "A");
        assert_eq!(KeyCode(0x01).to_name(), "S");
        assert_eq!(KeyCode(0x02).to_name(), "D");
        assert_eq!(KeyCode(0x03).to_name(), "F");
    }

    #[test]
    fn test_keycode_to_name_special() {
        assert_eq!(KeyCode(0x24).to_name(), "Return");
        assert_eq!(KeyCode(0x31).to_name(), "Space");
        assert_eq!(KeyCode(0x33).to_name(), "Backspace");
        assert_eq!(KeyCode(0x35).to_name(), "Escape");
        assert_eq!(KeyCode(0x30).to_name(), "Tab");
    }

    #[test]
    fn test_keycode_to_name_modifiers() {
        assert_eq!(KeyCode(0x38).to_name(), "LeftShift");
        assert_eq!(KeyCode(0x3C).to_name(), "RightShift");
        assert_eq!(KeyCode(0x37).to_name(), "LeftCommand");
        assert_eq!(KeyCode(0x3B).to_name(), "LeftControl");
        assert_eq!(KeyCode(0x3A).to_name(), "LeftAlt");
    }

    #[test]
    fn test_keycode_to_name_unknown() {
        let name = KeyCode(0xFF).to_name();
        assert!(name.starts_with("Unknown"));
    }

    #[test]
    fn test_keycode_display() {
        assert_eq!(format!("{}", KeyCode(0x00)), "A");
        assert_eq!(format!("{}", KeyCode(0x31)), "Space");
    }

    #[test]
    fn test_keycode_from_rdev_key() {
        use rdev::Key;
        
        assert_eq!(KeyCode::from_rdev_key(&Key::KeyA).0, 0x00);
        assert_eq!(KeyCode::from_rdev_key(&Key::Space).0, 0x31);
        assert_eq!(KeyCode::from_rdev_key(&Key::Return).0, 0x24);
        assert_eq!(KeyCode::from_rdev_key(&Key::ShiftLeft).0, 0x38);
    }
}
