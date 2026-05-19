use codex_protocol::ThreadId;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::RedesignChromeContext;
use super::truncate_text;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RedesignSidebarState {
    focused: bool,
    selected: RedesignSidebarSelection,
}

impl Default for RedesignSidebarState {
    fn default() -> Self {
        Self {
            focused: false,
            selected: RedesignSidebarSelection::Chat(0),
        }
    }
}

impl RedesignSidebarState {
    pub(crate) fn focused(self) -> bool {
        self.focused
    }

    pub(crate) fn selected(self) -> RedesignSidebarSelection {
        self.selected
    }

    pub(crate) fn toggle_focus(&mut self, chat_count: usize) {
        self.focused = !self.focused;
        if self.focused {
            self.normalize_selection(chat_count);
        }
    }

    pub(crate) fn blur(&mut self) {
        self.focused = false;
    }

    pub(crate) fn select_previous(&mut self, chat_count: usize) {
        self.normalize_selection(chat_count);
        self.selected = match self.selected {
            RedesignSidebarSelection::Chat(idx) if idx > 0 => {
                RedesignSidebarSelection::Chat(idx - 1)
            }
            RedesignSidebarSelection::Chat(_) => {
                RedesignSidebarSelection::Action(RedesignSidebarItem::Editor)
            }
            RedesignSidebarSelection::Action(RedesignSidebarItem::NewChat) if chat_count > 0 => {
                RedesignSidebarSelection::Chat(chat_count - 1)
            }
            RedesignSidebarSelection::Action(item) => {
                RedesignSidebarSelection::Action(item.previous())
            }
        };
    }

    pub(crate) fn select_next(&mut self, chat_count: usize) {
        self.normalize_selection(chat_count);
        self.selected = match self.selected {
            RedesignSidebarSelection::Chat(idx) if idx + 1 < chat_count => {
                RedesignSidebarSelection::Chat(idx + 1)
            }
            RedesignSidebarSelection::Chat(_) => {
                RedesignSidebarSelection::Action(RedesignSidebarItem::Commands)
            }
            RedesignSidebarSelection::Action(RedesignSidebarItem::Editor) if chat_count > 0 => {
                RedesignSidebarSelection::Chat(0)
            }
            RedesignSidebarSelection::Action(item) => RedesignSidebarSelection::Action(item.next()),
        };
    }

    pub(crate) fn normalize_for_chat_count(&mut self, chat_count: usize) {
        self.normalize_selection(chat_count);
    }

