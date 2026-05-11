//! Keyboard input, external editor, and status-line dispatch for the TUI app.
//!
//! This module owns global key bindings that sit above ChatWidget, including transcript overlay
//! entry, Ctrl-L clear, external editor launch, and agent navigation shortcuts.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RedesignShortcutAction {
    None,
    Redraw,
    ClearTerminal,
    OpenTranscript,
    OpenExternalEditor,
    OpenChat(ThreadId),
    StartNewChat,
}

const REDESIGN_LINE_SCROLL: usize = 3;

pub(super) fn redesign_global_quit_key_matches(event: &TuiEvent) -> bool {
    matches!(
        event,
        TuiEvent::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) if modifiers.contains(KeyModifiers::CONTROL)
            && !modifiers.contains(KeyModifiers::ALT)
            && c.eq_ignore_ascii_case(&'c')
    )
}

impl App {
    pub(super) async fn launch_external_editor(&mut self, tui: &mut tui::Tui) {
        let editor_cmd = match external_editor::resolve_editor_command() {
            Ok(cmd) => cmd,
            Err(external_editor::EditorError::MissingEditor) => {
                self.chat_widget
                    .add_to_history(history_cell::new_error_event(
                    "Cannot open external editor: set $VISUAL or $EDITOR before starting Codex."
                        .to_string(),
                ));
                self.reset_external_editor_state(tui);
                return;
            }
            Err(err) => {
                self.chat_widget
                    .add_to_history(history_cell::new_error_event(format!(
                        "Failed to open editor: {err}",
                    )));
                self.reset_external_editor_state(tui);
                return;
            }
        };

        let seed = self.chat_widget.composer_text_with_pending();
        let editor_result = tui
            .with_restored(tui::RestoreMode::KeepRaw, || async {
                external_editor::run_editor(&seed, &editor_cmd).await
            })
            .await;
        self.reset_external_editor_state(tui);

        match editor_result {
            Ok(new_text) => {
                // Trim trailing whitespace
                let cleaned = new_text.trim_end().to_string();
                self.chat_widget.apply_external_edit(cleaned);
            }
            Err(err) => {
                self.chat_widget
                    .add_to_history(history_cell::new_error_event(format!(
                        "Failed to open editor: {err}",
                    )));
            }
        }
        tui.frame_requester().schedule_frame();
    }

    pub(super) fn request_external_editor_launch(&mut self, tui: &mut tui::Tui) {
        self.chat_widget
            .set_external_editor_state(ExternalEditorState::Requested);
        self.chat_widget.set_footer_hint_override(Some(vec![(
            EXTERNAL_EDITOR_HINT.to_string(),
            String::new(),
        )]));
        tui.frame_requester().schedule_frame();
    }

    pub(super) fn reset_external_editor_state(&mut self, tui: &mut tui::Tui) {
        self.chat_widget
            .set_external_editor_state(ExternalEditorState::Closed);
        self.chat_widget.set_footer_hint_override(/*items*/ None);
        tui.frame_requester().schedule_frame();
    }

    pub(super) fn apply_raw_output_mode(
        &mut self,
        tui: &mut tui::Tui,
        enabled: bool,
        notify: bool,
    ) {
        if notify {
            self.chat_widget.set_raw_output_mode_and_notify(enabled);
        } else {
            self.chat_widget.set_raw_output_mode(enabled);
        }
        if let Err(err) = self.reflow_transcript_now(tui) {
            tracing::warn!(error = %err, "failed to reflow transcript after raw output mode toggle");
            self.chat_widget
                .add_error_message(format!("Failed to redraw transcript: {err}"));
        }
        tui.frame_requester().schedule_frame();
    }

