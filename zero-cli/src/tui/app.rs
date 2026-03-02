use crate::tui::runtime::RuntimeEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessage {
    pub role: UiRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Sessions,
    Messages,
    Input,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub title: String,
    pub messages: Vec<UiMessage>,
    pub input: String,
    pub busy: bool,
    pub streaming: String,
    pub scroll: u16,
    pub auto_follow: bool,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub sessions: Vec<SessionState>,
    pub active_session: usize,
    pub focus: FocusArea,
    pub status: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub provider: String,
    pub model: String,
}

impl AppState {
    pub fn new_with_context(provider: String, model: String) -> Self {
        Self {
            sessions: vec![SessionState {
                title: "Session 1".to_string(),
                messages: Vec::new(),
                input: String::new(),
                busy: false,
                streaming: String::new(),
                scroll: 0,
                auto_follow: true,
            }],
            active_session: 0,
            focus: FocusArea::Input,
            status: "ready".to_string(),
            show_help: false,
            should_quit: false,
            provider,
            model,
        }
    }

    pub fn active_session_mut(&mut self) -> &mut SessionState {
        &mut self.sessions[self.active_session]
    }

    pub fn active_session(&self) -> &SessionState {
        &self.sessions[self.active_session]
    }

    pub fn add_session(&mut self) {
        let next = self.sessions.len() + 1;
        self.sessions.push(SessionState {
            title: format!("Session {}", next),
            messages: Vec::new(),
            input: String::new(),
            busy: false,
            streaming: String::new(),
            scroll: 0,
            auto_follow: true,
        });
        self.active_session = self.sessions.len() - 1;
    }

    pub fn prev_session(&mut self) {
        if self.active_session > 0 {
            self.active_session -= 1;
        }
    }

    pub fn next_session(&mut self) {
        if self.active_session + 1 < self.sessions.len() {
            self.active_session += 1;
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Sessions => FocusArea::Messages,
            FocusArea::Messages => FocusArea::Input,
            FocusArea::Input => FocusArea::Sessions,
        };
    }

    pub fn scroll_up(&mut self) {
        let session = self.active_session_mut();
        session.scroll = session.scroll.saturating_sub(1);
        session.auto_follow = false;
    }

    pub fn scroll_down(&mut self) {
        let session = self.active_session_mut();
        session.scroll = session.scroll.saturating_add(1);
        session.auto_follow = false;
    }

    pub fn page_up(&mut self) {
        let session = self.active_session_mut();
        session.scroll = session.scroll.saturating_sub(10);
        session.auto_follow = false;
    }

    pub fn page_down(&mut self) {
        let session = self.active_session_mut();
        session.scroll = session.scroll.saturating_add(10);
        session.auto_follow = false;
    }

    pub fn follow_bottom(&mut self) {
        let session = self.active_session_mut();
        session.auto_follow = true;
        session.scroll = u16::MAX;
    }

    pub fn start_submit(&mut self, prompt: String) {
        let session = self.active_session_mut();
        session.messages.push(UiMessage {
            role: UiRole::User,
            content: prompt,
        });
        session.input.clear();
        session.streaming.clear();
        session.busy = true;
        session.auto_follow = true;
        self.status = "thinking...".to_string();
    }

