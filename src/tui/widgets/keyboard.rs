use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::tui::keyboard_layout::{QwertyLayout, Finger};

pub struct KeyboardHeatmap<'a> {
    layout: &'a QwertyLayout,
    frequencies: &'a HashMap<u32, f64>,
    show_fingers: bool,
}

impl<'a> KeyboardHeatmap<'a> {
    pub fn new(layout: &'a QwertyLayout, frequencies: &'a HashMap<u32, f64>) -> Self {
        Self {
            layout,
            frequencies,
            show_fingers: false,
        }
    }

    pub fn show_fingers(mut self, show: bool) -> Self {
        self.show_fingers = show;
        self
    }

    fn frequency_to_char(percentage: f64, max_percentage: f64) -> char {
        if max_percentage <= 0.0 {
            return ' ';
        }
        let normalized = (percentage / max_percentage) * 100.0;
        match normalized {
            p if p >= 75.0 => '█',
            p if p >= 50.0 => '▓',
            p if p >= 25.0 => '▒',
            p if p > 0.0 => '░',
            _ => ' ',
        }
    }

    fn finger_to_gray(finger: Finger) -> Color {
        match finger {
            Finger::LeftPinky | Finger::RightPinky => Color::Rgb(80, 80, 80),
            Finger::LeftRing | Finger::RightRing => Color::Rgb(120, 120, 120),
            Finger::LeftMiddle | Finger::RightMiddle => Color::Rgb(160, 160, 160),
            Finger::LeftIndex | Finger::RightIndex => Color::Rgb(200, 200, 200),
            Finger::Thumb => Color::Rgb(220, 220, 220),
        }
    }
}

impl<'a> Widget for KeyboardHeatmap<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 6 {
            return;
        }

        let max_freq = self
            .frequencies
            .values()
            .copied()
            .fold(0.0_f64, f64::max);

        let start_x = area.x + 1;
        let mut y = area.y;

        for row in &self.layout.rows {
            let mut x = start_x;

            if row.len() == 1 && row[0].label == "␣" {
                x = start_x + 8;
            }

            for key in row {
                let freq = self.frequencies.get(&key.keycode).copied().unwrap_or(0.0);
                let heat_char = Self::frequency_to_char(freq, max_freq);

                let style = if self.show_fingers {
                    Style::default().fg(Self::finger_to_gray(key.finger))
                } else {
                    Style::default().fg(Color::White)
                };

                if x + key.width <= area.x + area.width && y < area.y + area.height {
                    let display = if key.width >= 3 {
                        format!("{}{}", heat_char, key.label)
                    } else {
                        key.label.to_string()
                    };

                    buf.set_string(x, y, &display, style);

                    if freq > 0.0 && key.width >= 2 {
                        let heat_style = Style::default().fg(Color::Rgb(
                            ((freq / max_freq) * 255.0) as u8,
                            ((freq / max_freq) * 255.0) as u8,
                            ((freq / max_freq) * 255.0) as u8,
                        ));
                        buf.set_string(x, y, &heat_char.to_string(), heat_style);
                    }
                }

                x += key.width + 1;
            }
            y += 1;
        }

        if y < area.y + area.height {
            let legend = "░Low ▒Med ▓High █Max";
            buf.set_string(start_x, y, legend, Style::default().fg(Color::DarkGray));
        }
    }
}
