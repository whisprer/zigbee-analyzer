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
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_incidents(f, app, chunks[1]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.security.get_statistics();
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Security Incidents: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", stats.total_incidents)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Critical: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{:3}  ", stats.critical_incidents)),
            Span::styled("High: ", Style::default().fg(Color::LightRed)),
            Span::raw(format!("{:3}  ", stats.high_incidents)),
            Span::styled("Medium: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:3}  ", stats.medium_incidents)),
            Span::styled("Low: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:3}", stats.low_incidents)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Networks: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", stats.networks_analyzed)),
            Span::styled("Devices: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} (Secured: {}, Unsecured: {})", 
                stats.devices_analyzed, stats.secured_devices, stats.unsecured_devices)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Avg Encryption Rate: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:.1}%  ", stats.avg_encryption_rate * 100.0)),
            Span::styled("Avg Trust Score: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{:.2}/1.0", stats.avg_trust_score)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Join Attempts: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{} (Success: {}, Failed: {})", 
                stats.join_attempts, stats.successful_joins, stats.failed_joins)),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Security Analysis"));

    f.render_widget(paragraph, area);
}

fn draw_incidents(f: &mut Frame, app: &App, area: Rect) {
    let incidents = app.security.get_incidents();

    let rows: Vec<Row> = incidents
        .iter()
        .rev()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(4) as usize)
        .map(|i| {
            let severity_style = match i.severity {
                zigbee_analysis::IncidentSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                zigbee_analysis::IncidentSeverity::High => Style::default().fg(Color::LightRed),
                zigbee_analysis::IncidentSeverity::Medium => Style::default().fg(Color::Yellow),
                zigbee_analysis::IncidentSeverity::Low => Style::default().fg(Color::Green),
                zigbee_analysis::IncidentSeverity::Info => Style::default().fg(Color::Cyan),
            };

            let device_str = i.affected_device
                .map(|d| format!("{}", d))
                .unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                format!("{:?}", i.severity),
                format!("{:?}", i.incident_type),
                i.description.chars().take(45).collect::<String>(),
                device_str,
            ])
            .style(severity_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(47),
            Constraint::Length(20),
        ],
    )
    .header(
        Row::new(vec!["Severity", "Type", "Description", "Device"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title(format!("Security Incidents (scroll: {})", app.scroll_offset)));

    f.render_widget(table, area);
}