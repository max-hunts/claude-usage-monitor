use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::models::AggregatedUsage;

const BG: Color = Color::Rgb(22, 27, 34);
const FG: Color = Color::Rgb(230, 237, 243);
const ACCENT: Color = Color::Rgb(88, 166, 255);
const SUCCESS: Color = Color::Rgb(63, 185, 80);
const ORANGE: Color = Color::Rgb(255, 136, 0);
const DANGER: Color = Color::Rgb(248, 81, 73);
const MUTED: Color = Color::Rgb(139, 148, 158);
const BAR_EMPTY: Color = Color::Rgb(48, 54, 61);

pub fn render(
    f: &mut Frame,
    area: Rect,
    usage: &AggregatedUsage,
    last_updated: &str,
    error: Option<&str>,
) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // top flex
            Constraint::Length(3), // 5h
            Constraint::Length(1), // spacer
            Constraint::Length(3), // 7d
            Constraint::Length(1), // spacer
            Constraint::Length(3), // extra credits
            Constraint::Min(0),    // bottom flex
            Constraint::Length(1), // footer
        ])
        .split(area);

    render_header(f, chunks[0]);
    render_window(
        f,
        chunks[2],
        "5h Window",
        usage.five_hour_util,
        usage.five_hour_resets_at.as_deref(),
    );
    render_window(
        f,
        chunks[4],
        "7d Window",
        usage.seven_day_util,
        usage.seven_day_resets_at.as_deref(),
    );
    render_spend(f, chunks[6], usage);
    render_footer(f, chunks[8], last_updated, error);
}

fn render_header(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "Claude Usage Monitor",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default().bg(BG)),
        Span::styled("●", Style::default().fg(SUCCESS)),
        Span::styled(" live   ", Style::default().fg(MUTED)),
        Span::styled("claude.ai", Style::default().fg(ACCENT)),
        Span::styled("   ·   e: edit creds  q: quit", Style::default().fg(MUTED)),
    ]);
    let p = Paragraph::new(line)
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG));
    f.render_widget(p, area);
}

fn render_spend(f: &mut Frame, area: Rect, usage: &AggregatedUsage) {
    let label = if usage.extra_enabled {
        let used = usage.extra_used / 100.0;
        let limit = usage.extra_limit / 100.0;
        let symbol = currency_symbol(&usage.currency);
        Line::from(vec![
            Span::styled("Extra Credits   ", Style::default().fg(MUTED)),
            Span::styled(
                format!(
                    "{}{:.2} / {}{:.2}   ({:.1}%)",
                    symbol, used, symbol, limit, usage.extra_util
                ),
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Extra Credits   ", Style::default().fg(MUTED)),
            Span::styled("not enabled", Style::default().fg(MUTED)),
        ])
    };

    render_section(
        f,
        area,
        label,
        usage.extra_util,
        usage.extra_enabled,
    );
}

fn render_window(f: &mut Frame, area: Rect, name: &str, util: f64, resets_at: Option<&str>) {
    let label = Line::from(vec![
        Span::styled(format!("{}   ", name), Style::default().fg(MUTED)),
        Span::styled(
            format!("{:.0}%", util),
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   ·   resets in {}", format_resets_in(resets_at)),
            Style::default().fg(MUTED),
        ),
    ]);

    render_section(f, area, label, util, true);
}

fn render_section(f: &mut Frame, area: Rect, label: Line<'_>, util: f64, draw_bar: bool) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let p = Paragraph::new(label)
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG));
    f.render_widget(p, rows[0]);

    if draw_bar {
        render_bar(f, rows[1], (util / 100.0).clamp(0.0, 1.0), bar_color(util));
    } else {
        f.render_widget(
            Block::default().style(Style::default().bg(BG)),
            rows[1],
        );
    }
}

fn render_bar(f: &mut Frame, area: Rect, ratio: f64, fill: Color) {
    let total_w = area.width as usize;
    if total_w == 0 {
        return;
    }
    let bar_w = total_w.saturating_sub(8).min(120).max(10);
    let bar_w = bar_w.min(total_w);
    let pad = (total_w - bar_w) / 2;

    let filled = ((bar_w as f64) * ratio).round() as usize;
    let filled = filled.min(bar_w);
    let empty = bar_w - filled;

    let mut spans = Vec::with_capacity(4);
    if pad > 0 {
        spans.push(Span::styled(" ".repeat(pad), Style::default().bg(BG)));
    }
    spans.push(Span::styled(
        "━".repeat(filled),
        Style::default().fg(fill).bg(BG),
    ));
    spans.push(Span::styled(
        "━".repeat(empty),
        Style::default().fg(BAR_EMPTY).bg(BG),
    ));
    let trailing = total_w - pad - bar_w;
    if trailing > 0 {
        spans.push(Span::styled(" ".repeat(trailing), Style::default().bg(BG)));
    }

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(BG));
    f.render_widget(p, area);
}

fn bar_color(util: f64) -> Color {
    if util >= 100.0 {
        DANGER
    } else {
        ORANGE
    }
}

fn currency_symbol(code: &str) -> &'static str {
    match code {
        "GBP" => "£",
        "USD" => "$",
        "EUR" => "€",
        _ => "",
    }
}

fn format_resets_in(resets_at: Option<&str>) -> String {
    let Some(s) = resets_at else {
        return "—".to_string();
    };
    let Ok(dt) = DateTime::parse_from_rfc3339(s) else {
        return "—".to_string();
    };
    let delta = dt.with_timezone(&Utc) - Utc::now();
    let secs = delta.num_seconds();
    if secs <= 0 {
        return "now".to_string();
    }
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins.max(1))
    }
}

fn render_footer(f: &mut Frame, area: Rect, last_updated: &str, error: Option<&str>) {
    let content = if let Some(err) = error {
        Line::from(vec![
            Span::styled("Error: ", Style::default().fg(DANGER)),
            Span::styled(err, Style::default().fg(DANGER)),
            Span::styled("   ·   ", Style::default().fg(MUTED)),
            Span::styled("q: quit", Style::default().fg(MUTED)),
        ])
    } else {
        Line::from(vec![
            Span::styled("updated ", Style::default().fg(MUTED)),
            Span::styled(last_updated, Style::default().fg(FG)),
            Span::styled("   ·   ", Style::default().fg(MUTED)),
            Span::styled("q: quit", Style::default().fg(MUTED)),
        ])
    };

    let p = Paragraph::new(content)
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG));
    f.render_widget(p, area);
}
