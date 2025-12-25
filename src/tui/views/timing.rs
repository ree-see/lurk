use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render_timing(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Min(10),
        ])
        .split(area);

    render_timing_histogram(f, app, chunks[0]);
    
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    render_speed_metrics(f, app, bottom_chunks[0]);
    render_fastest_pairs(f, app, bottom_chunks[1]);
    render_slowest_pairs(f, app, bottom_chunks[2]);
}

fn render_timing_histogram(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Inter-Key Timing Distribution ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let histogram = app.get_timing_histogram();
    
    if histogram.is_empty() {
        let msg = Paragraph::new("No timing data available")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
        return;
    }

    let data: Vec<(&str, u64)> = histogram
        .iter()
        .map(|(label, count)| (label.as_str(), *count))
        .collect();

    let chart = BarChart::default()
        .data(&data)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::White))
        .value_style(Style::default().fg(Color::Gray));

    f.render_widget(chart, inner);
}

fn render_speed_metrics(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Speed Metrics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let metrics = app.get_speed_metrics();

    let consistency_color = match metrics.consistency.as_str() {
        "Excellent" => Color::Green,
        "Good" => Color::Cyan,
        "Fair" => Color::Yellow,
        _ => Color::Red,
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Mean Inter-Key:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6.0}ms", metrics.mean_ms),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Median:           ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}ms", metrics.median_ms),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("P95:              ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}ms", metrics.p95_ms),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("P99:              ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}ms", metrics.p99_ms),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Estimated WPM:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}", metrics.estimated_wpm),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Burst WPM:        ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}", metrics.burst_wpm),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Consistency:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:>6}", metrics.consistency),
                Style::default().fg(consistency_color),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_fastest_pairs(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Fastest Pairs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let pairs = app.get_fastest_pairs();

    let header = Line::from(vec![
        Span::styled(
            format!("{:<6} {:>6} {:>6}", "Pair", "Med", "Count"),
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut items = vec![ListItem::new(header)];

    for (pair, median_ms, count) in pairs.iter().take(8) {
        let line = Line::from(vec![
            Span::styled(format!("{:<6}", pair), Style::default().fg(Color::Green)),
            Span::styled(format!("{:>5}ms", median_ms), Style::default().fg(Color::White)),
            Span::styled(format!("{:>6}", count), Style::default().fg(Color::DarkGray)),
        ]);
        items.push(ListItem::new(line));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_slowest_pairs(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Slowest Pairs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let pairs = app.get_slowest_pairs();

    let header = Line::from(vec![
        Span::styled(
            format!("{:<6} {:>6} {:>6}", "Pair", "Med", "Count"),
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut items = vec![ListItem::new(header)];

    for (pair, median_ms, count) in pairs.iter().take(8) {
        let line = Line::from(vec![
            Span::styled(format!("{:<6}", pair), Style::default().fg(Color::Red)),
            Span::styled(format!("{:>5}ms", median_ms), Style::default().fg(Color::White)),
            Span::styled(format!("{:>6}", count), Style::default().fg(Color::DarkGray)),
        ]);
        items.push(ListItem::new(line));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
