use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_anomaly_list(f, app, chunks[1]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.anomalies.get_statistics();
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total Anomalies: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", stats.total_anomalies)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Critical: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{:4}  ", stats.critical_incidents)),
            Span::styled("High: ", Style::default().fg(Color::LightRed)),
            Span::raw(format!("{:4}  ", stats.high_incidents)),
            Span::styled("Medium: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:4}  ", stats.medium_incidents)),
            Span::styled("Low: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:4}", stats.low_incidents)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packets Processed: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", stats.packets_processed)),
            Span::styled("Anomaly Rate: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.4}%", stats.anomaly_rate * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Detection: ", Style::default().fg(Color::Magenta)),
            Span::raw("Traffic, Security, Behavior, Protocol"),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Anomaly Detection Summary"));

    f.render_widget(paragraph, area);
}

fn draw_anomaly_list(f: &mut Frame, app: &App, area: Rect) {
    let anomalies = app.anomalies.get_anomalies();

    let rows: Vec<Row> = anomalies
        .iter()
        .rev()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(4) as usize)
        .map(|a| {
            let severity_style = match a.severity {
                zigbee_analysis::AnomalySeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                zigbee_analysis::AnomalySeverity::High => Style::default().fg(Color::LightRed),
                zigbee_analysis::AnomalySeverity::Medium => Style::default().fg(Color::Yellow),
                zigbee_analysis::AnomalySeverity::Low => Style::default().fg(Color::Green),
            };

            let device_str = a.affected_device
                .map(|d| format!("{}", d))
                .unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                format!("{:?}", a.severity),
                format!("{:?}", a.anomaly_type),
                a.description.chars().take(40).collect::<String>(),
                device_str,
                format!("{:.0}%", a.confidence * 100.0),
            ])
            .style(severity_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(42),
            Constraint::Length(20),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Severity", "Type", "Description", "Device", "Conf%"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title(format!("Detected Anomalies (scroll: {})", app.scroll_offset)));

    f.render_widget(table, area);
}