//! Stitch-inspired TUI surface.
//!
//! The redesigned mode owns the full terminal frame instead of embedding the legacy chat widget.
//! Event handling, submission, approvals, and command dispatch still stay with `ChatWidget`; this
//! module renders a dedicated top bar, side navigation, transcript viewport, work strip, composer,
//! and footer from the existing application state.

use crate::app::App;
use crate::app::AppFrameRender;
use crate::history_cell;
use crate::history_cell::HistoryCell;
use crate::motion::MotionMode;
use crate::motion::ReducedMotionIndicator;
use crate::motion::rotating_activity_indicator;
use crate::status::format_directory_display;
use crate::status::format_tokens_compact;
use crate::token_usage::TokenUsage;
use crate::version::CODEX_CLI_VERSION;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_lines;
use codex_protocol::models::PermissionProfile;
use crossterm::cursor::SetCursorStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use std::time::Instant;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

const TOP_ROWS: u16 = 2;
const CHAT_SEPARATOR_ROWS: u16 = 1;
const CHAT_HEADER_ROWS: u16 = 1;
const FOOTER_ROWS: u16 = 1;
const COMPOSER_ROWS: u16 = 2;
const WIDE_SIDE_WIDTH: u16 = 24;
const RIGHT_RAIL_WIDTH: u16 = 30;
const MIN_WIDE_WIDTH: u16 = 88;
const MIN_RIGHT_RAIL_WIDTH: u16 = 120;
const COMPACT_SIDE_WIDTH: u16 = 22;
const MIN_COMPACT_SIDEBAR_WIDTH: u16 = 56;

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
    Editor,
}

