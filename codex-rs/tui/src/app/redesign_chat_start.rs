//! Non-blocking new-chat startup for the redesigned TUI.
//!
//! Starting an app-server thread can be slow, so redesigned New Chat keeps the current thread active
//! and applies the new thread only after the background start request completes.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RedesignChatStartRequest {
    AlreadyPending,
    Started(Uuid),
}

impl App {
    pub(super) async fn start_redesign_chat_from_ui(
        &mut self,
        _tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
    ) -> Result<()> {
        match self.request_redesign_chat_start(app_server).await {
            RedesignChatStartRequest::AlreadyPending | RedesignChatStartRequest::Started(_) => {
                Ok(())
            }
        }
    }

    pub(super) async fn request_redesign_chat_start(
        &mut self,
        app_server: &mut AppServerSession,
    ) -> RedesignChatStartRequest {
        self.refresh_in_memory_config_from_disk_best_effort("starting a new chat")
            .await;

        if self.pending_redesign_chat_start.is_some() {
            self.chat_widget.add_info_message(
                "A new chat is already starting.".to_string(),
                /*hint*/ None,
            );
            return RedesignChatStartRequest::AlreadyPending;
        }

        self.upsert_current_displayed_thread_for_redesign();

        let config = self.fresh_session_config();
        let request_id = Uuid::new_v4();
        self.pending_redesign_chat_start = Some(PendingRedesignChatStart {
            request_id,
            config: config.clone(),
        });
        self.chat_widget
            .add_info_message("Starting a new chat...".to_string(), /*hint*/ None);

        let request = app_server.thread_start_request();
        let app_event_tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = request
                .start_thread(config, /*session_start_source*/ None)
                .await
                .map_err(|err| format!("{err:#}"));
            app_event_tx.send(AppEvent::RedesignChatStarted { request_id, result });
        });

        RedesignChatStartRequest::Started(request_id)
    }

    fn upsert_current_displayed_thread_for_redesign(&mut self) {
        let previous_thread_id = self.active_thread_id.or(self.chat_widget.thread_id());
        if let Some(thread_id) = previous_thread_id {
            let existing_entry = self.agent_navigation.get(&thread_id).cloned();
            let agent_nickname = existing_entry
                .as_ref()
                .and_then(|entry| entry.agent_nickname.clone())
                .or_else(|| self.redesign_thread_display_name(thread_id));
            let agent_role = existing_entry
                .as_ref()
                .and_then(|entry| entry.agent_role.clone());
            let is_closed = existing_entry.as_ref().is_some_and(|entry| entry.is_closed);
            self.upsert_agent_picker_thread(thread_id, agent_nickname, agent_role, is_closed);
        }
    }

    pub(super) async fn finish_redesign_chat_start(
        &mut self,
        tui: &mut tui::Tui,
        request_id: Uuid,
        result: std::result::Result<AppServerStartedThread, String>,
    ) -> Result<()> {
        let Some(pending) = self.pending_redesign_chat_start.take() else {
            return Ok(());
        };
        if pending.request_id != request_id {
            self.pending_redesign_chat_start = Some(pending);
            return Ok(());
        }
        let started = result.map_err(color_eyre::eyre::Report::msg)?;

        self.upsert_current_displayed_thread_for_redesign();
        self.config = pending.config;
        self.store_active_thread_receiver().await;
        self.active_thread_id = None;
        self.active_thread_rx = None;

        let init = self.chatwidget_init_for_forked_or_resumed_thread(
            tui,
            self.config.clone(),
            /*initial_user_message*/ None,
        );
        self.replace_chat_widget(ChatWidget::new_with_app_event(init));
        let reset_error = self.reset_for_thread_switch(tui).err();
        self.enqueue_primary_thread_session(started.session, started.turns)
            .await?;
        if let Some(err) = reset_error {
            tracing::warn!(error = %err, "failed to clear terminal while starting redesign chat");
            self.chat_widget
                .add_error_message(format!("Failed to redraw new chat: {err}"));
        }
        Ok(())
    }
}
