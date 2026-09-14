mod api;
mod config;
mod models;
mod monitor;
mod setup;
mod ui;

use anyhow::Result;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use ratatui::{init, restore, DefaultTerminal};

use config::Config;
use monitor::{Monitor, Snapshot};
use setup::{SetupForm, SetupOutcome};

const TICK: std::time::Duration = std::time::Duration::from_millis(200);

enum AppState {
    Setup {
        form: SetupForm,
    },
    Running {
        monitor: Monitor,
        snapshot: Snapshot,
    },
}

fn run_app(terminal: &mut DefaultTerminal) -> Result<()> {
    execute!(std::io::stdout(), EnableBracketedPaste).ok();

    let mut state = match Config::load() {
        Some(cfg) => AppState::Running {
            monitor: Monitor::start(cfg, true),
            snapshot: Snapshot::default(),
        },
        None => AppState::Setup {
            form: SetupForm::new(None),
        },
    };

    loop {
        if let AppState::Running { monitor, snapshot } = &mut state {
            for update in monitor.updates.try_iter() {
                snapshot.apply(update);
            }
        }

        terminal.draw(|f| match &state {
            AppState::Setup { form } => setup::render(f, f.area(), form),
            AppState::Running { snapshot, .. } => {
                let updated = snapshot
                    .claude_updated_at
                    .as_deref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|t| {
                        t.with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_else(|| "waiting for usage".into());
                ui::render(
                    f,
                    f.area(),
                    &snapshot.aggregated(),
                    &updated,
                    snapshot.claude_error.as_deref(),
                );
            }
        })?;

        if event::poll(TICK)? {
            match event::read()? {
                Event::Key(key) => match &mut state {
                    AppState::Setup { form } => match form.handle_key(key) {
                        SetupOutcome::Cancel => return Ok(()),
                        SetupOutcome::Saved(cfg) => {
                            state = AppState::Running {
                                monitor: Monitor::start(cfg, true),
                                snapshot: Snapshot::default(),
                            };
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
    let monitor = Monitor::start(cfg, false);
    let mut snapshot = Snapshot::default();
    for update in &monitor.updates {
        snapshot.apply(update);
    }
    println!("{}", snapshot.json());
    Ok(())
}
