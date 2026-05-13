//! User-initiated chat closing for the redesigned TUI.
//!
//! Closing a redesigned chat is a local UI action: the app stays open, the thread is unsubscribed
//! from the app-server connection, and sidebar/navigation state forgets the row entirely.

use super::*;

impl App {
    pub(super) async fn close_redesign_chat(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) -> Result<()> {
        if self.agent_navigation.get(&thread_id).is_none()
            && self.current_displayed_thread_id() != Some(thread_id)
            && !self.thread_event_channels.contains_key(&thread_id)
        {
            self.chat_widget.add_info_message(
                "No chat is available to close.".to_string(),
                /*hint*/ None,
            );
            return Ok(());
        }

        let was_displayed = self.current_displayed_thread_id() == Some(thread_id);
        let next_thread_id = if was_displayed {
            let chats = self.redesign_chat_entries();
            chats
                .iter()
                .position(|chat| chat.thread_id == thread_id)
                .and_then(|current_idx| {
                    chats
                        .iter()
                        .skip(current_idx + 1)
                        .chain(chats.iter().take(current_idx).rev())
                        .find(|chat| chat.thread_id != thread_id)
                        .map(|chat| chat.thread_id)
                })
        } else {
            None
        };

        self.interrupt_redesign_chat_if_needed(app_server, thread_id)
            .await;
        app_server.thread_unsubscribe(thread_id).await?;
        self.remove_redesign_chat_local(thread_id).await;

        let chat_count = self.redesign_chat_entries().len();
        self.redesign_sidebar_state
            .normalize_for_chat_count(chat_count);

        if !was_displayed {
            return Ok(());
        }

        if let Some(next_thread_id) = next_thread_id {
            self.select_agent_thread(tui, app_server, next_thread_id)
                .await?;
        } else {
            let init = self.chatwidget_init_for_forked_or_resumed_thread(
                tui,
                self.config.clone(),
                /*initial_user_message*/ None,
            );
            self.replace_chat_widget(ChatWidget::new_with_app_event(init));
            if let Err(err) = self.reset_for_thread_switch(tui) {
                tracing::warn!(error = %err, "failed to clear terminal after closing chat");
                self.chat_widget
                    .add_error_message(format!("Failed to redraw after closing chat: {err}"));
            }
            self.request_redesign_chat_start(app_server).await;
        }
        Ok(())
    }

    async fn interrupt_redesign_chat_if_needed(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) {
        let interrupt_result =
            if let Some(turn_id) = self.active_turn_id_for_thread(thread_id).await {
                app_server.turn_interrupt(thread_id, turn_id).await
            } else if self.redesign_chat_activity.get(&thread_id)
                == Some(&redesign_chrome::RedesignChatActivity::Working)
            {
                app_server.startup_interrupt(thread_id).await
            } else {
                return;
            };

        if let Err(err) = interrupt_result {
            tracing::warn!("failed to interrupt chat {thread_id} before closing: {err}");
        }
    }

    pub(super) async fn remove_redesign_chat_local(&mut self, thread_id: ThreadId) {
        self.abort_thread_event_listener(thread_id);
        self.thread_event_channels.remove(&thread_id);
        self.side_threads.remove(&thread_id);
        self.agent_navigation.remove(thread_id);
        self.redesign_chat_names.remove(&thread_id);
        self.redesign_chat_activity.remove(&thread_id);
        self.redesign_chat_unread.remove(&thread_id);
        self.redesign_plan_window_open_threads.remove(&thread_id);

        if self.active_thread_id == Some(thread_id) {
            self.active_thread_id = None;
            self.active_thread_rx = None;
        }
        if self.primary_thread_id == Some(thread_id) {
            self.primary_thread_id = None;
            self.primary_session_configured = None;
            self.last_subagent_backfill_attempt = None;
        }

        self.refresh_pending_thread_approvals().await;
        self.sync_active_agent_label();
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::make_test_app;
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn remove_redesign_chat_local_drops_sidebar_entry_and_thread_state() {
        let mut app = make_test_app().await;
        let active_thread_id = ThreadId::new();
        let other_thread_id = ThreadId::new();
        app.primary_thread_id = Some(active_thread_id);
        app.active_thread_id = Some(active_thread_id);
        let mut active_channel = ThreadEventChannel::new(/*capacity*/ 4);
        app.active_thread_rx = active_channel.receiver.take();
        app.thread_event_channels
            .insert(active_thread_id, ThreadEventChannel::new(/*capacity*/ 4));
        app.thread_event_channels
            .insert(other_thread_id, ThreadEventChannel::new(/*capacity*/ 4));
        app.upsert_agent_picker_thread(
            active_thread_id,
            Some("Main chat".to_string()),
            /*agent_role*/ None,
            /*is_closed*/ false,
        );
        app.upsert_agent_picker_thread(
            other_thread_id,
            Some("Keep chat".to_string()),
            /*agent_role*/ None,
            /*is_closed*/ false,
        );
        app.redesign_chat_unread.insert(active_thread_id);
        app.redesign_plan_window_open_threads
            .insert(active_thread_id);

        app.remove_redesign_chat_local(active_thread_id).await;

        assert_eq!(
            app.redesign_chat_entries()
                .into_iter()
                .map(|entry| entry.thread_id)
                .collect::<Vec<_>>(),
            vec![other_thread_id]
        );
        assert_eq!(app.active_thread_id, None);
        assert_eq!(app.primary_thread_id, None);
        assert!(!app.thread_event_channels.contains_key(&active_thread_id));
        assert!(!app.redesign_chat_unread.contains(&active_thread_id));
        assert!(
            !app.redesign_plan_window_open_threads
                .contains(&active_thread_id)
        );
    }
}