    pub(super) async fn handle_key_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        key_event: KeyEvent,
    ) {
        // Some terminals, especially on macOS, encode Option+Left/Right as Option+b/f unless
        // enhanced keyboard reporting is available. We only treat those word-motion fallbacks as
        // agent-switch shortcuts when the composer is empty so we never steal the expected
        // editing behavior for moving across words inside a draft.
        let allow_agent_word_motion_fallback = !self.enhanced_keys_supported
            && self.chat_widget.composer_text_with_pending().is_empty();
        if self.overlay.is_none()
            && self.chat_widget.no_modal_or_popup_active()
            // Alt+Left/Right are also natural word-motion keys in the composer. Keep agent
            // fast-switch available only once the draft is empty so editing behavior wins whenever
            // there is text on screen.
            && self.chat_widget.composer_text_with_pending().is_empty()
            && previous_agent_shortcut_matches(key_event, allow_agent_word_motion_fallback)
        {
            if let Some(thread_id) = self
                .adjacent_thread_id_with_backfill(app_server, AgentNavigationDirection::Previous)
                .await
            {
                let _ = self
                    .select_agent_thread_and_discard_side(tui, app_server, thread_id)
                    .await;
            }
            return;
        }
        if self.overlay.is_none()
            && self.chat_widget.no_modal_or_popup_active()
            // Mirror the previous-agent rule above: empty drafts may use these keys for thread
            // switching, but non-empty drafts keep them for expected word-wise cursor motion.
            && self.chat_widget.composer_text_with_pending().is_empty()
            && next_agent_shortcut_matches(key_event, allow_agent_word_motion_fallback)
        {
            if let Some(thread_id) = self
                .adjacent_thread_id_with_backfill(app_server, AgentNavigationDirection::Next)
                .await
            {
                let _ = self
                    .select_agent_thread_and_discard_side(tui, app_server, thread_id)
                    .await;
            }
            return;
        }
        if side_return_shortcut_matches(key_event)
            && self.maybe_return_from_side(tui, app_server).await
        {
            return;
        }

        match self.handle_redesign_shortcut_key(key_event, tui.terminal.viewport_area) {
            RedesignShortcutAction::None => {}
            RedesignShortcutAction::Redraw => {
                tui.frame_requester().schedule_frame();
                return;
            }
            RedesignShortcutAction::ClearTerminal => {
                if let Err(err) = self.clear_terminal_ui(tui, /*redraw_header*/ false) {
                    tracing::warn!(error = %err, "failed to clear terminal UI");
                    self.chat_widget
                        .add_error_message(format!("Failed to clear terminal UI: {err}"));
                } else {
                    self.reset_app_ui_state_after_clear();
                    tui.frame_requester().schedule_frame();
                }
                return;
            }
            RedesignShortcutAction::OpenTranscript => {
                let _ = tui.enter_alt_screen();
                self.overlay = Some(Overlay::new_transcript(
                    self.transcript_cells.clone(),
                    self.keymap.pager.clone(),
                ));
                tui.frame_requester().schedule_frame();
                return;
            }
            RedesignShortcutAction::OpenExternalEditor => {
                if self.overlay.is_none()
                    && self.chat_widget.can_launch_external_editor()
                    && self.chat_widget.external_editor_state() == ExternalEditorState::Closed
                {
                    self.request_external_editor_launch(tui);
                }
                return;
            }
            RedesignShortcutAction::OpenChat(thread_id) => {
                self.redesign_sidebar_state.blur();
                if let Err(err) = self
                    .select_agent_thread_and_discard_side(tui, app_server, thread_id)
                    .await
                {
                    self.chat_widget
                        .add_error_message(format!("Failed to switch to chat {thread_id}: {err}"));
                }
                tui.frame_requester().schedule_frame();
                return;
            }
            RedesignShortcutAction::StartNewChat => {
                self.redesign_sidebar_state.blur();
                if let Err(err) = self.start_redesign_chat(tui, app_server).await {
                    self.chat_widget
                        .add_error_message(format!("Failed to start a new chat: {err}"));
                }
                tui.frame_requester().schedule_frame();
                return;
            }
        }

        let app_keymap_shortcuts_available = self.app_keymap_shortcuts_available();

        if app_keymap_shortcuts_available && self.keymap.app.toggle_vim_mode.is_pressed(key_event) {
            self.chat_widget.toggle_vim_mode_and_notify();
            return;
        }

        if app_keymap_shortcuts_available
            && self.keymap.app.toggle_fast_mode.is_pressed(key_event)
            && self.chat_widget.can_toggle_fast_mode_from_keybinding()
        {
            self.chat_widget.toggle_fast_mode_from_ui();
            return;
        }

        if app_keymap_shortcuts_available && self.keymap.app.toggle_raw_output.is_pressed(key_event)
        {
            let enabled = !self.chat_widget.raw_output_mode();
            self.apply_raw_output_mode(tui, enabled, /*notify*/ false);
            return;
        }

        if app_keymap_shortcuts_available && self.keymap.app.open_transcript.is_pressed(key_event) {
            // Enter alternate screen and set viewport to full size.
            let _ = tui.enter_alt_screen();
            self.overlay = Some(Overlay::new_transcript(
                self.transcript_cells.clone(),
                self.keymap.pager.clone(),
            ));
            tui.frame_requester().schedule_frame();
            return;
        }

        if app_keymap_shortcuts_available
            && self.keymap.app.open_external_editor.is_pressed(key_event)
        {
            // Only launch the external editor if there is no overlay and the bottom pane is not in use.
            // Note that it can be launched while a task is running to enable editing while the previous turn is ongoing.
            if self.overlay.is_none()
                && self.chat_widget.can_launch_external_editor()
                && self.chat_widget.external_editor_state() == ExternalEditorState::Closed
            {
                self.request_external_editor_launch(tui);
            }
            return;
        }

        if matches!(key_event.code, KeyCode::Esc)
            && matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        {
            // Esc primes/advances legacy backtracking only in normal (not working) mode
            // with the composer focused and empty. The redesigned UI keeps transcript
            // entry explicit via Ctrl-T/sidebar action. In any other state, forward
            // Esc so the active UI (e.g. status indicator, modals, popups) handles it.
            if self.should_handle_backtrack_esc(key_event) {
                self.handle_backtrack_esc_key(tui);
            } else {
                self.chat_widget.handle_key_event(key_event);
            }
            return;
        }

        match key_event {
            _ if app_keymap_shortcuts_available
                && self.keymap.app.clear_terminal.is_pressed(key_event) =>
            {
                if !self.chat_widget.can_run_ctrl_l_clear_now() {
                    return;
                }
                if let Err(err) = self.clear_terminal_ui(tui, /*redraw_header*/ false) {
                    tracing::warn!(error = %err, "failed to clear terminal UI");
                    self.chat_widget
                        .add_error_message(format!("Failed to clear terminal UI: {err}"));
                } else {
                    self.reset_app_ui_state_after_clear();
                    if !self.redesign_chrome_enabled {
                        self.queue_clear_ui_header(tui);
                    }
                    tui.frame_requester().schedule_frame();
                }
            }
            // Enter confirms backtrack when primed + count > 0. Otherwise pass to widget.
            KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            } if self.backtrack.primed
                && self.backtrack.nth_user_message != usize::MAX
                && self.chat_widget.composer_is_empty() =>
            {
                if let Some(selection) = self.confirm_backtrack_from_main() {
                    self.apply_backtrack_selection(tui, selection);
                }
            }
            KeyEvent {
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                // Any non-Esc key press should cancel a primed backtrack.
                // This avoids stale "Esc-primed" state after the user starts typing
                // (even if they later backspace to empty).
                if key_event.code != KeyCode::Esc && self.backtrack.primed {
                    self.reset_backtrack_state();
                }
                self.chat_widget.handle_key_event(key_event);
            }
            _ => {
                self.chat_widget.handle_key_event(key_event);
            }
        };
    }

    pub(super) fn should_handle_backtrack_esc(&self, key_event: KeyEvent) -> bool {
        !self.redesign_chrome_enabled
            && self.chat_widget.is_normal_backtrack_mode()
            && self.chat_widget.composer_is_empty()
            && !self.chat_widget.should_handle_vim_insert_escape(key_event)
    }

    fn handle_redesign_shortcut_key(
        &mut self,
        key_event: KeyEvent,
        viewport_area: ratatui::layout::Rect,
    ) -> RedesignShortcutAction {
        if !self.redesign_chrome_enabled
            || !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            || !self.app_keymap_shortcuts_available()
        {
            return RedesignShortcutAction::None;
        }

        let chat_count = self.redesign_chat_entries().len();

        if redesign_sidebar_toggle_key_matches(key_event) {
            self.redesign_sidebar_state.toggle_focus(chat_count);
            return RedesignShortcutAction::Redraw;
        }

        if self.redesign_sidebar_state.focused() {
            return self.handle_redesign_sidebar_key(key_event, chat_count);
        }

        if let Some(action) = self.handle_redesign_transcript_scroll_key(key_event, viewport_area) {
            return action;
        }

        match key_event {
            KeyEvent {
                code: KeyCode::Char('?'),
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.chat_widget.insert_str("?");
                RedesignShortcutAction::Redraw
            }
            KeyEvent {
                code: KeyCode::F(1),
                kind: KeyEventKind::Press,
                ..
            } => {
                self.redesign_sidebar_state.blur();
                self.chat_widget.open_keymap_picker();
                RedesignShortcutAction::Redraw
            }
            KeyEvent {
                code: KeyCode::F(2),
                kind: KeyEventKind::Press,
                ..
            } if self.chat_widget.composer_text_with_pending().is_empty() => {
                self.redesign_sidebar_state.blur();
                self.chat_widget.insert_str("/");
                RedesignShortcutAction::Redraw
            }
            KeyEvent {
                code: KeyCode::F(3),
                kind: KeyEventKind::Press,
                ..
            } if self.chat_widget.can_run_ctrl_l_clear_now() => {
                self.redesign_sidebar_state.blur();
                RedesignShortcutAction::ClearTerminal
            }
            KeyEvent {
                code: KeyCode::F(4),
                kind: KeyEventKind::Press,
                ..
            } => {
                self.redesign_sidebar_state.blur();
                self.chat_widget.open_model_popup();
                RedesignShortcutAction::Redraw
            }
            _ => RedesignShortcutAction::None,
        }
    }

    fn handle_redesign_transcript_scroll_key(
        &mut self,
        key_event: KeyEvent,
        viewport_area: ratatui::layout::Rect,
    ) -> Option<RedesignShortcutAction> {
        if !self.chat_widget.composer_text_with_pending().is_empty() {
            return None;
        }

        let scroll_limit = redesign_chrome::transcript_scroll_limit(viewport_area, self);
        let page_scroll = viewport_area
            .height
            .saturating_sub(6)
            .max(REDESIGN_LINE_SCROLL as u16) as usize;
        match key_event {
            KeyEvent {
                code: KeyCode::Up,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = self
                    .redesign_transcript_scroll
                    .saturating_add(REDESIGN_LINE_SCROLL)
                    .min(scroll_limit);
                Some(RedesignShortcutAction::Redraw)
            }
            KeyEvent {
                code: KeyCode::Down,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = self
                    .redesign_transcript_scroll
                    .saturating_sub(REDESIGN_LINE_SCROLL)
                    .min(scroll_limit);
                Some(RedesignShortcutAction::Redraw)
            }
            KeyEvent {
                code: KeyCode::PageUp,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = self
                    .redesign_transcript_scroll
                    .saturating_add(page_scroll)
                    .min(scroll_limit);
                Some(RedesignShortcutAction::Redraw)
            }
            KeyEvent {
                code: KeyCode::PageDown,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = self
                    .redesign_transcript_scroll
                    .saturating_sub(page_scroll)
                    .min(scroll_limit);
                Some(RedesignShortcutAction::Redraw)
            }
            KeyEvent {
                code: KeyCode::Home,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = scroll_limit;
                Some(RedesignShortcutAction::Redraw)
            }
            KeyEvent {
                code: KeyCode::End,
                modifiers,
                ..
            } if !crate::key_hint::has_ctrl_or_alt(modifiers) => {
                self.redesign_transcript_scroll = 0;
                Some(RedesignShortcutAction::Redraw)
            }
            _ => None,
        }
    }

    fn handle_redesign_sidebar_key(
        &mut self,
        key_event: KeyEvent,
        chat_count: usize,
    ) -> RedesignShortcutAction {
        if redesign_sidebar_global_key_should_pass_through(key_event) {
            return RedesignShortcutAction::None;
        }

        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.redesign_sidebar_state.blur();
                RedesignShortcutAction::Redraw
            }
            KeyEvent {
                code: KeyCode::Char('n' | 'N'),
                modifiers,
                ..
            } if modifiers == KeyModifiers::NONE => {
                self.redesign_sidebar_state.blur();
                RedesignShortcutAction::StartNewChat
            }
            KeyEvent {
                code: KeyCode::Char('f' | 'F'),
                modifiers,
                ..
            } if modifiers == KeyModifiers::NONE => {
                self.redesign_final_only_transcript = !self.redesign_final_only_transcript;
                RedesignShortcutAction::Redraw
            }
            _ if redesign_sidebar_previous_key_matches(key_event) => {
                self.redesign_sidebar_state.select_previous(chat_count);
                RedesignShortcutAction::Redraw
            }
            _ if redesign_sidebar_next_key_matches(key_event) => {
                self.redesign_sidebar_state.select_next(chat_count);
                RedesignShortcutAction::Redraw
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => self.activate_redesign_sidebar_item(),
            _ => RedesignShortcutAction::Redraw,
        }
    }

    fn activate_redesign_sidebar_item(&mut self) -> RedesignShortcutAction {
        let selected = self.redesign_sidebar_state.selected();
        self.redesign_sidebar_state.blur();

        match selected {
            redesign_chrome::RedesignSidebarSelection::Chat(idx) => self
                .redesign_chat_entries()
                .get(idx)
                .map(|entry| RedesignShortcutAction::OpenChat(entry.thread_id))
                .unwrap_or(RedesignShortcutAction::Redraw),
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::NewChat,
            ) => RedesignShortcutAction::StartNewChat,
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::FinalOnly,
            ) => {
                self.redesign_final_only_transcript = !self.redesign_final_only_transcript;
                RedesignShortcutAction::Redraw
            }
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::Commands,
            ) => {
                if self.chat_widget.composer_text_with_pending().is_empty() {
                    self.chat_widget.insert_str("/");
                }
                RedesignShortcutAction::Redraw
            }
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::Models,
            ) => {
                self.chat_widget.open_model_popup();
                RedesignShortcutAction::Redraw
            }
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::History,
            ) => {
                self.chat_widget
                    .handle_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
                RedesignShortcutAction::Redraw
            }
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::Transcript,
            ) => RedesignShortcutAction::OpenTranscript,
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::Editor,
            ) => RedesignShortcutAction::OpenExternalEditor,
        }
    }

    fn app_keymap_shortcuts_available(&self) -> bool {
        self.overlay.is_none() && self.chat_widget.no_modal_or_popup_active()
    }

    pub(super) fn refresh_status_line(&mut self) {
        self.chat_widget.refresh_status_line();
    }
}