impl RedesignSidebarItem {
    const ALL: [Self; 7] = [
        Self::NewChat,
        Self::FinalOnly,
        Self::Commands,
        Self::Models,
        Self::History,
        Self::Transcript,
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
            Self::Editor => "EDITOR",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::NewChat => "N",
            Self::FinalOnly => "F",
            Self::Commands => "F2",
            Self::Models => "F4",
            Self::History => "C-R",
            Self::Transcript => "C-T",
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
    pub(crate) thread_id: codex_protocol::ThreadId,
    pub(crate) label: String,
    pub(crate) activity: RedesignChatActivity,
    pub(crate) is_active: bool,
    pub(crate) unread: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RedesignChromeContext {
    product: String,
    model: String,
    reasoning: String,
    token_usage: TokenUsage,
    pricing: Option<String>,
    permissions: String,
    approval: String,
    context_left: String,
    cwd: String,
    branch: String,
    changes: String,
    thread: String,
    working: bool,
    animations_enabled: bool,
    work_started_at: Option<Instant>,
    chats: Vec<RedesignChatListEntry>,
    final_only: bool,
}

impl RedesignChromeContext {
    pub(crate) fn from_app(app: &App) -> Self {
        let config = app.chat_widget.config_ref();
        let permission_profile = config.permissions.permission_profile();
        let permissions = config
            .permissions
            .active_permission_profile()
            .map(|profile| profile.id)
            .unwrap_or_else(|| permission_profile_label(&permission_profile).to_string());
        let approval = format!(
            "{}/{}",
            config.permissions.approval_policy.get(),
            config.approvals_reviewer
        );
        let context_left = app
            .chat_widget
            .redesign_context_remaining_percent()
            .map(|percent| format!("{percent}% left"))
            .unwrap_or_else(|| "context --".to_string());
        let branch = app
            .chat_widget
            .status_line_branch_name()
            .unwrap_or("unknown")
            .to_string();
        let changes = app
            .chat_widget
            .branch_change_summary()
            .unwrap_or_else(|| "--".to_string());
        let thread = app
            .chat_widget
            .thread_name()
            .unwrap_or_else(|| "New thread".to_string());

        Self {
            product: "CODEX_CLI".to_string(),
            model: app.chat_widget.redesign_model_label(),
            reasoning: app.chat_widget.redesign_reasoning_effort_label(),
            token_usage: app.chat_widget.token_usage(),
            pricing: None,
            permissions,
            approval,
            context_left,
            cwd: format_directory_display(config.cwd.as_path(), Some(48)),
            branch,
            changes,
            thread,
            working: app.chat_widget.redesign_task_running(),
            animations_enabled: app.chat_widget.redesign_animations_enabled(),
            work_started_at: app.chat_widget.redesign_work_started_at(),
            chats: app.redesign_chat_entries(),
            final_only: app.redesign_final_only_transcript,
        }
    }

    #[cfg(test)]
    fn fixture() -> Self {
        Self {
            product: "CODEX_CLI".to_string(),
            model: "gpt-5.4".to_string(),
            reasoning: "xhigh".to_string(),
            token_usage: TokenUsage {
                input_tokens: 1_200,
                cached_input_tokens: 0,
                output_tokens: 900,
                reasoning_output_tokens: 150,
                total_tokens: 2_250,
            },
            pricing: None,
            permissions: "workspace-write".to_string(),
            approval: "auto-review".to_string(),
            context_left: "72% left".to_string(),
            cwd: "~/codes/codex".to_string(),
            branch: "redesign-tui".to_string(),
            changes: "3 files".to_string(),
            thread: "Improve terminal UI".to_string(),
            working: true,
            animations_enabled: true,
            work_started_at: None,
            chats: vec![
                RedesignChatListEntry {
                    thread_id: codex_protocol::ThreadId::new(),
                    label: "Main [default]".to_string(),
                    activity: RedesignChatActivity::Working,
                    is_active: true,
                    unread: false,
                },
                RedesignChatListEntry {
                    thread_id: codex_protocol::ThreadId::new(),
                    label: "Scout [explorer]".to_string(),
                    activity: RedesignChatActivity::Done,
                    is_active: false,
                    unread: true,
                },
                RedesignChatListEntry {
                    thread_id: codex_protocol::ThreadId::new(),
                    label: "Patch [worker]".to_string(),
                    activity: RedesignChatActivity::NeedsInput,
                    is_active: false,
                    unread: false,
                },
            ],
            final_only: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TranscriptRole {
    User,
    Codex,
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BubbleAlign {
    Left,
    Right,
    Center,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TranscriptBlock {
    role: TranscriptRole,
    lines: Vec<Line<'static>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SystemRailBlock {
    title: &'static str,
    lines: Vec<Line<'static>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RedesignLayout {
    side: Rect,
    main: Rect,
    right: Rect,
    chat_separator: Rect,
    chat_header: Rect,
    transcript: Rect,
    composer: Rect,
    footer: Rect,
}

pub(crate) fn render_app(area: Rect, buf: &mut Buffer, app: &App) -> AppFrameRender {
    let context = RedesignChromeContext::from_app(app);
    let legacy_bottom_pane = app.chat_widget.redesign_should_render_bottom_pane();
    let layout = layout_for(area, app, legacy_bottom_pane);

    render_background(area, buf);
    render_chrome(area, buf, &context, app.redesign_sidebar_state);
    app.chat_widget
        .redesign_schedule_work_indicator_frame_if_needed();
    let transcript_blocks = transcript_blocks(app, layout.transcript.width);
    let system_blocks = system_rail_blocks(app, layout.right.width);
    let scroll_limit = transcript_scroll_limit_for_blocks(layout.transcript, &transcript_blocks);
    render_transcript(
        layout.transcript,
        buf,
        &transcript_blocks,
        app.redesign_transcript_scroll.min(scroll_limit),
    );
    render_system_rail(layout.right, buf, &context, &system_blocks);
    if legacy_bottom_pane {
        app.chat_widget
            .render_redesign_bottom_pane(layout.composer, buf);
        return AppFrameRender {
            cursor_pos: app
                .chat_widget
                .redesign_bottom_pane_cursor_pos(layout.composer),
            cursor_style: app
                .chat_widget
                .redesign_bottom_pane_cursor_style(layout.composer),
        };
    }

    let cursor_pos = render_composer(
        layout.composer,
        buf,
        &app.chat_widget.redesign_composer_text(),
        composer_activity_indicator(&context),
    );
    AppFrameRender {
        cursor_pos,
        cursor_style: SetCursorStyle::SteadyBar,
    }
}

#[cfg(test)]
fn content_width_for_terminal_width(width: u16) -> u16 {
    layout_for_dimensions(Rect::new(0, 0, width, 10), COMPOSER_ROWS)
        .transcript
        .width
}

pub(crate) fn transcript_scroll_limit(area: Rect, app: &App) -> usize {
    if area.is_empty() {
        return 0;
    }
    let layout = layout_for(
        area,
        app,
        app.chat_widget.redesign_should_render_bottom_pane(),
    );
    let blocks = transcript_blocks(app, layout.transcript.width);
    transcript_scroll_limit_for_blocks(layout.transcript, &blocks)
}

pub(crate) fn render_chrome(
    area: Rect,
    buf: &mut Buffer,
    context: &RedesignChromeContext,
    sidebar: RedesignSidebarState,
) {
    if area.is_empty() {
        return;
    }

    let side_width = side_width_for_state(area.width, sidebar);
    let layout = layout_for_dimensions_with_side(area, side_width, COMPOSER_ROWS);
    render_top_bar(area, buf, context);
    render_top_separator(area, buf);
    draw_horizontal_rule(layout.chat_separator, buf, layout.chat_separator.y);
    render_chat_bar(layout.chat_header, buf, context);
    render_side_nav(layout.side, buf, context, sidebar);
    render_footer(layout.footer, buf, context);
}

fn layout_for(area: Rect, app: &App, legacy_bottom_pane: bool) -> RedesignLayout {
    if area.is_empty() {
        return layout_for_dimensions(area, COMPOSER_ROWS);
    }

    let side_width = side_width_for_state(area.width, app.redesign_sidebar_state);
    let main_width = area
        .width
        .saturating_sub(side_width + right_rail_width(area.width, side_width));
    let available_body_height = area.height.saturating_sub(TOP_ROWS + FOOTER_ROWS);
    let available_chat_body_height =
        available_body_height.saturating_sub(CHAT_SEPARATOR_ROWS + CHAT_HEADER_ROWS);
    let composer_height = if legacy_bottom_pane {
        app.chat_widget
            .redesign_bottom_pane_desired_height(main_width)
            .min(available_chat_body_height.max(1))
            .max(1)
    } else {
        COMPOSER_ROWS.min(available_chat_body_height)
    };

    layout_for_dimensions_with_side(area, side_width, composer_height)
}

fn layout_for_dimensions(area: Rect, composer_height: u16) -> RedesignLayout {
    layout_for_dimensions_with_side(area, side_width(area.width), composer_height)
}

fn layout_for_dimensions_with_side(
    area: Rect,
    side_width: u16,
    composer_height: u16,
) -> RedesignLayout {
    if area.is_empty() {
        return RedesignLayout {
            side: Rect::new(area.x, area.y, 0, 0),
            main: Rect::new(area.x, area.y, 0, 0),
            right: Rect::new(area.x, area.y, 0, 0),
            chat_separator: Rect::new(area.x, area.y, 0, 0),
            chat_header: Rect::new(area.x, area.y, 0, 0),
            transcript: Rect::new(area.x, area.y, 0, 0),
            composer: Rect::new(area.x, area.y, 0, 0),
            footer: Rect::new(area.x, area.y, 0, 0),
        };
    }

    let footer = Rect::new(
        area.x,
        area.bottom().saturating_sub(FOOTER_ROWS),
        area.width,
        FOOTER_ROWS.min(area.height),
    );
    let body_y = area.y.saturating_add(TOP_ROWS.min(area.height));
    let body_bottom = footer.y;
    let body_height = body_bottom.saturating_sub(body_y);
    let side = Rect::new(area.x, body_y, side_width, body_height);
    let right_width = right_rail_width(area.width, side_width);
    let main = Rect::new(
        area.x.saturating_add(side_width),
        body_y,
        area.width.saturating_sub(side_width + right_width),
        body_height,
    );
    let right = Rect::new(main.right(), body_y, right_width, body_height);
    let chat_separator_height = CHAT_SEPARATOR_ROWS.min(main.height);
    let chat_separator = Rect::new(main.x, main.y, main.width, chat_separator_height);
    let chat_header_y = main.y.saturating_add(chat_separator_height);
    let chat_header_height =
        CHAT_HEADER_ROWS.min(main.height.saturating_sub(chat_separator_height));
    let chat_header = Rect::new(main.x, chat_header_y, main.width, chat_header_height);
    let chat_body_y = chat_header_y.saturating_add(chat_header_height);
    let chat_body_height = main
        .height
        .saturating_sub(chat_separator_height + chat_header_height);
    let composer_height = composer_height.min(chat_body_height);
    let transcript_height = chat_body_height.saturating_sub(composer_height);
    let transcript = Rect::new(main.x, chat_body_y, main.width, transcript_height);
    let composer = Rect::new(main.x, transcript.bottom(), main.width, composer_height);

    RedesignLayout {
        side,
        main,
        right,
        chat_separator,
        chat_header,
        transcript,
        composer,
        footer,
    }
}

fn render_background(area: Rect, buf: &mut Buffer) {
    buf.set_style(area, Style::default().bg(Color::Black));
}

fn render_top_bar(area: Rect, buf: &mut Buffer, context: &RedesignChromeContext) {
    let line = Line::from(vec![
        Span::from(product_version_label(&context.product))
            .magenta()
            .bold(),
    ]);
    render_line(area, buf, area.y, line);
}

fn render_top_separator(area: Rect, buf: &mut Buffer) {
    draw_horizontal_rule(area, buf, area.y.saturating_add(1));
}

fn product_version_label(product: &str) -> String {
    format!("{product} v{CODEX_CLI_VERSION}")
}

fn render_chat_bar(area: Rect, buf: &mut Buffer, context: &RedesignChromeContext) {
    if area.is_empty() {
        return;
    }

    let mut spans = if area.width >= 72 {
        vec![
            "chat ".dim(),
            Span::from(format!("{} {}", context.model, context.reasoning)).magenta(),
            "  ctx ".dim(),
            Span::from(context.context_left.clone()),
            "  tokens ".dim(),
            Span::from(token_usage_label(&context.token_usage)).cyan(),
        ]
    } else {
        vec![
            Span::from(format!("{} {}", context.model, context.reasoning)).magenta(),
            " | ctx ".dim(),
            Span::from(context.context_left.clone()),
            " | tok ".dim(),
            Span::from(token_usage_label(&context.token_usage)).cyan(),
        ]
    };
    if let Some(pricing) = &context.pricing {
        spans.push("  price ".dim());
        spans.push(Span::from(pricing.clone()).green());
    }

    render_line(area, buf, area.y, Line::from(spans));
}

fn token_usage_label(usage: &TokenUsage) -> String {
    format!(
        "in {} / out {} / total {}",
        format_tokens_compact(usage.input_tokens),
        format_tokens_compact(usage.output_tokens),
        format_tokens_compact(usage.total_tokens)
    )
}

fn render_side_nav(
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
                "C-B close".magenta().bold()
            } else {
                "C-B focus".dim()
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

fn render_transcript(
    area: Rect,
    buf: &mut Buffer,
    blocks: &[TranscriptBlock],
    scroll_offset: usize,
) {
    if area.is_empty() {
        return;
    }

    let lines = transcript_display_lines(blocks, area.width);
    let scroll_limit = lines.len().saturating_sub(area.height as usize);
    let scroll_offset = scroll_offset.min(scroll_limit);
    let visible_start = lines
        .len()
        .saturating_sub((area.height as usize).saturating_add(scroll_offset));
    let visible = lines
        .into_iter()
        .skip(visible_start)
        .take(area.height as usize)
        .collect::<Vec<_>>();
    Paragraph::new(visible)
        .wrap(Wrap { trim: false })
        .render(area, buf);

    if scroll_limit > 0 && area.width > 2 {
        let scrollbar_x = area.right().saturating_sub(1);
        for y in area.y..area.bottom() {
            buf[(scrollbar_x, y)]
                .set_symbol("|")
                .set_style(Style::new().dim());
        }
        let thumb_range = area.height.saturating_sub(1) as usize;
        let thumb_offset = (scroll_limit - scroll_offset) * thumb_range / scroll_limit;
        let thumb_y = area
            .y
            .saturating_add(thumb_offset as u16)
            .min(area.bottom().saturating_sub(1));
        buf[(scrollbar_x, thumb_y)].set_symbol("#");
    }
}

fn transcript_scroll_limit_for_blocks(area: Rect, blocks: &[TranscriptBlock]) -> usize {
    if area.is_empty() {
        return 0;
    }

    transcript_display_lines(blocks, area.width)
        .len()
        .saturating_sub(area.height as usize)
}

fn transcript_display_lines(blocks: &[TranscriptBlock], width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if blocks.is_empty() {
        lines.push(Line::from(vec![
            "MSG> ".magenta().bold(),
            "Start a task below.".dim(),
        ]));
    } else {
        for block in blocks {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.extend(bubble_lines(block, width));
        }
    }
    lines
}

fn render_system_rail(
    area: Rect,
    buf: &mut Buffer,
    context: &RedesignChromeContext,
    blocks: &[SystemRailBlock],
) {
    if area.is_empty() {
        return;
    }

    for y in area.y..area.bottom() {
        buf[(area.x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }
    let content = Rect::new(
        area.x.saturating_add(1),
        area.y,
        area.width.saturating_sub(1),
        area.height,
    );
    if content.is_empty() {
        return;
    }

    let permissions = truncate_text(&context.permissions, content.width.saturating_sub(7));
    let approval = truncate_text(&context.approval, content.width.saturating_sub(7));
    let header = vec![
        Line::from(""),
        Line::from(vec![" ".into(), "SYSTEM".cyan().bold()]),
        Line::from(vec![" ".into(), "perm ".dim(), permissions.cyan()]),
        Line::from(vec![" ".into(), "appv ".dim(), approval.magenta()]),
        Line::from(vec![" ".into(), "thinking + events".dim()]),
        Line::from(""),
    ];
    let body_capacity = content.height.saturating_sub(header.len() as u16) as usize;
    let body = system_rail_display_lines(blocks, content.width);
    let body_start = body.len().saturating_sub(body_capacity);
    let mut lines = header;
    lines.extend(body.into_iter().skip(body_start).take(body_capacity));

    Paragraph::new(lines).render(content, buf);
}

fn system_rail_display_lines(blocks: &[SystemRailBlock], width: u16) -> Vec<Line<'static>> {
    if blocks.is_empty() {
        return vec![Line::from(vec![" ".into(), "No system activity".dim()])];
    }

    let wrap_width = width.saturating_sub(3).max(1) as usize;
    let mut lines = Vec::new();
    for block in blocks {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![" ".into(), block.title.cyan().bold()]));
        for line in &block.lines {
            for wrapped in
                adaptive_wrap_lines(std::iter::once(line.clone()), RtOptions::new(wrap_width))
            {
                let mut spans = vec!["  ".dim()];
                spans.extend(wrapped.spans);
                lines.push(Line::from(spans));
            }
        }
    }
    lines
}

fn composer_activity_indicator(context: &RedesignChromeContext) -> Option<Span<'static>> {
    if context.working {
        let motion_mode = MotionMode::from_animations_enabled(context.animations_enabled);
        rotating_activity_indicator(
            context.work_started_at,
            motion_mode,
            ReducedMotionIndicator::StaticBullet,
        )
    } else {
        None
    }
}

fn render_composer(
    area: Rect,
    buf: &mut Buffer,
    draft: &str,
    activity_indicator: Option<Span<'static>>,
) -> Option<(u16, u16)> {
    if area.is_empty() {
        return None;
    }

    draw_horizontal_rule(area, buf, area.y);
    let input_y = area
        .y
        .saturating_add(1)
        .min(area.bottom().saturating_sub(1));
    let input_area = Rect::new(area.x, input_y, area.width, 1);
    let placeholder = "Describe the next change...";
    let activity_width = activity_indicator
        .as_ref()
        .map(|indicator| UnicodeWidthStr::width(indicator.content.as_ref()) + 1)
        .unwrap_or_default();
    let mut spans = Vec::new();
    if let Some(indicator) = activity_indicator {
        spans.push(indicator);
        spans.push(" ".into());
    }
    spans.push("MSG> ".magenta().bold());
    if draft.is_empty() {
        spans.push(placeholder.dim());
    } else {
        spans.push(Span::from(draft.to_string()));
    }
    render_line(input_area, buf, input_y, Line::from(spans));

    let prefix_width = (activity_width + UnicodeWidthStr::width("MSG> ")) as u16;
    let draft_width = UnicodeWidthStr::width(draft) as u16;
    let cursor_x = input_area
        .x
        .saturating_add(prefix_width)
        .saturating_add(draft_width)
        .min(input_area.right().saturating_sub(1));
    Some((cursor_x, input_y))
}

fn render_footer(area: Rect, buf: &mut Buffer, context: &RedesignChromeContext) {
    if area.is_empty() {
        return;
    }

    let line = if area.width >= 90 {
        let hints = "  C-B side  F1 help  F2 cmds  F4 model  C-T transcript  C-C exit";
        let workspace_width = area
            .width
            .saturating_sub(UnicodeWidthStr::width(hints) as u16);
        let workspace = compact_workspace_label(context, workspace_width);
        Line::from(vec![
            Span::from(workspace).dim(),
            "  C-B".cyan(),
            " side".dim(),
            "  F1".cyan(),
            " help".dim(),
            "  F2".cyan(),
            " cmds".dim(),
            "  F4".cyan(),
            " model".dim(),
            "  C-T".cyan(),
            " transcript".dim(),
            "  C-C".cyan(),
            " exit".dim(),
        ])
    } else if area.width >= 64 {
        let hints = "  F1 help  C-T transcript  C-C exit";
        let workspace_width = area
            .width
            .saturating_sub(UnicodeWidthStr::width(hints) as u16);
        let workspace = compact_workspace_label(context, workspace_width);
        Line::from(vec![
            Span::from(workspace).dim(),
            "  F1".cyan(),
            " help".dim(),
            "  C-T".cyan(),
            " transcript".dim(),
            "  C-C".cyan(),
            " exit".dim(),
        ])
    } else {
        let hints = "  C-T transcript  C-C exit";
        let workspace_width = area
            .width
            .saturating_sub(UnicodeWidthStr::width(hints) as u16);
        let workspace = compact_workspace_label(context, workspace_width);
        Line::from(vec![
            Span::from(workspace).dim(),
            "  C-T".cyan(),
            " transcript".dim(),
            "  C-C".cyan(),
            " exit".dim(),
        ])
    };
    render_line(area, buf, area.y, line);
}

fn compact_workspace_label(context: &RedesignChromeContext, max_width: u16) -> String {
    let workspace = format!(
        "{} · {} · {} · {}",
        context.cwd, context.branch, context.changes, context.thread
    );
    truncate_text(&workspace, max_width)
}

fn transcript_blocks(app: &App, width: u16) -> Vec<TranscriptBlock> {
    let mut blocks = Vec::new();
    let content_width = width.saturating_sub(4).max(1);
    for cell in &app.transcript_cells {
        let cell = cell.as_ref();
        if is_startup_cell(cell) || is_system_rail_cell(cell) {
            continue;
        }

        let lines = cell_content_lines(cell, content_width);
        if !lines.is_empty() {
            push_transcript_block(
                &mut blocks,
                role_for_cell(cell),
                lines,
                cell.is_stream_continuation(),
            );
        }
    }

    if let Some(active) = app
        .chat_widget
        .redesign_active_final_output_display(content_width)
    {
        let lines = display_lines_to_content(active.lines);
        if !lines.is_empty() {
            push_transcript_block(
                &mut blocks,
                TranscriptRole::Codex,
                lines,
                active.is_stream_continuation,
            );
        }
    }

    blocks
}

fn system_rail_blocks(app: &App, width: u16) -> Vec<SystemRailBlock> {
    if width == 0 || app.redesign_final_only_transcript {
        return Vec::new();
    }

    let content_width = width.saturating_sub(4).max(1);
    let mut blocks = Vec::new();
    for cell in &app.transcript_cells {
        let cell = cell.as_ref();
        if !is_system_rail_cell(cell) {
            continue;
        }

        let lines = cell_content_lines(cell, content_width);
        if !lines.is_empty() {
            blocks.push(SystemRailBlock {
                title: system_rail_title(cell),
                lines,
            });
        }
    }

    if let Some(active) = app
        .chat_widget
        .redesign_active_system_display(content_width)
    {
        let lines = display_lines_to_content(active.lines);
        if !lines.is_empty() {
            blocks.push(SystemRailBlock {
                title: "ACTIVE",
                lines,
            });
        }
    }

    blocks
}

fn push_transcript_block(
    blocks: &mut Vec<TranscriptBlock>,
    role: TranscriptRole,
    lines: Vec<Line<'static>>,
    is_stream_continuation: bool,
) {
    if is_stream_continuation
        && let Some(previous) = blocks.last_mut()
        && previous.role == role
    {
        previous.lines.extend(lines);
        return;
    }

    blocks.push(TranscriptBlock { role, lines });
}

fn is_startup_cell(cell: &dyn HistoryCell) -> bool {
    cell.as_any().is::<history_cell::SessionHeaderHistoryCell>()
        || cell.as_any().is::<history_cell::SessionInfoCell>()
}

fn is_system_rail_cell(cell: &dyn HistoryCell) -> bool {
    !is_startup_cell(cell) && !is_final_output_cell(cell)
}

fn system_rail_title(cell: &dyn HistoryCell) -> &'static str {
    if cell.as_any().is::<history_cell::ReasoningSummaryCell>() {
        "THINKING"
    } else if cell.as_any().is::<history_cell::ProposedPlanCell>()
        || cell.as_any().is::<history_cell::ProposedPlanStreamCell>()
        || cell.as_any().is::<history_cell::PlanUpdateCell>()
    {
        "PLAN"
    } else if cell
        .as_any()
        .is::<history_cell::UnifiedExecInteractionCell>()
        || cell.as_any().is::<history_cell::PatchHistoryCell>()
        || cell.as_any().is::<history_cell::McpToolCallCell>()
        || cell.as_any().is::<history_cell::WebSearchCell>()
    {
        "TOOLS"
    } else if cell
        .as_any()
        .is::<history_cell::UpdateAvailableHistoryCell>()
        || cell.as_any().is::<history_cell::CyberPolicyNoticeCell>()
        || cell.as_any().is::<history_cell::DeprecationNoticeCell>()
    {
        "NOTICE"
    } else {
        "SYSTEM"
    }
}

fn role_for_cell(cell: &dyn HistoryCell) -> TranscriptRole {
    if cell.as_any().is::<history_cell::UserHistoryCell>() {
        TranscriptRole::User
    } else if cell.as_any().is::<history_cell::AgentMarkdownCell>()
        || cell.as_any().is::<history_cell::AgentMessageCell>()
        || cell.as_any().is::<history_cell::ReasoningSummaryCell>()
        || cell.as_any().is::<history_cell::ProposedPlanCell>()
        || cell.as_any().is::<history_cell::ProposedPlanStreamCell>()
        || cell.as_any().is::<history_cell::PlanUpdateCell>()
    {
        TranscriptRole::Codex
    } else {
        TranscriptRole::System
    }
}

fn is_final_output_cell(cell: &dyn HistoryCell) -> bool {
    cell.as_any().is::<history_cell::UserHistoryCell>()
        || cell.as_any().is::<history_cell::AgentMarkdownCell>()
        || cell.as_any().is::<history_cell::AgentMessageCell>()
}

fn cell_content_lines(cell: &dyn HistoryCell, width: u16) -> Vec<Line<'static>> {
    if cell.as_any().is::<history_cell::UserHistoryCell>() {
        display_lines_to_content(cell.raw_lines())
    } else {
        display_lines_to_content(cell.display_lines(width))
    }
}

fn display_lines_to_content(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let mut out = lines
        .into_iter()
        .map(strip_legacy_prefix)
        .collect::<Vec<_>>();
    while out
        .first()
        .is_some_and(|line| plain_line_text(line).trim().is_empty())
    {
        out.remove(0);
    }
    while out
        .last()
        .is_some_and(|line| plain_line_text(line).trim().is_empty())
    {
        out.pop();
    }
    out
}

fn plain_line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>()
}

fn strip_legacy_prefix(line: Line<'static>) -> Line<'static> {
    let text = plain_line_text(&line);
    let trimmed = text.trim_start();
    let leading_trim_bytes = text.len().saturating_sub(trimmed.len());
    for prefix in ["› ", "• ", "> "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return drop_line_prefix(line, text.len().saturating_sub(rest.len()));
        }
    }
    drop_line_prefix(line, leading_trim_bytes)
}

fn drop_line_prefix(line: Line<'static>, mut bytes_to_drop: usize) -> Line<'static> {
    if bytes_to_drop == 0 {
        return line;
    }

    let mut spans = Vec::new();
    for span in line.spans {
        if bytes_to_drop >= span.content.len() {
            bytes_to_drop -= span.content.len();
            continue;
        }

        let content = span.content.into_owned();
        let keep = content[bytes_to_drop..].to_string();
        spans.push(Span::styled(keep, span.style));
        bytes_to_drop = 0;
    }

    Line::from(spans).style(line.style)
}

fn bubble_lines(block: &TranscriptBlock, area_width: u16) -> Vec<Line<'static>> {
    let viewport_width = area_width.saturating_sub(2).max(1) as usize;
    let max_bubble_width = bubble_max_width(block.role, viewport_width);
    let wrap_width = max_bubble_width.saturating_sub(4).max(1);
    let mut wrapped = Vec::new();

    for line in &block.lines {
        if plain_line_text(line).trim().is_empty() {
            wrapped.push(Line::from(""));
            continue;
        }
        wrapped.extend(adaptive_wrap_lines(
            std::iter::once(line.clone()),
            RtOptions::new(wrap_width),
        ));
    }
    if wrapped.is_empty() {
        wrapped.push(Line::from(""));
    }

    let inner_width = wrapped
        .iter()
        .map(line_width)
        .max()
        .unwrap_or(1)
        .max(role_name(block.role).len())
        .min(wrap_width);
    let bubble_width = inner_width + 4;
    let prefix_width = bubble_prefix_width(block.role, viewport_width, bubble_width);
    let label_width = UnicodeWidthStr::width(role_name(block.role));
    let label_prefix_width = match bubble_align(block.role) {
        BubbleAlign::Left => prefix_width,
        BubbleAlign::Right => prefix_width + bubble_width.saturating_sub(label_width),
        BubbleAlign::Center => prefix_width,
    };
    let bubble_style = bubble_style(block.role);
    let border_style = bubble_border_style(block.role);

    let mut lines = vec![
        Line::from(vec![
            Span::from(" ".repeat(label_prefix_width)),
            role_label_span(block.role),
        ]),
        Line::from(vec![
            Span::from(" ".repeat(prefix_width)),
            Span::styled(format!("╭{}╮", "─".repeat(inner_width + 2)), border_style),
        ]),
    ];

    for line in wrapped {
        let text_width = line_width(&line);
        let padding = inner_width.saturating_sub(text_width);
        let mut spans = vec![
            Span::from(" ".repeat(prefix_width)),
            Span::styled("│ ", border_style),
        ];
        spans.extend(styled_bubble_content_spans(line, bubble_style));
        spans.push(Span::styled(" ".repeat(padding), bubble_style));
        spans.push(Span::styled(" │", border_style));
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(vec![
        Span::from(" ".repeat(prefix_width)),
        Span::styled(format!("╰{}╯", "─".repeat(inner_width + 2)), border_style),
    ]));
    lines
}

fn styled_bubble_content_spans(line: Line<'static>, bubble_style: Style) -> Vec<Span<'static>> {
    line.spans
        .into_iter()
        .map(|span| {
            if span.style == Style::default() {
                Span::styled(span.content, bubble_style)
            } else {
                span
            }
        })
        .collect()
}

fn bubble_max_width(role: TranscriptRole, viewport_width: usize) -> usize {
    let ratio_width = match role {
        TranscriptRole::User => viewport_width.saturating_mul(3) / 4,
        TranscriptRole::Codex => viewport_width.saturating_mul(4) / 5,
        TranscriptRole::System => viewport_width.saturating_mul(7) / 10,
    };
    ratio_width.clamp(24.min(viewport_width), viewport_width.max(1))
}

fn bubble_prefix_width(role: TranscriptRole, viewport_width: usize, bubble_width: usize) -> usize {
    match bubble_align(role) {
        BubbleAlign::Left => 1.min(viewport_width.saturating_sub(bubble_width)),
        BubbleAlign::Right => viewport_width.saturating_sub(bubble_width),
        BubbleAlign::Center => viewport_width.saturating_sub(bubble_width) / 2,
    }
}

fn bubble_align(role: TranscriptRole) -> BubbleAlign {
    match role {
        TranscriptRole::User => BubbleAlign::Right,
        TranscriptRole::Codex => BubbleAlign::Left,
        TranscriptRole::System => BubbleAlign::Center,
    }
}

fn role_name(role: TranscriptRole) -> &'static str {
    match role {
        TranscriptRole::User => "You",
        TranscriptRole::Codex => "Codex",
        TranscriptRole::System => "System",
    }
}

fn role_label_span(role: TranscriptRole) -> Span<'static> {
    match role {
        TranscriptRole::User => role_name(role).cyan().bold(),
        TranscriptRole::Codex => role_name(role).magenta().bold(),
        TranscriptRole::System => role_name(role).dim().bold(),
    }
}

fn bubble_style(role: TranscriptRole) -> Style {
    match role {
        TranscriptRole::User => Style::new().fg(Color::Cyan),
        TranscriptRole::Codex => Style::new().fg(Color::Magenta),
        TranscriptRole::System => Style::new().fg(Color::DarkGray),
    }
}

fn bubble_border_style(role: TranscriptRole) -> Style {
    match role {
        TranscriptRole::User => Style::new().fg(Color::Cyan),
        TranscriptRole::Codex => Style::new().fg(Color::Magenta),
        TranscriptRole::System => Style::new().fg(Color::DarkGray),
    }
}

fn render_line(area: Rect, buf: &mut Buffer, y: u16, line: Line<'static>) {
    if area.width == 0 || y >= area.bottom() {
        return;
    }
    let line = truncate_line(line, area.width as usize);
    Paragraph::new(line).render(Rect::new(area.x, y, area.width, 1), buf);
}

fn draw_horizontal_rule(area: Rect, buf: &mut Buffer, y: u16) {
    if area.width == 0 || y >= area.bottom() {
        return;
    }
    for x in area.x..area.right() {
        buf[(x, y)].set_symbol("-").set_style(Style::new().dim());
    }
}

fn side_width(width: u16) -> u16 {
    if width >= MIN_WIDE_WIDTH {
        WIDE_SIDE_WIDTH
    } else {
        0
    }
}

fn right_rail_width(width: u16, side_width: u16) -> u16 {
    let center_width = width.saturating_sub(side_width + RIGHT_RAIL_WIDTH);
    if width >= MIN_RIGHT_RAIL_WIDTH && center_width >= 64 {
        RIGHT_RAIL_WIDTH
    } else {
        0
    }
}

fn side_width_for_state(width: u16, sidebar: RedesignSidebarState) -> u16 {
    let default_width = side_width(width);
    if default_width > 0 {
        return default_width;
    }
    if sidebar.focused() && width >= MIN_COMPACT_SIDEBAR_WIDTH {
        COMPACT_SIDE_WIDTH
    } else {
        0
    }
}

fn permission_profile_label(profile: &PermissionProfile) -> &'static str {
    match profile {
        PermissionProfile::Managed { .. } => "managed",
        PermissionProfile::Disabled => "disabled",
        PermissionProfile::External { .. } => "external",
    }
}

fn truncate_text(text: &str, max_width: u16) -> String {
    let max_width = max_width as usize;
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let mut remaining = max_width.saturating_sub(3);
    let mut out = String::new();
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if ch_width > remaining {
            break;
        }
        out.push(ch);
        remaining -= ch_width;
    }
    out.push_str("...");
    out
}

