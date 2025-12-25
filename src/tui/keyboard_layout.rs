use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    Thumb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl Finger {
    pub fn hand(&self) -> Hand {
        match self {
            Finger::LeftPinky | Finger::LeftRing | Finger::LeftMiddle | Finger::LeftIndex => {
                Hand::Left
            }
            Finger::RightIndex | Finger::RightMiddle | Finger::RightRing | Finger::RightPinky => {
                Hand::Right
            }
            Finger::Thumb => Hand::Right,
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Finger::LeftPinky => "L4",
            Finger::LeftRing => "L3",
            Finger::LeftMiddle => "L2",
            Finger::LeftIndex => "L1",
            Finger::RightIndex => "R1",
            Finger::RightMiddle => "R2",
            Finger::RightRing => "R3",
            Finger::RightPinky => "R4",
            Finger::Thumb => "Th",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub keycode: u32,
    pub label: &'static str,
    pub width: u16,
    pub finger: Finger,
}

pub struct QwertyLayout {
    pub rows: Vec<Vec<KeyInfo>>,
    finger_map: HashMap<u32, Finger>,
}

impl QwertyLayout {
    pub fn new() -> Self {
        let rows = vec![
            vec![
                KeyInfo { keycode: 0x32, label: "`", width: 2, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x12, label: "1", width: 2, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x13, label: "2", width: 2, finger: Finger::LeftRing },
                KeyInfo { keycode: 0x14, label: "3", width: 2, finger: Finger::LeftMiddle },
                KeyInfo { keycode: 0x15, label: "4", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x17, label: "5", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x16, label: "6", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x1A, label: "7", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x1C, label: "8", width: 2, finger: Finger::RightMiddle },
                KeyInfo { keycode: 0x19, label: "9", width: 2, finger: Finger::RightRing },
                KeyInfo { keycode: 0x1D, label: "0", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x1B, label: "-", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x18, label: "=", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x33, label: "⌫", width: 3, finger: Finger::RightPinky },
            ],
            vec![
                KeyInfo { keycode: 0x30, label: "⇥", width: 3, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x0C, label: "Q", width: 2, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x0D, label: "W", width: 2, finger: Finger::LeftRing },
                KeyInfo { keycode: 0x0E, label: "E", width: 2, finger: Finger::LeftMiddle },
                KeyInfo { keycode: 0x0F, label: "R", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x11, label: "T", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x10, label: "Y", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x20, label: "U", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x22, label: "I", width: 2, finger: Finger::RightMiddle },
                KeyInfo { keycode: 0x1F, label: "O", width: 2, finger: Finger::RightRing },
                KeyInfo { keycode: 0x23, label: "P", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x21, label: "[", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x1E, label: "]", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x2A, label: "\\", width: 2, finger: Finger::RightPinky },
            ],
            vec![
                KeyInfo { keycode: 0x39, label: "⇪", width: 4, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x00, label: "A", width: 2, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x01, label: "S", width: 2, finger: Finger::LeftRing },
                KeyInfo { keycode: 0x02, label: "D", width: 2, finger: Finger::LeftMiddle },
                KeyInfo { keycode: 0x03, label: "F", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x05, label: "G", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x04, label: "H", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x26, label: "J", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x28, label: "K", width: 2, finger: Finger::RightMiddle },
                KeyInfo { keycode: 0x25, label: "L", width: 2, finger: Finger::RightRing },
                KeyInfo { keycode: 0x29, label: ";", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x27, label: "'", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x24, label: "⏎", width: 4, finger: Finger::RightPinky },
            ],
            vec![
                KeyInfo { keycode: 0x38, label: "⇧", width: 5, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x06, label: "Z", width: 2, finger: Finger::LeftPinky },
                KeyInfo { keycode: 0x07, label: "X", width: 2, finger: Finger::LeftRing },
                KeyInfo { keycode: 0x08, label: "C", width: 2, finger: Finger::LeftMiddle },
                KeyInfo { keycode: 0x09, label: "V", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x0B, label: "B", width: 2, finger: Finger::LeftIndex },
                KeyInfo { keycode: 0x2D, label: "N", width: 2, finger: Finger::RightIndex },
                KeyInfo { keycode: 0x2E, label: "M", width: 2, finger: Finger::RightMiddle },
                KeyInfo { keycode: 0x2B, label: ",", width: 2, finger: Finger::RightMiddle },
                KeyInfo { keycode: 0x2F, label: ".", width: 2, finger: Finger::RightRing },
                KeyInfo { keycode: 0x2C, label: "/", width: 2, finger: Finger::RightPinky },
                KeyInfo { keycode: 0x3C, label: "⇧", width: 5, finger: Finger::RightPinky },
            ],
            vec![
                KeyInfo { keycode: 0x31, label: "␣", width: 20, finger: Finger::Thumb },
            ],
        ];

        let mut finger_map = HashMap::new();
        for row in &rows {
            for key in row {
                finger_map.insert(key.keycode, key.finger);
            }
        }

        Self { rows, finger_map }
    }

    pub fn get_finger(&self, keycode: u32) -> Option<Finger> {
        self.finger_map.get(&keycode).copied()
    }

    pub fn total_width(&self) -> u16 {
        self.rows
            .iter()
            .map(|row| row.iter().map(|k| k.width).sum::<u16>())
            .max()
            .unwrap_or(0)
    }
}

impl Default for QwertyLayout {
    fn default() -> Self {
        Self::new()
    }
}
