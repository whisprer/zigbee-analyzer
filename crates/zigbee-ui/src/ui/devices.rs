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
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_device_list(f, app, chunks[1]);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.devices.get_statistics();
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total Devices: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", stats.total_devices)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Identified: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{:4} ({:.1}%)  ", 
                stats.identified_devices,
                stats.identified_devices as f32 / stats.total_devices.max(1) as f32 * 100.0
            )),
            Span::styled("Unidentified: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:4} ({:.1}%)",
                stats.unidentified_devices,
                stats.unidentified_devices as f32 / stats.total_devices.max(1) as f32 * 100.0
            )),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Avg Confidence: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{:.1}%", stats.avg_confidence * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Manufacturers: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}", stats.by_manufacturer.len())),
        ]),
    ];

    let paragraph = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Device Fingerprinting"));

    f.render_widget(paragraph, area);
}

fn draw_device_list(f: &mut Frame, app: &App, area: Rect) {
    let mut devices: Vec<_> = app.devices.get_all_fingerprints().values().collect();
    devices.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let rows: Vec<Row> = devices
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(4) as usize)
        .map(|d| {
            let confidence_style = if d.confidence >= 0.8 {
                Style::default().fg(Color::Green)
            } else if d.confidence >= 0.5 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            let manufacturer = d.manufacturer.as_deref().unwrap_or("Unknown");
            let model = d.model.as_deref().unwrap_or("-");
            
            let type_short = match d.device_type {
                zigbee_analysis::DeviceType::Coordinator => "Coord",
                zigbee_analysis::DeviceType::Router => "Router",
                zigbee_analysis::DeviceType::EndDevice => "EndDev",
                zigbee_analysis::DeviceType::OnOffLight => "Light",
                zigbee_analysis::DeviceType::DimmableLight => "DimLight",
                zigbee_analysis::DeviceType::ColorLight => "ColorLight",
                zigbee_analysis::DeviceType::TemperatureSensor => "TempSens",
                zigbee_analysis::DeviceType::MotionSensor => "MotionSens",
                zigbee_analysis::DeviceType::ContactSensor => "ContactSens",
                zigbee_analysis::DeviceType::SmartPlug => "Plug",
                zigbee_analysis::DeviceType::Thermostat => "Thermo",
                _ => "Unknown",
            };

            Row::new(vec![
                format!("{}", d.mac_addr),
                type_short.to_string(),
                manufacturer.chars().take(12).collect::<String>(),
                model.chars().take(15).collect::<String>(),
                format!("{:.0}%", d.confidence * 100.0),
                format!("{}", d.packet_count),
                format!("{:.1}%", d.protocol.encryption_rate * 100.0),
            ])
            .style(confidence_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(13),
            Constraint::Length(16),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["MAC Address", "Type", "Manufacturer", "Model", "Conf%", "Packets", "Enc%"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).title(format!("Device Fingerprints (scroll: {})", app.scroll_offset)));

    f.render_widget(table, area);
}