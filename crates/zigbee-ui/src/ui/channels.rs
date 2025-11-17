use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_channel_list(f, app, chunks[1]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let best_channel = app.channels.get_best_channel().unwrap_or(11);
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Best Channel: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", best_channel)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Channels: ", Style::default().fg(Color::Cyan)),
            Span::raw("Scanning 11-26"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Analysis: ", Style::default().fg(Color::Yellow)),
            Span::raw("Utilization, Interference, Quality"),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Channel Analysis"));

    f.render_widget(paragraph, area);
}

fn draw_channel_list(f: &mut Frame, app: &App, area: Rect) {
    let mut rows = Vec::new();
    
    for channel in 11..=26 {
        if let Some(metrics) = app.channels.get_channel_metrics(channel) {
            let util_style = if metrics.utilization < 0.3 {
                Style::default().fg(Color::Green)
            } else if metrics.utilization < 0.7 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            rows.push(Row::new(vec![
                format!("{}", channel),
                format!("{:.1}%", metrics.utilization * 100.0),
                format!("{:.0}", metrics.avg_rssi),
                format!("{}", metrics.packet_count),
                format!("{}", 0)  // TODO: track errors,
            ])
            .style(util_style));
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(15),
        ],
    )
    .header(
        Row::new(vec!["Channel", "Utilization", "RSSI", "Packets", "Errors"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title("Channel Metrics"));

    f.render_widget(table, area);
}
