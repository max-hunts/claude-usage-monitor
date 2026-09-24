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

const FIELDS: [&str; 7] = [
    "Claude Org ID",
    "Claude sessionKey",
    "Claude cf_clearance",
    "Claude __cf_bm (optional)",
    "Codex Authorization (paste Bearer + token)",
    "Codex account ID",
    "Codex Cookie header (optional)",
];
const NUM_FIELDS: usize = FIELDS.len();

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
                c.codex_access_token.clone().unwrap_or_default(),
                c.codex_account_id.clone().unwrap_or_default(),
                c.codex_cookie.clone().unwrap_or_default(),
            ]
        } else {
            Default::default()
        };
        SetupForm {
            values,
            focus: 0,
            status: None,
        }
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
            KeyCode::F(2) => self.clear_field(),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.clear_field()
            }
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

    fn clear_field(&mut self) -> SetupOutcome {
        self.values[self.focus].clear();
        self.status = None;
        SetupOutcome::Continue
    }

    pub fn handle_paste(&mut self, text: String) -> SetupOutcome {
        let cleaned: String = text.chars().filter(|c| !c.is_control()).collect();
        self.values[self.focus].push_str(&cleaned);
        self.status = None;
        SetupOutcome::Continue
    }

    fn try_save(&mut self) -> SetupOutcome {
        for (i, label) in FIELDS.iter().enumerate().take(3) {
            if self.values[i].trim().is_empty() {
                self.focus = i;
                self.status = Some((format!("{} is required", label), true));
                return SetupOutcome::Continue;
            }
        }
        if !self.values[4].trim().is_empty() && self.values[5].trim().is_empty() {
            self.focus = 5;
            self.status = Some((
                "Codex account ID is required with an access token".into(),
                true,
            ));
            return SetupOutcome::Continue;
        }
        let optional = |i: usize| {
            let value = self.values[i].trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        };
        let cf_bm = self.values[3].trim().to_string();
        let cfg = Config {
            org_id: self.values[0].trim().to_string(),
            session_key: self.values[1].trim().to_string(),
            cf_clearance: self.values[2].trim().to_string(),
            cf_bm: if cf_bm.is_empty() { None } else { Some(cf_bm) },
            codex_access_token: optional(4)
                .map(|v| v.strip_prefix("Bearer ").unwrap_or(&v).to_owned()),
            codex_account_id: optional(5),
            codex_cookie: optional(6),
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
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // title
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Claude + Codex Usage — Setup",
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
            "Codex (optional): copy the full Authorization header, including Bearer, and account ID.",
            Style::default().fg(MUTED),
        )),
        Line::from(Span::styled(
            "Tab/Shift-Tab: switch fields  ·  Ctrl+U: clear field  ·  Enter: save  ·  Esc: cancel",
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
    let visible = (chunks[4].height / 3).max(1) as usize;
    let first = form.focus.saturating_sub(visible - 1);
    let count = visible.min(NUM_FIELDS - first);
    let field_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); count])
        .split(chunks[4]);

    for (row, i) in (first..first + count).enumerate() {
        render_field(
            f,
            field_chunks[row],
            FIELDS[i],
            &form.values[i],
            i == form.focus,
            matches!(i, 1 | 2 | 3 | 4 | 6),
        );
    }

    // status
    if let Some((msg, is_error)) = &form.status {
        let color = if *is_error { DANGER } else { SUCCESS };
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                msg.clone(),
                Style::default().fg(color),
            )))
            .alignment(Alignment::Center)
            .style(Style::default().bg(BG)),
            chunks[6],
        );
    }

    // footer
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "F2 / Ctrl+U: clear field  ·  Tab: next  ·  Enter: save",
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
        "•".repeat(value.chars().count().min(24))
    } else {
        value.to_string()
    };

    let cursor = if focused { "▏" } else { "" };
    let border_color = if focused { ACCENT } else { MUTED };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).bg(BG))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default().fg(if focused { FG } else { MUTED }),
        ));

    if focused {
        block = block.title_bottom(Line::from(Span::styled(
            " F2 / Ctrl+U: clear field ",
            Style::default().fg(ACCENT),
        )));
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_w = inner.width.saturating_sub(1) as usize;
    let shown: String = display
        .chars()
        .rev()
        .take(max_w)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(shown, Style::default().fg(FG).bg(BG)),
            Span::styled(cursor, Style::default().fg(ACCENT).bg(BG)),
        ]))
        .style(Style::default().bg(BG)),
        inner,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn codex_fields_scroll_into_view_and_mask_secrets() {
        let cfg = Config {
            codex_access_token: Some("secret-access-token".into()),
            codex_cookie: Some("sëcret-cookie".into()),
            ..Default::default()
        };
        let mut form = SetupForm::new(Some(&cfg));
        for focus in [4, 6] {
            form.focus = focus;
            let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
            terminal.draw(|f| render(f, f.area(), &form)).unwrap();
            let text = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(text.contains(FIELDS[focus]));
            assert!(text.contains("F2 / Ctrl+U: clear field"));
            assert!(!text.contains("secret-access-token"));
            assert!(!text.contains("sëcret-cookie"));
        }
        form.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert!(form.values[6].is_empty());
    }
    #[test]
    fn clear_shortcuts_only_clear_focused_field_and_allow_replacement() {
        for key in [
            KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        ] {
            let mut form = SetupForm::new(None);
            form.focus = 4;
            form.values[4] = "old-token".repeat(1000);
            form.values[5] = "keep-account".into();
            form.status = Some(("old error".into(), true));
            form.handle_key(key);
            assert!(form.values[4].is_empty());
            assert_eq!(form.values[5], "keep-account");
            assert!(form.status.is_none());
            form.handle_paste("replacement-token".into());
            assert_eq!(form.values[4], "replacement-token");
        }
    }
}
