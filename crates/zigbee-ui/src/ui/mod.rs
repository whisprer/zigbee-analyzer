mod overview;  // Fixed from overrview.rs typo
mod topology;
mod statistics;
mod channels;  // For channel analysis
mod anomalies;  // Fixed from channel.rs
mod security;
mod devices;
mod help;


use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header/tabs
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(f.size());

    // Draw tabs
    draw_tabs(f, app, chunks[0]);

    // Draw content based on current tab
    match app.current_tab {
        Tab::Overview => overview::draw(f, app, chunks[1]),
        Tab::Topology => topology::draw(f, app, chunks[1]),
        Tab::Statistics => statistics::draw(f, app, chunks[1]),
        Tab::Channels => channels::draw(f, app, chunks[1]),
        Tab::Anomalies => anomalies::draw(f, app, chunks[1]),
        Tab::Security => security::draw(f, app, chunks[1]),
        Tab::Devices => devices::draw(f, app, chunks[1]),
    }

    // Draw status bar
    draw_status_bar(f, app, chunks[2]);

    // Draw help overlay if visible
    if app.help_visible {
        help::draw_help_overlay(f, f.size());
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        "Overview [1]",
        "Topology [2]",
        "Statistics [3]",
        "Channels [4]",
        "Anomalies [5]",
        "Security [6]",
        "Devices [7]",
    ];
    
    let selected = match app.current_tab {
        Tab::Overview => 0,
        Tab::Topology => 1,
        Tab::Statistics => 2,
        Tab::Channels => 3,
        Tab::Anomalies => 4,
        Tab::Security => 5,
        Tab::Devices => 6,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("🔌 Zigbee Analyzer"))
        .select(selected)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue)
        );

    f.render_widget(tabs, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let uptime = app.uptime();
    let status_text = if app.paused {
        format!(
            " ⏸ PAUSED | Packets: {} | Errors: {} | Uptime: {}s | [SPACE] Resume | [?] Help | [Q] Quit",
            app.packet_count,
            app.error_count,
            uptime.as_secs()
        )
    } else {
        format!(
            " ▶ LIVE | Packets: {} | Errors: {} | PPS: {:.1} | Uptime: {}s | [SPACE] Pause | [?] Help | [Q] Quit",
            app.packet_count,
            app.error_count,
            app.statistics.packets_per_second(),
            uptime.as_secs()
        )
    };

    let status_style = if app.paused {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green).bg(Color::DarkGray)
    };

    let status = Line::from(vec![Span::styled(status_text, status_style)]);

    f.render_widget(status, area);
}
