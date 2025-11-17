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
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    draw_devices(f, app, chunks[0]);
    draw_links(f, app, chunks[1]);
}

fn draw_devices(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Summary
    let stats = app.topology.get_statistics();
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", stats.total_devices)),
            Span::styled("Coordinators: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} ", stats.coordinators)),
            Span::styled("Routers: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ", stats.routers)),
            Span::styled("End Devices: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}", stats.end_devices)),
        ]),
    ];

    let summary = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Device Summary"));
    f.render_widget(summary, chunks[0]);

    // Device list
    let devices: Vec<_> = app.topology.devices.values().collect();
    // TODO: Add packet_count to Device for sorting

    let rows: Vec<Row> = devices
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(6) as usize)
        .map(|d| {
            // Style based on whether device is coordinator/router
            let type_style = if d.is_coordinator {
                Style::default().fg(Color::Green)
            } else if d.is_router {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Blue)
            };

            Row::new(vec![
                format!("{}", d.address),
                d.short_addr.map(|a| format!("0x{:04x}", a)).unwrap_or_else(|| "N/A".to_string()),
                format!("{:?}", d.device_type),
                format!("{}", 0),      // TODO: track packet_count in Device
                format!("{:.0}", 0.0), // TODO: track avg_rssi in Device
                format!("{:.0}", 0.0), // TODO: track avg_lqi in Device
            ])
            .style(type_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["MAC Address", "NWK Addr", "Type", "Packets", "RSSI", "LQI"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title("Devices"));

    f.render_widget(table, chunks[1]);
}

fn draw_links(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Summary
    let stats = app.topology.get_statistics();
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total Links: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", stats.total_links)),
            Span::styled("Avg Quality: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:.1}", 0.0)),  // TODO: calculate avg link quality
        ]),
    ];

    let summary = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Link Summary"));
    f.render_widget(summary, chunks[0]);

    // Links list
    let mut links: Vec<_> = app.topology.links.iter().collect();
    links.sort_by(|a, b| b.packet_count.cmp(&a.packet_count));

    let rows: Vec<Row> = links
        .iter()
        .take(area.height.saturating_sub(6) as usize)
        .map(|l| {
            let quality_style = if l.link_quality as f32 > 200.0 {
                Style::default().fg(Color::Green)
            } else if l.link_quality as f32 > 150.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            Row::new(vec![
                format!("{}", l.source),
                format!("{}", l.destination),
                format!("{}", l.packet_count),
                format!("{:.0}", l.rssi as f32),
                format!("{:.0}", l.link_quality as f32),
            ])
            .style(quality_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Source", "Destination", "Packets", "RSSI", "LQI"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title("Communication Links"));

    f.render_widget(table, chunks[1]);
}