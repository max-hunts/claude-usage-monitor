mod api;
mod config;
mod models;
mod setup;
mod ui;

use anyhow::Result;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use ratatui::{init, restore, DefaultTerminal};

use api::ApiClient;
use config::Config;
use models::AggregatedUsage;
use setup::{SetupForm, SetupOutcome};

const REFRESH: std::time::Duration = std::time::Duration::from_secs(2);
const TICK: std::time::Duration = std::time::Duration::from_millis(200);

enum AppState {
    Setup { form: SetupForm },
    Running { api: ApiClient, usage: AggregatedUsage, last_error: Option<String> },
}

fn run_app(terminal: &mut DefaultTerminal) -> Result<()> {
    execute!(std::io::stdout(), EnableBracketedPaste).ok();

    let mut state = match Config::load() {
        Some(cfg) => AppState::Running {
            api: ApiClient::new(cfg)?,
            usage: AggregatedUsage::default(),
            last_error: None,
        },
        None => AppState::Setup { form: SetupForm::new(None) },
    };

    let mut last_fetch = std::time::Instant::now()
        .checked_sub(REFRESH * 2)
        .unwrap_or_else(std::time::Instant::now);

    loop {
        if let AppState::Running { api, usage, last_error } = &mut state {
            if last_fetch.elapsed() >= REFRESH {
                match api.fetch_usage() {
                    Ok(u) => {
                        *usage = u;
                        *last_error = None;
                    }
                    Err(e) => *last_error = Some(e.to_string()),
                }
                last_fetch = std::time::Instant::now();
            }
        }

        let last_updated = chrono::Local::now().format("%H:%M:%S").to_string();

        terminal.draw(|f| match &state {
            AppState::Setup { form } => setup::render(f, f.area(), form),
            AppState::Running { usage, last_error, .. } => {
                ui::render(f, f.area(), usage, &last_updated, last_error.as_deref());
            }
        })?;

        if event::poll(TICK)? {
            match event::read()? {
                Event::Key(key) => match &mut state {
                    AppState::Setup { form } => match form.handle_key(key) {
                        SetupOutcome::Cancel => return Ok(()),
                        SetupOutcome::Saved(cfg) => {
                            state = AppState::Running {
                                api: ApiClient::new(cfg)?,
                                usage: AggregatedUsage::default(),
                                last_error: None,
                            };
                            last_fetch = std::time::Instant::now()
                                .checked_sub(REFRESH * 2)
                                .unwrap_or_else(std::time::Instant::now);
                        }
                        SetupOutcome::Continue => {}
                    },
                    AppState::Running { .. } => {
                        if key.code == KeyCode::Char('q') {
                            break;
                        }
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            break;
                        }
                        if key.code == KeyCode::Char('e') {
                            let prefill = Config::load();
                            state = AppState::Setup {
                                form: SetupForm::new(prefill.as_ref()),
                            };
                        }
                    }
                },
                Event::Paste(text) => {
                    if let AppState::Setup { form } = &mut state {
                        form.handle_paste(text);
                    }
                }
                _ => {}
            }
        }
    }

    execute!(std::io::stdout(), DisableBracketedPaste).ok();
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--json") {
        return run_json();
    }

    let mut terminal = init();
    let result = run_app(&mut terminal);
    restore();
    result
}

fn run_json() -> Result<()> {
    let cfg = Config::load().ok_or_else(|| {
        anyhow::anyhow!("no credentials configured — run claude-usage-monitor first to set them up")
    })?;
    let body = api::fetch_raw_json(&cfg)?;
    print!("{}", body);
    Ok(())
}
