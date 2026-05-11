use std::io;
use std::io::Stdout;
use std::time::Duration;

use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::DemoMode;
use crate::FocusTarget;
use crate::Overlay;
use crate::RedesignApp;
use crate::RedesignState;

pub fn run_terminal_preview(initial_mode: DemoMode) -> io::Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut state = RedesignState::demo(initial_mode);

    loop {
        session.terminal.draw(|frame| {
            frame.render_widget(RedesignApp::new(&state), frame.area());
        })?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
            && handle_key_event(&mut state, key_event)
        {
            break;
        }
    }

    Ok(())
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn should_exit(key_event: KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Esc)
        || matches!(key_event.code, KeyCode::Char('q') if key_event.modifiers.is_empty())
        || matches!(
            key_event,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL)
        )
}

fn handle_key_event(state: &mut RedesignState, key_event: KeyEvent) -> bool {
    if state.overlay != Overlay::None {
        return handle_overlay_key(state, key_event);
    }

    if should_exit(key_event)
        && (key_event.code == KeyCode::Esc || state.focus != FocusTarget::Composer)
    {
        return true;
    }

    match key_event {
        KeyEvent {
            code: KeyCode::Char('?'),
            ..
        } if state.focus != FocusTarget::Composer => state.overlay = Overlay::Help,
        KeyEvent {
            code: KeyCode::F(2),
            ..
        } => state.open_commands(),
        KeyEvent {
            code: KeyCode::Char('p'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => state.open_commands(),
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => state.overlay = Overlay::History,
        KeyEvent {
            code: KeyCode::Char('t'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => state.overlay = Overlay::Transcript,
        KeyEvent {
            code: KeyCode::Char('1'),
            ..
        } if state.focus != FocusTarget::Composer => *state = RedesignState::demo(DemoMode::Idle),
        KeyEvent {
            code: KeyCode::Char('2'),
            ..
        } if state.focus != FocusTarget::Composer => {
            *state = RedesignState::demo(DemoMode::Running)
        }
        KeyEvent {
            code: KeyCode::Char('3'),
            ..
        } if state.focus != FocusTarget::Composer => {
            *state = RedesignState::demo(DemoMode::Approval)
        }
        KeyEvent {
            code: KeyCode::Tab, ..
        } => state.focus_next(),
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => state.focus_previous(),
        KeyEvent {
            code: KeyCode::Left,
            ..
        } if state.focus == FocusTarget::Approval => state.select_previous_approval_action(),
        KeyEvent {
            code: KeyCode::Right,
            ..
        } if state.focus == FocusTarget::Approval => state.select_next_approval_action(),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } if state.focus == FocusTarget::Approval => state.apply_selected_approval_action(),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } if state.focus == FocusTarget::Composer => state.submit_composer(),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } if state.focus == FocusTarget::Composer => {
            state.composer.draft.pop();
        }
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers,
            ..
        } if state.focus == FocusTarget::Composer && accepts_text_input(modifiers) => {
            state.composer.draft.push(character);
        }
        _ => {}
    }
    false
}

fn handle_overlay_key(state: &mut RedesignState, key_event: KeyEvent) -> bool {
    if state.overlay == Overlay::Commands {
        return handle_commands_overlay_key(state, key_event);
    }

    match key_event {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?'),
            ..
        } => state.overlay = Overlay::None,
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => return true,
        _ => {}
    }
    false
}

fn handle_commands_overlay_key(state: &mut RedesignState, key_event: KeyEvent) -> bool {
    match key_event {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Char('q'),
            ..
        } => state.overlay = Overlay::None,
        KeyEvent {
            code: KeyCode::Char('?'),
            ..
        } => state.overlay = Overlay::Help,
        KeyEvent {
            code: KeyCode::Up | KeyCode::BackTab,
            ..
        } => state.select_previous_command(),
        KeyEvent {
            code: KeyCode::Down | KeyCode::Tab,
            ..
        } => state.select_next_command(),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => state.apply_selected_command(),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => state.pop_command_query_char(),
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => state.command_query.clear(),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers,
            ..
        } if accepts_text_input(modifiers) => state.push_command_query_char(character),
        _ => {}
    }
    false
}

fn accepts_text_input(modifiers: KeyModifiers) -> bool {
    modifiers.is_empty() || modifiers == KeyModifiers::SHIFT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    #[test]
    fn question_mark_in_composer_is_text() {
        let mut state = RedesignState::demo(DemoMode::Idle);

        let should_exit = handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );

        assert!(!should_exit);
        assert_eq!(state.overlay, Overlay::None);
        assert_eq!(state.composer.draft, "?");
    }

    #[test]
    fn question_mark_outside_composer_opens_help() {
        let mut state = RedesignState::demo(DemoMode::Approval);

        let should_exit = handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );

        assert!(!should_exit);
        assert_eq!(state.overlay, Overlay::Help);
        assert_eq!(state.composer.draft, "");
    }

    #[test]
    fn f2_opens_commands_overlay() {
        let mut state = RedesignState::demo(DemoMode::Idle);

        let should_exit =
            handle_key_event(&mut state, KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));

        assert!(!should_exit);
        assert_eq!(state.overlay, Overlay::Commands);
    }

    #[test]
    fn command_palette_accepts_filter_text() {
        let mut state = RedesignState::demo(DemoMode::Idle);
        state.open_commands();

        for character in ['a', 'p', 'p'] {
            let should_exit = handle_key_event(
                &mut state,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
            assert!(!should_exit);
        }

        assert_eq!(state.command_query, "app");
        assert_eq!(state.command_choice, crate::CommandChoice::SimulateApproval);
    }

    #[test]
    fn command_palette_runs_filtered_command() {
        let mut state = RedesignState::demo(DemoMode::Idle);
        state.open_commands();
        state.push_command_query_char('a');
        state.push_command_query_char('p');
        state.push_command_query_char('p');

        let should_exit = handle_key_event(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(!should_exit);
        assert!(state.approval.is_some());
        assert_eq!(state.overlay, Overlay::None);
    }
}
