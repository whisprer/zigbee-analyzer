use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_help_overlay(f: &mut Frame, area: Rect) {
    // Create a centered rect
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1])[1];

    // Clear the area
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  1-7      : Switch tabs"),
        Line::from("  Tab      : Next tab"),
        Line::from("  ↑/↓      : Scroll up/down"),
        Line::from("  PgUp/PgDn: Page up/down"),
        Line::from("  Home     : Scroll to top"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Controls", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space    : Pause/Resume"),
        Line::from("  ?/F1     : Toggle this help"),
        Line::from("  Q/Ctrl+C : Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tabs", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Overview : Network summary"),
        Line::from("  Topology : Device topology"),
        Line::from("  Statistics: Traffic stats"),
        Line::from("  Channels : Channel analysis"),
        Line::from("  Anomalies: Detected anomalies"),
        Line::from("  Security : Security analysis"),
        Line::from("  Devices  : Device fingerprints"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ? or F1 to close", Style::default().fg(Color::Cyan)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .style(Style::default().bg(Color::Black))
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, popup_area);
}