    pub fn apply_runtime_event(&mut self, event: RuntimeEvent) {
        match event {
            RuntimeEvent::TokenDelta {
                session_id,
                text: delta,
            } => {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    return;
                };
                session.streaming.push_str(&delta);
                if session.auto_follow {
                    session.scroll = session.scroll.saturating_add(1);
                }
            }
            RuntimeEvent::ToolEvent { session_id, name } => {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    return;
                };
                session.messages.push(UiMessage {
                    role: UiRole::System,
                    content: format!("[tool] {}", name),
                });
                if session.auto_follow {
                    session.scroll = session.scroll.saturating_add(1);
                }
            }
            RuntimeEvent::Done { session_id } => {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    return;
                };
                if !session.streaming.is_empty() {
                    let content = std::mem::take(&mut session.streaming);
                    session.messages.push(UiMessage {
                        role: UiRole::Assistant,
                        content,
                    });
                    if session.auto_follow {
                        session.scroll = session.scroll.saturating_add(1);
                    }
                }
                session.busy = false;
                self.status = "ready".to_string();
            }
            RuntimeEvent::Error {
                session_id,
                message: err,
            } => {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    return;
                };
                if !session.streaming.is_empty() {
                    let partial = std::mem::take(&mut session.streaming);
                    session.messages.push(UiMessage {
                        role: UiRole::Assistant,
                        content: format!("{} [interrupted]", partial),
                    });
                }
                session.messages.push(UiMessage {
                    role: UiRole::System,
                    content: format!("error: {}", err),
                });
                if session.auto_follow {
                    session.scroll = session.scroll.saturating_add(1);
                }
                session.busy = false;
                self.status = "error".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, FocusArea};
    use crate::tui::runtime::RuntimeEvent;

    #[test]
    fn session_creation_and_switching_works() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        assert_eq!(app.sessions.len(), 1);

        app.add_session();
        assert_eq!(app.sessions.len(), 2);
        assert_eq!(app.active_session, 1);

        app.prev_session();
        assert_eq!(app.active_session, 0);

        app.next_session();
        assert_eq!(app.active_session, 1);
    }

    #[test]
    fn focus_cycles() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        assert_eq!(app.focus, FocusArea::Input);
        app.cycle_focus();
        assert_eq!(app.focus, FocusArea::Sessions);
        app.cycle_focus();
        assert_eq!(app.focus, FocusArea::Messages);
        app.cycle_focus();
        assert_eq!(app.focus, FocusArea::Input);
    }

    #[test]
    fn scroll_saturates_at_zero() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        app.scroll_up();
        assert_eq!(app.active_session().scroll, 0);
        assert!(!app.active_session().auto_follow);
        app.scroll_down();
        assert_eq!(app.active_session().scroll, 1);
        app.page_down();
        assert_eq!(app.active_session().scroll, 11);
        app.page_up();
        assert_eq!(app.active_session().scroll, 1);
        app.follow_bottom();
        assert!(app.active_session().auto_follow);
    }

    #[test]
    fn runtime_event_applies_streaming_and_finalize() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        app.start_submit("hi".to_string());
        app.apply_runtime_event(RuntimeEvent::TokenDelta {
            session_id: 0,
            text: "hel".to_string(),
        });
        app.apply_runtime_event(RuntimeEvent::TokenDelta {
            session_id: 0,
            text: "lo".to_string(),
        });
        assert_eq!(app.active_session().streaming, "hello");

        app.apply_runtime_event(RuntimeEvent::Done { session_id: 0 });
        let session = app.active_session();
        assert!(!session.busy);
        assert!(session.streaming.is_empty());
        assert_eq!(
            session.messages.last().map(|m| m.content.as_str()),
            Some("hello")
        );
    }

    #[test]
    fn runtime_event_targets_correct_session() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        app.add_session();
        app.active_session = 0;
        app.start_submit("a".to_string());
        app.active_session = 1;
        app.start_submit("b".to_string());

        app.apply_runtime_event(RuntimeEvent::TokenDelta {
            session_id: 0,
            text: "s0".to_string(),
        });
        app.apply_runtime_event(RuntimeEvent::Done { session_id: 0 });
        app.apply_runtime_event(RuntimeEvent::TokenDelta {
            session_id: 1,
            text: "s1".to_string(),
        });
        app.apply_runtime_event(RuntimeEvent::Done { session_id: 1 });

        assert_eq!(
            app.sessions[0].messages.last().map(|m| m.content.as_str()),
            Some("s0")
        );
        assert_eq!(
            app.sessions[1].messages.last().map(|m| m.content.as_str()),
            Some("s1")
        );
    }

    #[test]
    fn submit_enables_auto_follow() {
        let mut app = AppState::new_with_context("p".to_string(), "m".to_string());
        app.scroll_down();
        assert!(!app.active_session().auto_follow);
        app.start_submit("hi".to_string());
        assert!(app.active_session().auto_follow);
    }
}
