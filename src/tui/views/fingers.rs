use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::keyboard_layout::{Finger, Hand, QwertyLayout};
use crate::tui::widgets::KeyboardHeatmap;

pub fn render_fingers(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
        ])
        .split(area);

    render_keyboard_with_fingers(f, app, chunks[0]);
    
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    render_finger_load(f, app, bottom_chunks[0]);
    render_hand_balance(f, app, bottom_chunks[1]);
    render_same_finger_bigrams(f, app, bottom_chunks[2]);
}

fn render_keyboard_with_fingers(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Finger Assignments (QWERTY) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = QwertyLayout::new();
    let frequencies = app.get_key_frequencies();
    let heatmap = KeyboardHeatmap::new(&layout, &frequencies).show_fingers(true);
    f.render_widget(heatmap, inner);
}

fn render_finger_load(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Finger Load ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let finger_loads = app.get_finger_loads();

    let items: Vec<ListItem> = finger_loads
        .iter()
        .map(|(finger, pct)| {
            let bar_width = (pct / 3.0) as usize;
            let bar: String = "█".repeat(bar_width.min(12));
            
            let color = match finger.hand() {
                Hand::Left => Color::Cyan,
                Hand::Right => Color::Magenta,
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<12}", finger_name(finger)), Style::default().fg(color)),
                Span::styled(format!("{:>5.1}% ", pct), Style::default().fg(Color::Gray)),
                Span::styled(bar, Style::default().fg(color)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_hand_balance(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Hand Balance ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (left_pct, right_pct) = app.get_hand_balance();

    let balance_status = if left_pct >= 45.0 && left_pct <= 55.0 {
        ("✓ Good", Color::Green)
    } else if left_pct >= 40.0 && left_pct <= 60.0 {
        ("○ Fair", Color::Yellow)
    } else {
        ("✗ Imbalanced", Color::Red)
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Left Hand:   ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:>5.1}%", left_pct),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Right Hand:  ", Style::default().fg(Color::Magenta)),
            Span::styled(
                format!("{:>5.1}%", right_pct),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Balance:     ", Style::default().fg(Color::Gray)),
            Span::styled(balance_status.0, Style::default().fg(balance_status.1)),
        ]),
        Line::from(vec![
            Span::styled("  (Ideal: 45-55%)", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}

fn render_same_finger_bigrams(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Same-Finger Bigrams ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let bigram_stats = app.get_bigram_finger_stats();

    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Same Finger: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", bigram_stats.same_finger_pct),
                Style::default().fg(if bigram_stats.same_finger_pct > 10.0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Alternating: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", bigram_stats.alternation_pct),
                Style::default().fg(Color::Green),
            ),
        ])),
        ListItem::new(Line::from("")),
        ListItem::new(Line::from(vec![
            Span::styled("Worst (same finger):", Style::default().fg(Color::Gray)),
        ])),
    ];

    for (bigram, count) in bigram_stats.worst_same_finger.iter().take(4) {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {} ", bigram), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{}", count), Style::default().fg(Color::DarkGray)),
        ])));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn finger_name(finger: &Finger) -> &'static str {
    match finger {
        Finger::LeftPinky => "Left Pinky",
        Finger::LeftRing => "Left Ring",
        Finger::LeftMiddle => "Left Middle",
        Finger::LeftIndex => "Left Index",
        Finger::RightIndex => "Right Index",
        Finger::RightMiddle => "Right Middle",
        Finger::RightRing => "Right Ring",
        Finger::RightPinky => "Right Pinky",
        Finger::Thumb => "Thumb",
    }
}