fn redesign_sidebar_toggle_key_matches(key_event: KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('b' | 'B'))
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
        && !key_event.modifiers.contains(KeyModifiers::ALT)
}

fn redesign_sidebar_previous_key_matches(key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Up => !crate::key_hint::has_ctrl_or_alt(key_event.modifiers),
        KeyCode::Char('k') => key_event.modifiers == KeyModifiers::NONE,
        _ => false,
    }
}

fn redesign_sidebar_next_key_matches(key_event: KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Down => !crate::key_hint::has_ctrl_or_alt(key_event.modifiers),
        KeyCode::Char('j') => key_event.modifiers == KeyModifiers::NONE,
        _ => false,
    }
}

fn redesign_sidebar_global_key_should_pass_through(key_event: KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('c' | 'C' | 'd' | 'D'))
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
        && !key_event.modifiers.contains(KeyModifiers::ALT)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::make_test_app;
    use super::*;
    use crate::history_cell;
    use crate::history_cell::HistoryCell;
    use crate::tui::TuiEvent;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use ratatui::layout::Rect;
    use std::sync::Arc;

    fn redesign_viewport() -> Rect {
        Rect::new(0, 0, 100, 24)
    }

    fn handle_redesign_key(app: &mut App, key_event: KeyEvent) -> RedesignShortcutAction {
        app.handle_redesign_shortcut_key(key_event, redesign_viewport())
    }

    fn populate_scrollable_transcript(app: &mut App) {
        let cwd = app.config.cwd.clone();
        app.transcript_cells = (0..24)
            .map(|idx| {
                Arc::new(history_cell::AgentMarkdownCell::new(
                    format!("line {idx}"),
                    cwd.as_path(),
                )) as Arc<dyn HistoryCell>
            })
            .collect();
    }

    #[tokio::test]
    async fn app_keymap_shortcuts_are_disabled_while_keymap_view_is_active() {
        let mut app = make_test_app().await;
        assert!(app.app_keymap_shortcuts_available());

        let keymap = app.keymap.clone();
        app.chat_widget.open_keymap_debug(&keymap);

        assert!(!app.app_keymap_shortcuts_available());
    }

    #[tokio::test]
    async fn redesign_question_mark_inserts_literal_text() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(app.chat_widget.composer_text_with_pending(), "?");
        assert!(!app.chat_widget.redesign_should_render_bottom_pane());
    }

    #[tokio::test]
    async fn redesign_ctrl_b_toggles_sidebar_focus() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.redesign_sidebar_state.focused());

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(!app.redesign_sidebar_state.focused());
    }

    #[tokio::test]
    async fn redesign_sidebar_navigation_selects_items() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(
            app.redesign_sidebar_state.selected(),
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::FinalOnly
            )
        );

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(
            app.redesign_sidebar_state.selected(),
            redesign_chrome::RedesignSidebarSelection::Action(
                redesign_chrome::RedesignSidebarItem::NewChat
            )
        );
    }

    #[tokio::test]
    async fn redesign_sidebar_enter_on_chat_returns_open_chat_action() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        let thread_id = ThreadId::new();
        app.primary_thread_id = Some(thread_id);
        app.active_thread_id = Some(thread_id);
        app.upsert_agent_picker_thread(
            thread_id, /*agent_nickname*/ None, /*agent_role*/ None,
            /*is_closed*/ false,
        );

        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );
        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::OpenChat(thread_id));
        assert!(!app.redesign_sidebar_state.focused());
    }

    #[tokio::test]
    async fn redesign_sidebar_enter_opens_selected_commands() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );
        handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(!app.redesign_sidebar_state.focused());
        assert_eq!(app.chat_widget.composer_text_with_pending(), "/");
        assert!(app.chat_widget.redesign_should_render_bottom_pane());
    }

    #[tokio::test]
    async fn redesign_sidebar_enter_on_new_chat_returns_start_action() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::StartNewChat);
        assert!(!app.redesign_sidebar_state.focused());
    }

    #[tokio::test]
    async fn redesign_sidebar_final_only_shortcut_toggles_filter() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.redesign_final_only_transcript);
    }

    #[tokio::test]
    async fn redesign_sidebar_question_mark_does_not_edit_composer() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.redesign_sidebar_state.focused());
        assert_eq!(app.chat_widget.composer_text_with_pending(), "");
    }

    #[tokio::test]
    async fn redesign_sidebar_transcript_item_returns_overlay_action() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
        );
        for _ in 0..5 {
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::OpenTranscript);
        assert!(!app.redesign_sidebar_state.focused());
    }

    #[tokio::test]
    async fn redesign_f1_opens_shortcuts_view() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.chat_widget.redesign_should_render_bottom_pane());
        assert!(!app.app_keymap_shortcuts_available());
    }

    #[tokio::test]
    async fn redesign_f2_opens_slash_commands_on_empty_draft() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(app.chat_widget.composer_text_with_pending(), "/");
        assert!(app.chat_widget.redesign_should_render_bottom_pane());
    }

    #[tokio::test]
    async fn redesign_f3_maps_to_clear_terminal_action() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::ClearTerminal);
    }

    #[tokio::test]
    async fn redesign_f4_opens_model_popup() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        let thread_id = ThreadId::new();
        app.chat_widget.handle_thread_session(ThreadSessionState {
            thread_id,
            forked_from_id: None,
            fork_parent_title: None,
            thread_name: None,
            model: "test-model".to_string(),
            model_provider_id: "test-provider".to_string(),
            service_tier: None,
            approval_policy: AskForApproval::Never,
            approvals_reviewer: ApprovalsReviewer::User,
            permission_profile: PermissionProfile::read_only(),
            active_permission_profile: None,
            cwd: app.config.cwd.clone(),
            instruction_source_paths: Vec::new(),
            reasoning_effort: Some(ReasoningEffortConfig::default()),
            message_history: None,
            network_proxy: None,
            rollout_path: None,
        });

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::F(4), KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.chat_widget.redesign_should_render_bottom_pane());
    }

    #[tokio::test]
    async fn redesign_esc_does_not_open_transcript_backtrack_path() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        assert!(app.chat_widget.composer_is_empty());
        assert!(!app.should_handle_backtrack_esc(esc));
    }

    #[tokio::test]
    async fn redesign_empty_up_scrolls_transcript_instead_of_composer_history() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        populate_scrollable_transcript(&mut app);

        let action = handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(app.chat_widget.composer_text_with_pending(), "");
        assert_eq!(app.redesign_transcript_scroll, REDESIGN_LINE_SCROLL);
    }

    #[tokio::test]
    async fn redesign_transcript_page_and_edge_scroll_keys_clamp_to_viewport() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        populate_scrollable_transcript(&mut app);
        let scroll_limit = redesign_chrome::transcript_scroll_limit(redesign_viewport(), &app);

        let action =
            handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(app.redesign_transcript_scroll, scroll_limit);

        let action = handle_redesign_key(
            &mut app,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
        );

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert!(app.redesign_transcript_scroll < scroll_limit);

        let action = handle_redesign_key(&mut app, KeyEvent::new(KeyCode::End, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::Redraw);
        assert_eq!(app.redesign_transcript_scroll, 0);
    }

    #[test]
    fn redesign_global_quit_key_matches_ctrl_c_only() {
        assert!(redesign_global_quit_key_matches(&TuiEvent::Key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
        )));
        assert!(redesign_global_quit_key_matches(&TuiEvent::Key(
            KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL)
        )));
        assert!(!redesign_global_quit_key_matches(&TuiEvent::Key(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)
        )));
        assert!(!redesign_global_quit_key_matches(&TuiEvent::Key(
            KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )
        )));
        assert!(!redesign_global_quit_key_matches(&TuiEvent::Key(
            KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                KeyEventKind::Release,
            )
        )));
    }

    #[tokio::test]
    async fn redesign_up_with_draft_stays_with_composer() {
        let mut app = make_test_app().await;
        app.redesign_chrome_enabled = true;
        populate_scrollable_transcript(&mut app);
        app.chat_widget.insert_str("draft");

        let action = handle_redesign_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));

        assert_eq!(action, RedesignShortcutAction::None);
        assert_eq!(app.redesign_transcript_scroll, 0);
        assert_eq!(app.chat_widget.composer_text_with_pending(), "draft");
    }
}
