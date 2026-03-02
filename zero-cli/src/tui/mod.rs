pub mod app;
pub mod event;
pub mod runtime;
pub mod ui;

use anyhow::Result;
use crossterm::event::{Event, poll, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::tui::app::AppState;
use crate::tui::event::{Command, map_key};
use crate::tui::runtime::RuntimeEvent;

pub async fn run_tui<F>(provider: String, model: String, mut on_submit: F) -> Result<()>
where
    F: FnMut(usize, String, mpsc::UnboundedSender<RuntimeEvent>) -> Result<()>,
{
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = async {
        let mut app = AppState::new_with_context(provider, model);
        let (tx, mut rx) = mpsc::unbounded_channel::<RuntimeEvent>();

        loop {
            while let Ok(event) = rx.try_recv() {
                app.apply_runtime_event(event);
            }

            terminal.draw(|f| ui::draw(f, &app))?;

            if app.should_quit {
                break;
            }

            if poll(Duration::from_millis(30))?
                && let Event::Key(key) = read()?
            {
                match map_key(key) {
                    Command::Quit => app.should_quit = true,
                    Command::CycleFocus => app.cycle_focus(),
                    Command::ToggleHelp => app.show_help = !app.show_help,
                    Command::NewSession => app.add_session(),
                    Command::PrevSession => app.prev_session(),
                    Command::NextSession => app.next_session(),
                    Command::ScrollUp => app.scroll_up(),
                    Command::ScrollDown => app.scroll_down(),
                    Command::PageUp => app.page_up(),
                    Command::PageDown => app.page_down(),
                    Command::FollowBottom => app.follow_bottom(),
                    Command::Backspace => {
                        app.active_session_mut().input.pop();
                    }
                    Command::InputChar(c) => {
                        app.active_session_mut().input.push(c);
                    }
                    Command::Newline => {
                        app.active_session_mut().input.push('\n');
                    }
                    Command::Submit => {
                        if app.active_session().busy {
                            continue;
                        }
                        let prompt = app.active_session().input.trim().to_string();
                        if prompt.is_empty() {
                            continue;
                        }

                        let session_id = app.active_session;
                        app.start_submit(prompt.clone());
                        if let Err(e) = on_submit(session_id, prompt, tx.clone()) {
                            app.apply_runtime_event(RuntimeEvent::Error {
                                session_id,
                                message: e.to_string(),
                            });
                        }
                    }
                    Command::Noop => {}
                }
            }
        }

        Result::<()>::Ok(())
    }
    .await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
