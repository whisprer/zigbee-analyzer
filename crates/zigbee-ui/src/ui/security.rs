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
            Span::raw(format!("{}", stats.total_threats)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Critical: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{:3}  ", stats.critical_threats)),
            Span::styled("High: ", Style::default().fg(Color::LightRed)),
            Span::raw(format!("{:3}  ", stats.high_threats)),
            Span::styled("Medium: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:3}  ", stats.medium_threats)),
            Span::styled("Low: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:3}", stats.low_threats)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Networks: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", 0)),  // TODO: track networks
            Span::styled("Devices: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} (Secured: {}, Unsecured: {})", 
                stats.encrypted_devices + stats.unencrypted_devices, stats.encrypted_devices, stats.unencrypted_devices)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Avg Encryption Rate: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:.1}%  ", 0.0)),  // TODO: calculate encryption rate
            Span::styled("Avg Trust Score: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{:.2}/1.0", 0.5)),  // TODO: calculate trust score
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Join Attempts: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{} (Success: {}, Failed: {})", 0, 0, 0)),  // TODO: track joins
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Security Analysis"));

    f.render_widget(paragraph, area);
}

fn draw_incidents(f: &mut Frame, app: &App, area: Rect) {
    let incidents = &app.security.incidents;

    let rows: Vec<Row> = incidents
        .iter()
        .rev()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(4) as usize)
        .map(|i| {
            // TODO: SecurityIncident needs a severity field
            let severity_style = match zigbee_analysis::ThreatSeverity::Medium {
                zigbee_analysis::ThreatSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                zigbee_analysis::ThreatSeverity::High => Style::default().fg(Color::LightRed),
                zigbee_analysis::ThreatSeverity::Medium => Style::default().fg(Color::Yellow),
                zigbee_analysis::ThreatSeverity::Low => Style::default().fg(Color::Green),
            };

            // TODO: SecurityIncident needs an affected_device field
            let device_str = "N/A".to_string();

            Row::new(vec![
                format!("Medium"),  // TODO: add severity field to SecurityIncident
                format!("{}", i.incident_type),
                i.details.chars().take(45).collect::<String>(),
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