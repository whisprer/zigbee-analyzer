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

    draw_recommendation(f, app, chunks[0]);
    draw_channel_table(f, app, chunks[1]);
}

fn draw_recommendation(f: &mut Frame, app: &App, area: Rect) {
    let recommendation = app.channels.recommend_channel();
    
    let rec_text = vec![
        Line::from(vec![
            Span::styled("Recommended Channel: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{} (Score: {:.1}/100)", recommendation.recommended_channel, recommendation.quality_score)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Reason: ", Style::default().fg(Color::Cyan)),
            Span::raw(&recommendation.reason),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("WiFi Interference: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} channels affected", app.channels.detect_wifi_interference().len())),
        ]),
    ];

    let paragraph = Paragraph::new(rec_text)
        .block(Block::default().borders(Borders::ALL).title("Channel Recommendation"));

    f.render_widget(paragraph, area);
}

fn draw_channel_table(f: &mut Frame, app: &App, area: Rect) {
    let channels = app.channels.get_channels_by_quality();

    let rows: Vec<Row> = channels
        .iter()
        .map(|(ch, score)| {
            let metrics = app.channels.get_channel_metrics(*ch).unwrap();
            
            let quality_style = if *score >= 80.0 {
                Style::default().fg(Color::Green)
            } else if *score >= 60.0 {
                Style::default().fg(Color::Yellow)
            } else if *score >= 40.0 {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default().fg(Color::Red)
            };

            let interference_str = if metrics.interference_score > 0.3 {
                format!("{:?} ({:.2})", metrics.interference_type, metrics.interference_score)
            } else {
                "None".to_string()
            };

            Row::new(vec![
                format!("{}", ch),
                format!("{}", metrics.frequency_mhz),
                format!("{:.1}", score),
                format!("{}", metrics.packet_count),
                format!("{:.0}", metrics.avg_rssi),
                format!("{:.0}", metrics.avg_lqi),
                format!("{:.1}", metrics.utilization * 100.0),
                interference_str,
            ])
            .style(quality_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(20),
        ],
    )
    .header(
        Row::new(vec!["Ch", "Freq", "Score", "Packets", "RSSI", "LQI", "Util%", "Interference"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title("Channel Analysis"));

    f.render_widget(table, area);
}