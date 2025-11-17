use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Summary
            Constraint::Length(10), // Gauges
            Constraint::Min(0),     // Recent activity
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_gauges(f, app, chunks[1]);
    draw_recent_activity(f, app, chunks[2]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.statistics.get_summary();
    let topo_stats = app.topology.get_statistics();
    let anomaly_stats = app.anomalies.get_statistics();
    let security_stats = app.security.get_statistics();

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Devices: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", topo_stats.total_devices)),
            Span::raw("  "),
            Span::styled("Networks: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", topo_stats.total_networks)),
            Span::raw("  "),
            Span::styled("Links: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", topo_stats.total_links)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packets: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} ({:.1} KB)", stats.total_packets, stats.total_bytes as f32 / 1024.0)),
            Span::raw("  "),
            Span::styled("Rate: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:.1} pps", stats.packets_per_second)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Anomalies: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ", anomaly_stats.total_anomalies)),
            Span::styled("(Critical: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{})", anomaly_stats.critical)),
            Span::raw("  "),
            Span::styled("Security: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{} incidents", security_stats.total_threats)),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Network Overview"));

    f.render_widget(paragraph, area);
}

fn draw_gauges(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Channel utilization
    if let Some(best_channel) = app.channels.get_best_channel() {
        if let Some(metrics) = app.channels.get_channel_metrics(best_channel) {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(format!("Channel {} Utilization", best_channel)))
                .gauge_style(Style::default().fg(Color::Green))
                .percent((metrics.utilization * 100.0) as u16);
            f.render_widget(gauge, chunks[0]);
        }
    }

    // Encryption rate
    let _security_stats = app.security.get_statistics();
    let enc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Network Encryption Rate"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(0);  // TODO: calculate actual encryption rate
    f.render_widget(enc_gauge, chunks[1]);

    // Trust score
    let trust_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Average Device Trust Score"))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent(50);  // TODO: calculate actual trust score
    f.render_widget(trust_gauge, chunks[2]);
}

fn draw_recent_activity(f: &mut Frame, app: &App, area: Rect) {
    let recent_anomalies = app.anomalies.get_recent_anomalies(10);
    
    let items: Vec<ListItem> = recent_anomalies
        .iter()
        .map(|a| {
            let style = match a.severity {
                zigbee_analysis::AnomalySeverity::Critical => Style::default().fg(Color::Red),
                zigbee_analysis::AnomalySeverity::High => Style::default().fg(Color::LightRed),
                zigbee_analysis::AnomalySeverity::Medium => Style::default().fg(Color::Yellow),
                zigbee_analysis::AnomalySeverity::Low => Style::default().fg(Color::Green),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{:?}] ", a.severity), style.add_modifier(Modifier::BOLD)),
                Span::raw(&a.description),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Recent Anomalies"));

    f.render_widget(list, area);
}
