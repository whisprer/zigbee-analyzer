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
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_frame_types(f, app, chunks[1]);
    draw_top_devices(f, app, chunks[2]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.statistics.get_summary();
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total Packets: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", stats.total_packets)),
            Span::styled("Total Bytes: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.2} MB", stats.total_bytes as f32 / 1_048_576.0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Avg Packet Size: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:.1} bytes", stats.avg_packet_size)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Rate: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.1} pps, {:.1} bps", stats.packets_per_second, stats.bytes_per_second)),
        ]),
        Line::from(vec![
            Span::styled("Peak Rate: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{:.1} pps, {:.1} bps", stats.peak_pps, stats.peak_bps)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Unique Devices: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{} ", stats.unique_devices)),
            Span::styled("Active Channels: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}", stats.active_channels)),
        ]),
        Line::from(vec![
            Span::styled("Parse Errors: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{} ", stats.parse_errors)),
            Span::styled("FCS Errors: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", stats.fcs_errors)),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Traffic Statistics"));

    f.render_widget(paragraph, area);
}

fn draw_frame_types(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.statistics.get_summary();
    
    let total = stats.beacon_frames + stats.data_frames + stats.ack_frames + stats.command_frames;
    
    let items = if total > 0 {
        vec![
            ListItem::new(Line::from(vec![
                Span::styled("Beacon:  ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{:6} ({:5.1}%)", stats.beacon_frames, stats.beacon_frames as f32 / total as f32 * 100.0)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Data:    ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:6} ({:5.1}%)", stats.data_frames, stats.data_frames as f32 / total as f32 * 100.0)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("ACK:     ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:6} ({:5.1}%)", stats.ack_frames, stats.ack_frames as f32 / total as f32 * 100.0)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:6} ({:5.1}%)", stats.command_frames, stats.command_frames as f32 / total as f32 * 100.0)),
            ])),
        ]
    } else {
        vec![ListItem::new("No data yet...")]
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Frame Type Distribution"));

    f.render_widget(list, area);
}

fn draw_top_devices(f: &mut Frame, app: &App, area: Rect) {
    let top_devices = app.statistics.get_top_devices(10);

    let rows: Vec<Row> = top_devices
        .iter()
        .enumerate()
        .map(|(i, (addr, stats))| {
            Row::new(vec![
                format!("{}", i + 1),
                format!("{}", addr),
                format!("{}", stats.tx_packets),
                format!("{}", stats.rx_packets),
                format!("{:.0}", stats.avg_rssi),
                format!("{:.0}", stats.avg_lqi),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["#", "Device", "TX Packets", "RX Packets", "RSSI", "LQI"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title("Top 10 Devices by Traffic"));

    f.render_widget(table, area);
}