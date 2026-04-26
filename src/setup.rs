use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::config::Config;

const BG: Color = Color::Rgb(22, 27, 34);
const FG: Color = Color::Rgb(230, 237, 243);
const ACCENT: Color = Color::Rgb(88, 166, 255);
const MUTED: Color = Color::Rgb(139, 148, 158);
const DANGER: Color = Color::Rgb(248, 81, 73);
const SUCCESS: Color = Color::Rgb(63, 185, 80);

const FIELDS: [&str; 4] = ["Org ID", "sessionKey", "cf_clearance", "__cf_bm (optional)"];
const NUM_FIELDS: usize = 4;

pub struct SetupForm {
    pub values: [String; NUM_FIELDS],
    pub focus: usize,
    pub status: Option<(String, bool)>, // (message, is_error)
}

pub enum SetupOutcome {
    Continue,
    Saved(Config),
    Cancel,
}

impl SetupForm {
    pub fn new(prefill: Option<&Config>) -> Self {
        let values = if let Some(c) = prefill {
            [
                c.org_id.clone(),
                c.session_key.clone(),
                c.cf_clearance.clone(),
                c.cf_bm.clone().unwrap_or_default(),
            ]
        } else {
            Default::default()
        };
        SetupForm { values, focus: 0, status: None }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SetupOutcome {
        match key.code {
            KeyCode::Esc => SetupOutcome::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                SetupOutcome::Cancel
            }
            KeyCode::Tab | KeyCode::Down => {
                self.focus = (self.focus + 1) % NUM_FIELDS;
                SetupOutcome::Continue
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focus = (self.focus + NUM_FIELDS - 1) % NUM_FIELDS;
                SetupOutcome::Continue
            }
            KeyCode::Enter => self.try_save(),
            KeyCode::Backspace => {
                self.values[self.focus].pop();
                self.status = None;
                SetupOutcome::Continue
            }
            KeyCode::Char(c) => {
                self.values[self.focus].push(c);
                self.status = None;
                SetupOutcome::Continue
            }
            _ => SetupOutcome::Continue,
        }
    }

    pub fn handle_paste(&mut self, text: String) -> SetupOutcome {
        let cleaned: String = text.chars().filter(|c| !c.is_control()).collect();
        self.values[self.focus].push_str(&cleaned);
        self.status = None;
        SetupOutcome::Continue
    }

    fn try_save(&mut self) -> SetupOutcome {
        for i in 0..3 {
            if self.values[i].trim().is_empty() {
                self.focus = i;
                self.status = Some((format!("{} is required", FIELDS[i]), true));
                return SetupOutcome::Continue;
            }
        }
        let cf_bm = self.values[3].trim().to_string();
        let cfg = Config {
            org_id: self.values[0].trim().to_string(),
            session_key: self.values[1].trim().to_string(),
            cf_clearance: self.values[2].trim().to_string(),
            cf_bm: if cf_bm.is_empty() { None } else { Some(cf_bm) },
        };
        match cfg.save() {
            Ok(path) => {
                self.status = Some((format!("saved to {}", path.display()), false));
                SetupOutcome::Saved(cfg)
            }
            Err(e) => {
                self.status = Some((format!("save failed: {}", e), true));
                SetupOutcome::Continue
            }
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, form: &SetupForm) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(3 * NUM_FIELDS as u16),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // title
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Claude Usage Monitor — Setup",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG)),
        chunks[1],
    );

    // help
    let help = vec![
        Line::from(Span::styled(
            "Open https://claude.ai/settings/usage in Chrome while signed in.",
            Style::default().fg(MUTED),
        )),
        Line::from(Span::styled(
            "DevTools (⌥⌘I) → Application → Cookies → https://claude.ai",
            Style::default().fg(MUTED),
        )),
        Line::from(Span::styled(
            "Copy sessionKey, cf_clearance, __cf_bm. Org ID is in the URL when viewing usage.",
            Style::default().fg(MUTED),
        )),
        Line::from(Span::styled(
            "Tab/Shift-Tab to switch fields  ·  Enter to save  ·  Esc to quit",
            Style::default().fg(MUTED),
        )),
    ];
    f.render_widget(
        Paragraph::new(help)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(BG)),
        chunks[2],
    );

    // fields
    let field_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3); NUM_FIELDS])
        .split(chunks[4]);

    for i in 0..NUM_FIELDS {
        render_field(
            f,
            field_chunks[i],
            FIELDS[i],
            &form.values[i],
            i == form.focus,
            i == 1, // mask sessionKey
        );
    }

    // status
    if let Some((msg, is_error)) = &form.status {
        let color = if *is_error { DANGER } else { SUCCESS };
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(msg.clone(), Style::default().fg(color))))
                .alignment(Alignment::Center)
                .style(Style::default().bg(BG)),
            chunks[6],
        );
    }

    // footer
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "stored in ~/.config/claude-usage-monitor/config.toml (chmod 600)",
            Style::default().fg(MUTED),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG)),
        chunks[7],
    );
}

fn render_field(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool, mask: bool) {
    let centered = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(80),
            Constraint::Min(0),
        ])
        .split(area);
    let area = centered[1];

    let display = if mask && !value.is_empty() {
        if value.len() <= 12 {
            "•".repeat(value.len())
        } else {
            format!("{}…{}", &value[..6], "•".repeat(8))
        }
    } else {
        value.to_string()
    };

    let cursor = if focused { "▏" } else { "" };
    let border_color = if focused { ACCENT } else { MUTED };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).bg(BG))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default().fg(if focused { FG } else { MUTED }),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_w = inner.width.saturating_sub(1) as usize;
    let shown: String = display.chars().rev().take(max_w).collect::<String>().chars().rev().collect();

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(shown, Style::default().fg(FG).bg(BG)),
            Span::styled(cursor, Style::default().fg(ACCENT).bg(BG)),
        ]))
        .style(Style::default().bg(BG)),
        inner,
    );
}
