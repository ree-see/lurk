use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::keyboard_layout::QwertyLayout;
use crate::tui::widgets::KeyboardHeatmap;

pub fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
        ])
        .split(area);

    render_keyboard_section(f, app, chunks[0]);
    render_stats_section(f, app, chunks[1]);
}

fn render_keyboard_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Keyboard Heatmap ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = QwertyLayout::new();
    let frequencies = app.get_key_frequencies();
    let heatmap = KeyboardHeatmap::new(&layout, &frequencies);
    f.render_widget(heatmap, inner);
}

fn render_stats_section(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    render_top_keys(f, app, chunks[0]);
    render_stats_box(f, app, chunks[1]);
}

fn render_top_keys(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Top Keys ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let top_keys = app.get_top_keys(10);

    let items: Vec<ListItem> = top_keys
        .iter()
        .enumerate()
        .map(|(i, (name, count, pct))| {
            let bar_width = ((pct / 20.0) * 10.0) as usize;
            let bar: String = "█".repeat(bar_width.min(10));
            
            let line = Line::from(vec![
                Span::styled(
                    format!("{:2}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:8}", name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>8} ", count),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("({:>5.2}%) ", pct),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    bar,
                    Style::default().fg(Color::White),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_stats_box(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Statistics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let stats = app.get_stats();

    let text = vec![
        Line::from(vec![
            Span::styled("Total Presses:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>12}", format_number(stats.total_presses)),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Daily Average:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>12}", format_number(stats.daily_average)),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Est. WPM:       ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>12}", stats.estimated_wpm),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Median Delay:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>10}ms", stats.median_delay_ms),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Time Range:     ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", app.time_range.label()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Days Active:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>12}", stats.days_active),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
