use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
    Frame,
};

use crate::tui::app::App;

pub fn render_trends(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(8),
            Constraint::Length(8),
        ])
        .split(area);

    render_daily_chart(f, app, chunks[0]);
    render_weekly_comparison(f, app, chunks[1]);
    render_app_distribution(f, app, chunks[2]);
}

fn render_daily_chart(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Daily Key Presses ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let daily_counts = app.get_daily_counts();
    
    if daily_counts.is_empty() {
        let msg = Paragraph::new("No data available")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
        return;
    }

    let max_count = daily_counts.iter().max().copied().unwrap_or(1);
    let data: Vec<u64> = daily_counts.iter().copied().collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .max(max_count)
        .style(Style::default().fg(Color::White));

    f.render_widget(sparkline, inner);
}

fn render_weekly_comparison(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Top Keys Over Time ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let weekly_data = app.get_weekly_comparison();

    let header = Line::from(vec![
        Span::styled(
            format!("{:<8} {:>8} {:>8} {:>8} {:>8}  {:<10}",
                "Key", "Week 1", "Week 2", "Week 3", "Week 4", "Trend"),
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut items = vec![ListItem::new(header)];

    for (key_name, percentages, trend) in weekly_data.iter().take(8) {
        let trend_style = match trend.as_str() {
            t if t.starts_with('↗') => Style::default().fg(Color::Green),
            t if t.starts_with('↘') => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Gray),
        };

        let line = Line::from(vec![
            Span::styled(format!("{:<8}", key_name), Style::default().fg(Color::White)),
            Span::styled(
                format!(" {:>7.1}%", percentages.first().unwrap_or(&0.0)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!(" {:>7.1}%", percentages.get(1).unwrap_or(&0.0)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!(" {:>7.1}%", percentages.get(2).unwrap_or(&0.0)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!(" {:>7.1}%", percentages.get(3).unwrap_or(&0.0)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(format!("  {:<10}", trend), trend_style),
        ]);
        items.push(ListItem::new(line));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_app_distribution(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Per-App Distribution ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let apps = app.get_app_distribution();

    let items: Vec<ListItem> = apps
        .iter()
        .take(5)
        .map(|(name, pct)| {
            let bar_width = (pct / 2.0) as usize;
            let bar: String = "█".repeat(bar_width.min(30));
            
            let line = Line::from(vec![
                Span::styled(format!("{:<20}", truncate_app_name(name)), Style::default().fg(Color::White)),
                Span::styled(format!("{:>6.1}% ", pct), Style::default().fg(Color::Gray)),
                Span::styled(bar, Style::default().fg(Color::White)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn truncate_app_name(name: &str) -> String {
    let short = name
        .split('.')
        .last()
        .unwrap_or(name);
    
    if short.len() > 18 {
        format!("{}…", &short[..17])
    } else {
        short.to_string()
    }
}