    fn normalize_selection(&mut self, chat_count: usize) {
        if chat_count == 0 {
            if matches!(self.selected, RedesignSidebarSelection::Chat(_)) {
                self.selected = RedesignSidebarSelection::Action(RedesignSidebarItem::NewChat);
            }
            return;
        }

        if let RedesignSidebarSelection::Chat(idx) = self.selected
            && idx >= chat_count
        {
            self.selected = RedesignSidebarSelection::Chat(chat_count - 1);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RedesignSidebarSelection {
    Chat(usize),
    Action(RedesignSidebarItem),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RedesignSidebarItem {
    NewChat,
    FinalOnly,
    Commands,
    Models,
    History,
    Transcript,
    Terminals,
    Editor,
}

impl RedesignSidebarItem {
    const ALL: [Self; 8] = [
        Self::NewChat,
        Self::FinalOnly,
        Self::Commands,
        Self::Models,
        Self::History,
        Self::Transcript,
        Self::Terminals,
        Self::Editor,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::NewChat => "NEW CHAT",
            Self::FinalOnly => "FINAL ONLY",
            Self::Commands => "COMMANDS",
            Self::Models => "MODELS",
            Self::History => "HISTORY",
            Self::Transcript => "TRANSCRIPT",
            Self::Terminals => "TERMINALS",
            Self::Editor => "EDITOR",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::NewChat => "N",
            Self::FinalOnly => "F",
            Self::Commands => "Alt-/",
            Self::Models => "Alt-M",
            Self::History => "C-R",
            Self::Transcript => "C-T",
            Self::Terminals => "Alt-T",
            Self::Editor => "C-G",
        }
    }

    fn previous(self) -> Self {
        let idx = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        let next_idx = idx.checked_sub(1).unwrap_or(Self::ALL.len() - 1);
        Self::ALL[next_idx]
    }

    fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RedesignChatActivity {
    Idle,
    Working,
    Done,
    NeedsInput,
    Failed,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RedesignChatListEntry {
    pub(crate) thread_id: ThreadId,
    pub(crate) label: String,
    pub(crate) activity: RedesignChatActivity,
    pub(crate) is_active: bool,
    pub(crate) unread: bool,
}

pub(super) fn render_side_nav(
    area: Rect,
    buf: &mut Buffer,
    context: &RedesignChromeContext,
    sidebar: RedesignSidebarState,
) {
    if area.is_empty() {
        return;
    }

    let content_area = Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height);
    let content_width = content_area.width;
    let mut lines = vec![
        Line::from(vec![" ".into(), "CHATS".cyan().bold()]),
        Line::from(vec![
            " ".into(),
            if sidebar.focused() {
                "Alt-B close".magenta().bold()
            } else {
                "Alt-B focus".dim()
            },
        ]),
        Line::from(""),
    ];
    let action_row_count = 2 + RedesignSidebarItem::ALL.len() as u16;
    let fixed_sidebar_rows = lines.len() as u16 + action_row_count;
    let chat_row_capacity = area.height.saturating_sub(fixed_sidebar_rows) as usize;
    if context.chats.is_empty() {
        lines.push(Line::from(vec![" ".into(), "No chats yet".dim()]));
    } else if chat_row_capacity > 0 {
        let selected_chat_idx = match sidebar.selected() {
            RedesignSidebarSelection::Chat(idx) => idx.min(context.chats.len() - 1),
            RedesignSidebarSelection::Action(_) => 0,
        };
        let chat_start = selected_chat_idx.saturating_sub(chat_row_capacity.saturating_sub(1));
        let chat_end = (chat_start + chat_row_capacity).min(context.chats.len());
        lines.extend(context.chats[chat_start..chat_end].iter().enumerate().map(
            |(offset, chat)| chat_item_line(chat_start + offset, chat, sidebar, content_width),
        ));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![" ".into(), "ACTIONS".cyan().bold()]));
    lines.extend(
        RedesignSidebarItem::ALL
            .into_iter()
            .map(|item| sidebar_item_line(item, sidebar, context.final_only)),
    );
    Paragraph::new(lines).render(content_area, buf);

    let border_x = area.right().saturating_sub(1);
    for y in area.y..area.bottom() {
        buf[(border_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }
}

fn chat_item_line(
    idx: usize,
    chat: &RedesignChatListEntry,
    sidebar: RedesignSidebarState,
    content_width: u16,
) -> Line<'static> {
    let selected = sidebar.selected() == RedesignSidebarSelection::Chat(idx);
    let marker = if selected && sidebar.focused() {
        "› ".magenta().bold()
    } else if selected {
        "• ".cyan()
    } else {
        "  ".into()
    };
    let status = chat_status_span(chat);
    let label = truncate_text(&chat.label, content_width.saturating_sub(9));
    let label = if chat.is_active {
        Span::from(label).bold()
    } else if chat.unread {
        Span::from(label).cyan().bold()
    } else if chat.activity == RedesignChatActivity::Closed {
        Span::from(label).dim()
    } else {
        Span::from(label)
    };

    Line::from(vec![marker, status, " ".into(), label])
}

fn sidebar_item_line(
    item: RedesignSidebarItem,
    sidebar: RedesignSidebarState,
    final_only: bool,
) -> Line<'static> {
    let selected = sidebar.selected() == RedesignSidebarSelection::Action(item);
    let marker = if selected && sidebar.focused() {
        "› ".magenta().bold()
    } else if selected {
        "• ".cyan()
    } else {
        "  ".into()
    };
    let hint = format!("{:<6}", item.hint());
    let label = if item == RedesignSidebarItem::FinalOnly {
        if final_only {
            "FINAL ONLY ON"
        } else {
            "FINAL ONLY OFF"
        }
    } else {
        item.label()
    };

    if selected && sidebar.focused() {
        Line::from(vec![
            marker,
            hint.magenta().bold(),
            " ".into(),
            label.bold(),
        ])
    } else if selected {
        Line::from(vec![marker, hint.cyan(), " ".into(), label.cyan().bold()])
    } else {
        Line::from(vec![marker, hint.dim(), " ".into(), label.dim()])
    }
}

fn chat_status_span(chat: &RedesignChatListEntry) -> Span<'static> {
    let label = if chat.is_active {
        "active"
    } else if chat.unread {
        "unread"
    } else {
        match chat.activity {
            RedesignChatActivity::Idle => "idle",
            RedesignChatActivity::Working => "work",
            RedesignChatActivity::Done => "done",
            RedesignChatActivity::NeedsInput => "needs",
            RedesignChatActivity::Failed => "failed",
            RedesignChatActivity::Closed => "closed",
        }
    };
    let label = format!("{label:<6}");

    if chat.is_active || chat.unread {
        label.cyan().bold()
    } else {
        match chat.activity {
            RedesignChatActivity::Idle | RedesignChatActivity::Closed => label.dim(),
            RedesignChatActivity::Working => label.green(),
            RedesignChatActivity::Done => label.cyan(),
            RedesignChatActivity::NeedsInput => label.magenta().bold(),
            RedesignChatActivity::Failed => label.red().bold(),
        }
    }
}
