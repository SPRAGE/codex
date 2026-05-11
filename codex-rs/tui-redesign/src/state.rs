#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DemoMode {
    Idle,
    Running,
    Approval,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedesignState {
    pub top: TopContext,
    pub workspace: WorkspaceContext,
    pub transcript: Vec<TranscriptEntry>,
    pub approval: Option<ApprovalRequest>,
    pub work: Option<WorkStatus>,
    pub composer: ComposerState,
    pub footer_shortcuts: Vec<FooterShortcut>,
    pub focus: FocusTarget,
    pub overlay: Overlay,
    pub approval_choice: ApprovalChoice,
    pub command_choice: CommandChoice,
    pub command_query: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopContext {
    pub product: String,
    pub model: String,
    pub reasoning: String,
    pub permissions: String,
    pub approval_mode: String,
    pub context_left: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceContext {
    pub path: String,
    pub branch: String,
    pub changed_files: String,
    pub thread_title: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    User,
    Codex,
    Running,
    ApprovalNeeded,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptEntry {
    pub role: Role,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalRequest {
    pub title: String,
    pub command: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkStatus {
    pub label: String,
    pub detail: String,
    pub elapsed: String,
    pub queued_messages: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComposerState {
    pub title: String,
    pub placeholder: String,
    pub draft: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FooterShortcut {
    pub key: String,
    pub label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusTarget {
    Transcript,
    Approval,
    Composer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overlay {
    None,
    Commands,
    Help,
    History,
    Transcript,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalChoice {
    Approve,
    ApproveSession,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandChoice {
    NewThread,
    SimulateRun,
    SimulateApproval,
    ClearDraft,
    ClearTranscript,
    OpenHelp,
}

const COMMAND_CHOICES: [CommandChoice; 6] = [
    CommandChoice::NewThread,
    CommandChoice::SimulateRun,
    CommandChoice::SimulateApproval,
    CommandChoice::ClearDraft,
    CommandChoice::ClearTranscript,
    CommandChoice::OpenHelp,
];

impl CommandChoice {
    pub fn label(self) -> &'static str {
        match self {
            CommandChoice::NewThread => "new-thread",
            CommandChoice::SimulateRun => "simulate-run",
            CommandChoice::SimulateApproval => "simulate-approval",
            CommandChoice::ClearDraft => "clear-draft",
            CommandChoice::ClearTranscript => "clear-transcript",
            CommandChoice::OpenHelp => "help",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            CommandChoice::NewThread => "Reset local transcript",
            CommandChoice::SimulateRun => "Show running status",
            CommandChoice::SimulateApproval => "Create approval request",
            CommandChoice::ClearDraft => "Clear composer text",
            CommandChoice::ClearTranscript => "Clear visible transcript",
            CommandChoice::OpenHelp => "Open shortcuts",
        }
    }

    fn matches_query(self, query: &str) -> bool {
        query.is_empty()
            || self.label().contains(query)
            || self.description().to_ascii_lowercase().contains(query)
    }
}

impl RedesignState {
    pub fn demo(mode: DemoMode) -> Self {
        let approval = matches!(mode, DemoMode::Approval).then(|| ApprovalRequest {
            title: "APVL_REQ".to_string(),
            command: "git commit -m \"Update TUI layout\"".to_string(),
            reason: "Commit requested by user".to_string(),
        });
        let has_approval = approval.is_some();

        let work = match mode {
            DemoMode::Idle => None,
            DemoMode::Running => Some(WorkStatus {
                label: "Running".to_string(),
                detail: "linting project...".to_string(),
                elapsed: "00:14".to_string(),
                queued_messages: 0,
            }),
            DemoMode::Approval => Some(WorkStatus {
                label: "Running".to_string(),
                detail: "linting project...".to_string(),
                elapsed: "00:14".to_string(),
                queued_messages: 0,
            }),
        };

        Self {
            top: TopContext {
                product: "Codex".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning: "xhigh".to_string(),
                permissions: "workspace-write".to_string(),
                approval_mode: "auto-review".to_string(),
                context_left: "72% left".to_string(),
            },
            workspace: WorkspaceContext {
                path: "/home/shaun/codes/codex".to_string(),
                branch: "redesign-tui".to_string(),
                changed_files: "3 files".to_string(),
                thread_title: "Improve terminal UI".to_string(),
            },
            transcript: vec![
                TranscriptEntry {
                    role: Role::User,
                    text: "Let's redesign the TUI to be more intuitive for CLI users.".to_string(),
                },
                TranscriptEntry {
                    role: Role::Codex,
                    text: "Agreed. I'll focus on information density and clear keyboard shortcuts."
                        .to_string(),
                },
            ],
            approval,
            work,
            composer: ComposerState {
                title: "Message Codex".to_string(),
                placeholder: "Describe the next change...".to_string(),
                draft: String::new(),
            },
            footer_shortcuts: vec![
                FooterShortcut {
                    key: "?".to_string(),
                    label: "shortcuts".to_string(),
                },
                FooterShortcut {
                    key: "Ctrl+R".to_string(),
                    label: "history".to_string(),
                },
                FooterShortcut {
                    key: "Ctrl+T".to_string(),
                    label: "transcript".to_string(),
                },
                FooterShortcut {
                    key: "Shift+Tab".to_string(),
                    label: "mode".to_string(),
                },
            ],
            focus: if has_approval {
                FocusTarget::Approval
            } else {
                FocusTarget::Composer
            },
            overlay: Overlay::None,
            approval_choice: ApprovalChoice::Approve,
            command_choice: CommandChoice::NewThread,
            command_query: String::new(),
            status: "Tab focus | ? help".to_string(),
        }
    }

    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            FocusTarget::Transcript => {
                if self.approval.is_some() {
                    FocusTarget::Approval
                } else {
                    FocusTarget::Composer
                }
            }
            FocusTarget::Approval => FocusTarget::Composer,
            FocusTarget::Composer => FocusTarget::Transcript,
        };
    }

    pub fn focus_previous(&mut self) {
        self.focus = match self.focus {
            FocusTarget::Transcript => FocusTarget::Composer,
            FocusTarget::Approval => FocusTarget::Transcript,
            FocusTarget::Composer => {
                if self.approval.is_some() {
                    FocusTarget::Approval
                } else {
                    FocusTarget::Transcript
                }
            }
        };
    }

    pub fn select_next_approval_action(&mut self) {
        self.approval_choice = match self.approval_choice {
            ApprovalChoice::Approve => ApprovalChoice::ApproveSession,
            ApprovalChoice::ApproveSession => ApprovalChoice::Deny,
            ApprovalChoice::Deny => ApprovalChoice::Approve,
        };
    }

    pub fn select_previous_approval_action(&mut self) {
        self.approval_choice = match self.approval_choice {
            ApprovalChoice::Approve => ApprovalChoice::Deny,
            ApprovalChoice::ApproveSession => ApprovalChoice::Approve,
            ApprovalChoice::Deny => ApprovalChoice::ApproveSession,
        };
    }

    pub fn open_commands(&mut self) {
        self.overlay = Overlay::Commands;
        self.command_query.clear();
        self.ensure_selected_command_visible();
    }

    pub fn push_command_query_char(&mut self, character: char) {
        self.command_query.push(character.to_ascii_lowercase());
        self.ensure_selected_command_visible();
    }

    pub fn pop_command_query_char(&mut self) {
        self.command_query.pop();
        self.ensure_selected_command_visible();
    }

    pub fn visible_commands(&self) -> Vec<CommandChoice> {
        COMMAND_CHOICES
            .iter()
            .copied()
            .filter(|choice| choice.matches_query(&self.command_query))
            .collect()
    }

    pub fn select_next_command(&mut self) {
        let commands = self.visible_commands();
        let Some(index) = commands
            .iter()
            .position(|choice| *choice == self.command_choice)
        else {
            self.ensure_selected_command_visible();
            return;
        };
        self.command_choice = commands[(index + 1) % commands.len()];
    }

    pub fn select_previous_command(&mut self) {
        let commands = self.visible_commands();
        let Some(index) = commands
            .iter()
            .position(|choice| *choice == self.command_choice)
        else {
            self.ensure_selected_command_visible();
            return;
        };
        self.command_choice = commands[(index + commands.len() - 1) % commands.len()];
    }

    pub fn apply_selected_command(&mut self) {
        if !self.visible_commands().contains(&self.command_choice) {
            self.status = "No matching command".to_string();
            return;
        }

        match self.command_choice {
            CommandChoice::NewThread => {
                self.transcript = vec![TranscriptEntry {
                    role: Role::Codex,
                    text: "New local design thread started.".to_string(),
                }];
                self.approval = None;
                self.work = None;
                self.composer.draft.clear();
                self.focus = FocusTarget::Composer;
                self.overlay = Overlay::None;
                self.status = "Started a new local design thread".to_string();
            }
            CommandChoice::SimulateRun => {
                self.work = Some(WorkStatus {
                    label: "Running".to_string(),
                    detail: "checking TUI interaction model...".to_string(),
                    elapsed: "00:03".to_string(),
                    queued_messages: 0,
                });
                self.overlay = Overlay::None;
                self.status = "Simulated a running task".to_string();
            }
            CommandChoice::SimulateApproval => {
                self.approval = Some(ApprovalRequest {
                    title: "APVL_REQ".to_string(),
                    command: "cargo test -p codex-tui".to_string(),
                    reason: "Prototype requested a verification run".to_string(),
                });
                self.work = Some(WorkStatus {
                    label: "Paused".to_string(),
                    detail: "waiting for approval".to_string(),
                    elapsed: "00:00".to_string(),
                    queued_messages: 0,
                });
                self.focus = FocusTarget::Approval;
                self.overlay = Overlay::None;
                self.status = "Simulated an approval request".to_string();
            }
            CommandChoice::ClearDraft => {
                self.composer.draft.clear();
                self.focus = FocusTarget::Composer;
                self.overlay = Overlay::None;
                self.status = "Cleared composer draft".to_string();
            }
            CommandChoice::ClearTranscript => {
                self.transcript.clear();
                self.overlay = Overlay::None;
                self.status = "Cleared transcript".to_string();
            }
            CommandChoice::OpenHelp => {
                self.overlay = Overlay::Help;
                self.status = "Opened shortcuts".to_string();
            }
        }
    }

    fn ensure_selected_command_visible(&mut self) {
        let commands = self.visible_commands();
        if !commands.contains(&self.command_choice)
            && let Some(first_choice) = commands.first()
        {
            self.command_choice = *first_choice;
        }
    }

    pub fn submit_composer(&mut self) {
        let draft = self.composer.draft.trim().to_string();
        if draft.is_empty() {
            self.status = "Composer is empty".to_string();
            return;
        }

        self.transcript.push(TranscriptEntry {
            role: Role::User,
            text: draft,
        });
        self.transcript.push(TranscriptEntry {
            role: Role::Codex,
            text: "Queued. This prototype records the turn locally; wiring to Codex core is the next integration step."
                .to_string(),
        });
        self.composer.draft.clear();
        self.work = Some(WorkStatus {
            label: "Queued".to_string(),
            detail: "waiting for Codex core integration".to_string(),
            elapsed: "00:00".to_string(),
            queued_messages: 1,
        });
        self.status = "Submitted draft into the local transcript".to_string();
    }

    pub fn apply_selected_approval_action(&mut self) {
        let Some(approval) = self.approval.take() else {
            self.status = "No approval request is active".to_string();
            return;
        };

        let (label, detail) = match self.approval_choice {
            ApprovalChoice::Approve => ("Approved", "Command approved for this run"),
            ApprovalChoice::ApproveSession => ("Approved", "Command approved for this session"),
            ApprovalChoice::Deny => ("Denied", "Command denied by user"),
        };
        self.transcript.push(TranscriptEntry {
            role: Role::ApprovalNeeded,
            text: format!("{label}: {}", approval.command),
        });
        self.work = Some(WorkStatus {
            label: label.to_string(),
            detail: detail.to_string(),
            elapsed: "00:00".to_string(),
            queued_messages: 0,
        });
        self.focus = FocusTarget::Composer;
        self.status = detail.to_string();
    }
}
