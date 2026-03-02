use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, layout::Rect};

use crate::tui::app::{AppState, FocusArea, UiRole};

pub fn draw(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    let busy = if app.active_session().busy {
        "busy"
    } else {
        "idle"
    };
    let follow = if app.active_session().auto_follow {
        "follow"
    } else {
        "manual"
    };
    let status_text = format!(
        "{} | provider={} model={} | session={}/{} | {} {}",
        app.status,
        app.provider,
        app.model,
        app.active_session + 1,
        app.sessions.len(),
        busy,
        follow
    );
    let status =
        Paragraph::new(status_text).style(Style::default().fg(Color::Black).bg(Color::Green));
    frame.render_widget(status, vertical[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(1)])
        .split(vertical[1]);

    let sessions: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let marker = if idx == app.active_session { ">" } else { " " };
            ListItem::new(format!("{} {}", marker, s.title))
        })
        .collect();

    let session_border = if app.focus == FocusArea::Sessions {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    frame.render_widget(
        List::new(sessions).block(
            Block::default()
                .title("Sessions")
                .borders(Borders::ALL)
                .border_style(session_border),
        ),
        body[0],
    );

    let mut lines = Vec::new();
    for m in &app.active_session().messages {
        let (tag, color) = match m.role {
            UiRole::User => ("You", Color::Blue),
            UiRole::Assistant => ("AI", Color::Cyan),
            UiRole::System => ("Sys", Color::Yellow),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("[{}] ", tag), Style::default().fg(color)),
            Span::raw(m.content.clone()),
        ]));
    }

    if !app.active_session().streaming.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "[AI*] ",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.active_session().streaming.clone()),
        ]));
    }

    let chat_border = if app.focus == FocusArea::Messages {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Chat")
                    .borders(Borders::ALL)
                    .border_style(chat_border),
            )
            .wrap(Wrap { trim: false })
            .scroll((app.active_session().scroll, 0)),
        body[1],
    );

    let input_border = if app.focus == FocusArea::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(app.active_session().input.as_str())
            .block(
                Block::default()
                    .title("Input")
                    .borders(Borders::ALL)
                    .border_style(input_border),
            )
            .wrap(Wrap { trim: false }),
        vertical[2],
    );

    if app.show_help {
        let popup = centered_rect(72, 50, area);
        frame.render_widget(Clear, popup);
        let help = Paragraph::new(
            "Enter send | Shift+Enter newline | Tab focus | h/l switch session | j/k or Up/Down scroll | PgUp/PgDn page scroll | End follow bottom | n new session | ? help | q quit",
        )
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
        frame.render_widget(help, popup);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::tui::app::AppState;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn render_smoke() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let app = AppState::new_with_context("p".to_string(), "m".to_string());
        terminal.draw(|f| draw(f, &app)).expect("draw");
    }
}