fn truncate_line(line: Line<'static>, max_width: usize) -> Line<'static> {
    if line_width(&line) <= max_width {
        return line;
    }
    if max_width == 0 {
        return Line::from("");
    }

    let mut remaining = max_width.saturating_sub(3);
    let mut spans = Vec::new();
    for span in line.spans {
        if remaining == 0 {
            break;
        }
        let width = UnicodeWidthStr::width(span.content.as_ref());
        if width <= remaining {
            remaining -= width;
            spans.push(span);
            continue;
        }

        let mut end = 0;
        for (idx, ch) in span.content.char_indices() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if ch_width > remaining {
                break;
            }
            remaining -= ch_width;
            end = idx + ch.len_utf8();
        }
        if end > 0 {
            spans.push(Span::styled(span.content[..end].to_string(), span.style));
        }
        break;
    }
    spans.push("...".dim());
    Line::from(spans)
}

fn line_width(line: &Line<'_>) -> usize {
    line.spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_fixture(width: u16, height: u16) -> String {
        render_fixture_with_sidebar(width, height, RedesignSidebarState::default())
    }

    fn render_fixture_with_sidebar(
        width: u16,
        height: u16,
        sidebar: RedesignSidebarState,
    ) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                let context = RedesignChromeContext::fixture();
                let layout = layout_for_dimensions_with_side(
                    area,
                    side_width_for_state(area.width, sidebar),
                    COMPOSER_ROWS,
                );
                let blocks = vec![
                    TranscriptBlock {
                        role: TranscriptRole::User,
                        lines: vec![Line::from(
                            "Let's redesign the TUI to be more intuitive for CLI users."
                                .to_string(),
                        )],
                    },
                    TranscriptBlock {
                        role: TranscriptRole::Codex,
                        lines: vec![Line::from(
                            "Agreed. I'll focus on information density and clear keyboard shortcuts."
                                .to_string(),
                        )],
                    },
                ];
                let system_blocks = vec![
                    SystemRailBlock {
                        title: "THINKING",
                        lines: vec![Line::from("Checking the layout and keyboard flow.")],
                    },
                    SystemRailBlock {
                        title: "SYSTEM",
                        lines: vec![Line::from("Tool output and notices stay out of chat.")],
                    },
                ];

                render_background(area, frame.buffer_mut());
                render_chrome(area, frame.buffer_mut(), &context, sidebar);
                render_transcript(
                    layout.transcript,
                    frame.buffer_mut(),
                    &blocks,
                    /*scroll_offset*/ 0,
                );
                render_system_rail(layout.right, frame.buffer_mut(), &context, &system_blocks);
                render_composer(
                    layout.composer,
                    frame.buffer_mut(),
                    "",
                    composer_activity_indicator(&context),
                );
            })
            .expect("draw");
        terminal.backend().to_string()
    }

    fn render_transcript_fixture(scroll_offset: usize) -> String {
        let mut terminal = Terminal::new(TestBackend::new(40, 6)).expect("terminal");
        let blocks = vec![TranscriptBlock {
            role: TranscriptRole::Codex,
            lines: (0..12)
                .map(|idx| Line::from(format!("line {idx}")))
                .collect(),
        }];

        terminal
            .draw(|frame| {
                render_transcript(frame.area(), frame.buffer_mut(), &blocks, scroll_offset);
            })
            .expect("draw");
        terminal.backend().to_string()
    }

    #[test]
    fn wide_chrome_snapshot() {
        assert_snapshot!("redesign_chrome_wide_100x24", render_fixture(100, 24));
    }

    #[test]
    fn three_column_chrome_snapshot() {
        assert_snapshot!(
            "redesign_chrome_three_column_132x24",
            render_fixture(132, 24)
        );
    }

    #[test]
    fn narrow_chrome_omits_side_nav_snapshot() {
        assert_snapshot!("redesign_chrome_narrow_72x18", render_fixture(72, 18));
    }

    #[test]
    fn focused_sidebar_snapshot() {
        let mut sidebar = RedesignSidebarState::default();
        sidebar.toggle_focus(/*chat_count*/ 3);
        sidebar.select_next(/*chat_count*/ 3);

        assert_snapshot!(
            "redesign_chrome_focused_sidebar_72x18",
            render_fixture_with_sidebar(72, 18, sidebar)
        );
    }

    #[test]
    fn transcript_scroll_offset_shows_older_lines() {
        let bottom = render_transcript_fixture(/*scroll_offset*/ 0);
        let older = render_transcript_fixture(/*scroll_offset*/ 6);

        assert!(bottom.contains("line 11"));
        assert!(!bottom.contains("line 2"));
        assert!(older.contains("line 2"));
        assert!(!older.contains("line 11"));
    }

    #[test]
    fn bubble_lines_render_visible_frame() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            lines: vec![Line::from("Bubble this message.")],
        };

        let lines = bubble_lines(&block, /*area_width*/ 80)
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            lines,
            vec![
                " Codex".to_string(),
                " ╭──────────────────────╮".to_string(),
                " │ Bubble this message. │".to_string(),
                " ╰──────────────────────╯".to_string(),
            ]
        );
    }

    #[test]
    fn product_version_label_uses_codex_cli_version() {
        assert_eq!(
            product_version_label("CODEX_CLI"),
            format!("CODEX_CLI v{CODEX_CLI_VERSION}")
        );
    }

    #[test]
    fn token_usage_label_shows_input_output_and_total() {
        assert_eq!(
            token_usage_label(&TokenUsage {
                input_tokens: 12_345,
                cached_input_tokens: 0,
                output_tokens: 6_789,
                reasoning_output_tokens: 345,
                total_tokens: 19_134,
            }),
            "in 12.3K / out 6.79K / total 19.1K"
        );
    }

    #[test]
    fn composer_activity_indicator_only_renders_while_working() {
        let mut context = RedesignChromeContext::fixture();
        assert_eq!(
            composer_activity_indicator(&context)
                .expect("working indicator")
                .content
                .as_ref(),
            "⠋"
        );

        context.working = false;

        assert_eq!(composer_activity_indicator(&context), None);
    }

    #[test]
    fn content_width_accounts_for_side_nav() {
        assert_eq!(content_width_for_terminal_width(100), 76);
        assert_eq!(content_width_for_terminal_width(132), 78);
        assert_eq!(content_width_for_terminal_width(72), 72);
    }

    #[tokio::test]
    async fn context_uses_runtime_model_and_reasoning() {
        let mut app = crate::app::test_support::make_test_app().await;

        app.chat_widget.set_model("gpt-redesign");
        app.chat_widget
            .set_reasoning_effort(Some(ReasoningEffortConfig::High));

        let context = RedesignChromeContext::from_app(&app);

        assert_eq!(context.model, "gpt-redesign");
        assert_eq!(context.reasoning, "high");
    }

    #[tokio::test]
    async fn transcript_routes_system_and_reasoning_to_right_rail() {
        let mut app = crate::app::test_support::make_test_app().await;
        let cwd = app.config.cwd.clone();
        app.transcript_cells = vec![
            std::sync::Arc::new(history_cell::new_user_prompt(
                "Run the tests".to_string(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )),
            std::sync::Arc::new(history_cell::ReasoningSummaryCell::new(
                "thinking".to_string(),
                "checking the changed crates".to_string(),
                cwd.as_path(),
                /*transcript_only*/ false,
            )),
            std::sync::Arc::new(history_cell::PlainHistoryCell::new(vec![
                "system notice".into(),
            ])),
            std::sync::Arc::new(history_cell::AgentMarkdownCell::new(
                "Tests passed".to_string(),
                cwd.as_path(),
            )),
        ];

        let blocks = transcript_blocks(&app, /*width*/ 80);
        let rail = system_rail_blocks(&app, /*width*/ 30);
        let rail_summary = rail
            .iter()
            .map(|block| {
                (
                    block.title,
                    block
                        .lines
                        .iter()
                        .map(plain_line_text)
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            blocks,
            vec![
                TranscriptBlock {
                    role: TranscriptRole::User,
                    lines: vec![Line::from("Run the tests")],
                },
                TranscriptBlock {
                    role: TranscriptRole::Codex,
                    lines: vec![Line::from("Tests passed")],
                },
            ]
        );
        assert_eq!(
            rail_summary,
            vec![
                ("THINKING", "checking the changed crates".to_string()),
                ("SYSTEM", "system notice".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn final_only_transcript_filters_system_and_reasoning_cells() {
        let mut app = crate::app::test_support::make_test_app().await;
        let cwd = app.config.cwd.clone();
        app.redesign_final_only_transcript = true;
        app.transcript_cells = vec![
            std::sync::Arc::new(history_cell::new_user_prompt(
                "Run the tests".to_string(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )),
            std::sync::Arc::new(history_cell::ReasoningSummaryCell::new(
                "thinking".to_string(),
                "hidden reasoning".to_string(),
                cwd.as_path(),
                /*transcript_only*/ false,
            )),
            std::sync::Arc::new(history_cell::PlainHistoryCell::new(vec![
                "system hidden".into(),
            ])),
            std::sync::Arc::new(history_cell::AgentMarkdownCell::new(
                "Tests passed".to_string(),
                cwd.as_path(),
            )),
        ];

        let blocks = transcript_blocks(&app, /*width*/ 80);
        let rail = system_rail_blocks(&app, /*width*/ 30);

        assert_eq!(
            blocks,
            vec![
                TranscriptBlock {
                    role: TranscriptRole::User,
                    lines: vec![Line::from("Run the tests")],
                },
                TranscriptBlock {
                    role: TranscriptRole::Codex,
                    lines: vec![Line::from("Tests passed")],
                },
            ]
        );
        assert_eq!(rail, Vec::<SystemRailBlock>::new());
    }

    #[tokio::test]
    async fn transcript_blocks_merge_stream_continuations_into_one_bubble() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.transcript_cells = vec![
            std::sync::Arc::new(history_cell::AgentMessageCell::new(
                vec![Line::from("First line")],
                /*is_first_line*/ true,
            )),
            std::sync::Arc::new(history_cell::AgentMessageCell::new(
                vec![Line::from("second line")],
                /*is_first_line*/ false,
            )),
        ];

        let blocks = transcript_blocks(&app, /*width*/ 80);

        assert_eq!(
            blocks,
            vec![TranscriptBlock {
                role: TranscriptRole::Codex,
                lines: vec![Line::from("First line"), Line::from("second line")],
            }]
        );
    }

    #[tokio::test]
    async fn transcript_blocks_preserve_line_span_formatting() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.transcript_cells = vec![std::sync::Arc::new(history_cell::AgentMessageCell::new(
            vec![Line::from(vec!["Important".bold(), " detail".into()])],
            /*is_first_line*/ true,
        ))];

        let blocks = transcript_blocks(&app, /*width*/ 80);
        let first_span = &blocks[0].lines[0].spans[0];

        assert_eq!(first_span.content.as_ref(), "Important");
        assert!(
            first_span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD)
        );
    }
}
