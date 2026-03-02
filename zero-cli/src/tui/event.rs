use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Quit,
    Submit,
    Newline,
    ToggleHelp,
    NewSession,
    NextSession,
    PrevSession,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    FollowBottom,
    CycleFocus,
    InputChar(char),
    Backspace,
    Noop,
}

pub fn map_key(key: KeyEvent) -> Command {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Command::Quit;
    }

    match key.code {
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Tab => Command::CycleFocus,
        KeyCode::Char('?') => Command::ToggleHelp,
        KeyCode::Char('n') => Command::NewSession,
        KeyCode::Char('h') | KeyCode::Left => Command::PrevSession,
        KeyCode::Char('l') | KeyCode::Right => Command::NextSession,
        KeyCode::Char('j') | KeyCode::Down => Command::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Command::ScrollUp,
        KeyCode::PageUp => Command::PageUp,
        KeyCode::PageDown => Command::PageDown,
        KeyCode::End => Command::FollowBottom,
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                Command::Newline
            } else {
                Command::Submit
            }
        }
        KeyCode::Backspace => Command::Backspace,
        KeyCode::Char(c) => Command::InputChar(c),
        _ => Command::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, map_key};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn key_translation() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Command::Quit
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            Command::CycleFocus
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            Command::PrevSession
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            Command::NextSession
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Command::ScrollDown
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Command::ScrollUp
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            Command::PageUp
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            Command::PageDown
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
            Command::FollowBottom
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Command::Submit
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)),
            Command::Newline
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Command::Quit
        );
    }
}
