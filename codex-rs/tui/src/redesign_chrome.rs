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
use std::time::Instant;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

mod background_terminals;
mod composer;
mod layout;
mod plan_window;
mod sidebar;
mod window;

use composer::COMPOSER_ROWS;
use composer::composer_desired_height;
use composer::render_composer;
#[cfg(test)]
use composer::render_composer_cursor;
use composer::render_work_status_line;
#[cfg(test)]
use composer::work_activity_indicator;
use layout::RedesignLayout;
use layout::available_chat_body_height;
use layout::layout_for_dimensions;
use layout::layout_for_dimensions_with_side;
use layout::right_rail_width;
use layout::side_width_for_state;
#[cfg(test)]
use plan_window::plan_window_rect;
use plan_window::render_plan_window_from_app;
pub(crate) use sidebar::RedesignChatActivity;
pub(crate) use sidebar::RedesignChatListEntry;
pub(crate) use sidebar::RedesignSidebarItem;
pub(crate) use sidebar::RedesignSidebarSelection;
pub(crate) use sidebar::RedesignSidebarState;
use sidebar::render_side_nav;

const SYSTEM_RAIL_EVENT_LINE_LIMIT: usize = 2;

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
    assistant_label: String,
    working: bool,
    animations_enabled: bool,
    work_started_at: Option<Instant>,
    work_status_line: Option<Line<'static>>,
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
            .unwrap_or_else(|| permission_profile_label(permission_profile).to_string());
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
            assistant_label: app.redesign_assistant_transcript_label(),
            working: app.chat_widget.redesign_task_running(),
            animations_enabled: app.chat_widget.redesign_animations_enabled(),
            work_started_at: app.chat_widget.redesign_work_started_at(),
            work_status_line: app.chat_widget.redesign_work_status_line(),
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
            assistant_label: "Codex".to_string(),
            working: true,
            animations_enabled: true,
            work_started_at: None,
            work_status_line: Some(Line::from(vec![
                "⠋".cyan(),
                " ".into(),
                "Working".into(),
                " (0s • esc to interrupt)".dim(),
            ])),
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
    speaker_label: Option<String>,
    lines: Vec<Line<'static>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TranscriptBlockInput {
    role: TranscriptRole,
    lines: Vec<Line<'static>>,
    is_stream_continuation: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SystemRailBlock {
    title: &'static str,
    lines: Vec<Line<'static>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TranscriptDisplayWindow {
    lines: Vec<Line<'static>>,
    hidden_above: bool,
    hidden_below: bool,
}

pub(crate) fn render_app(area: Rect, buf: &mut Buffer, app: &App) -> AppFrameRender {
    let context = RedesignChromeContext::from_app(app);
    let legacy_bottom_pane = app.chat_widget.redesign_should_render_bottom_pane();
    let layout = layout_for(area, app, legacy_bottom_pane);

    render_background(area, buf);
    render_chrome(area, buf, &context, app.redesign_sidebar_state);
    app.chat_widget
        .redesign_schedule_work_indicator_frame_if_needed();
    let route_system_cells_to_rail = should_route_system_cells_to_rail(layout.right.width, app);
    render_transcript_from_app(
        layout.transcript,
        buf,
        app,
        route_system_cells_to_rail,
        app.redesign_transcript_scroll,
        &context.assistant_label,
    );
    render_system_rail_from_app(layout.right, buf, &context, app);
    if legacy_bottom_pane {
        app.chat_widget
            .render_redesign_bottom_pane(layout.composer, buf);
        render_plan_window_from_app(layout.main, buf, app);
        render_background_terminal_window_from_app(layout.main, buf, app);
        return AppFrameRender {
            cursor_pos: app
                .chat_widget
                .redesign_bottom_pane_cursor_pos(layout.composer),
            cursor_style: app
                .chat_widget
                .redesign_bottom_pane_cursor_style(layout.composer),
        };
    }

    let draft = app.chat_widget.redesign_composer_text();
    let queued_messages = app.chat_widget.redesign_queued_message_texts();
    let work_status_line = render_work_status_line(&context);
    let cursor_pos = render_composer(
        layout.composer,
        buf,
        &draft,
        app.chat_widget.redesign_composer_cursor(),
        &queued_messages,
        work_status_line.as_ref(),
    );
    render_plan_window_from_app(layout.main, buf, app);
    render_background_terminal_window_from_app(layout.main, buf, app);
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
    let blocks = transcript_blocks(
        app,
        layout.transcript.width,
        should_route_system_cells_to_rail(layout.right.width, app),
    );
    let assistant_label = app.redesign_assistant_transcript_label();
    transcript_scroll_limit_for_blocks_with_assistant_label(
        layout.transcript,
        &blocks,
        &assistant_label,
    )
}

pub(crate) fn background_terminal_window_scroll_limit(area: Rect, app: &App) -> usize {
    if area.is_empty() {
        return 0;
    }
    let layout = layout_for(
        area,
        app,
        app.chat_widget.redesign_should_render_bottom_pane(),
    );
    let terminals = app.chat_widget.redesign_background_terminals();
    let selected_idx = app.redesign_terminal_window_selected_for_active_chat(terminals.len());
    let expanded_idx = app.redesign_terminal_window_expanded_for_active_chat(terminals.len());
    background_terminals::scroll_limit(layout.main, &terminals, selected_idx, expanded_idx)
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
    let inline_identity = (side_width == 0).then(|| product_version_label(&context.product));
    render_chat_bar_with_identity(layout.chat_header, buf, context, inline_identity.as_deref());
    draw_horizontal_rule(layout.chat_separator, buf, layout.chat_separator.y);
    render_side_nav(layout.side, buf, context, sidebar);
    render_footer_aligned(
        layout.footer,
        buf,
        context,
        layout.side.width,
        layout.main.width,
    );
}

fn layout_for(area: Rect, app: &App, legacy_bottom_pane: bool) -> RedesignLayout {
    if area.is_empty() {
        return layout_for_dimensions(area, COMPOSER_ROWS);
    }

    let side_width = side_width_for_state(area.width, app.redesign_sidebar_state);
    let main_width = area
        .width
        .saturating_sub(side_width + right_rail_width(area.width, side_width));
    let available_chat_body_height = available_chat_body_height(area);
    let composer_height = if legacy_bottom_pane {
        app.chat_widget
            .redesign_bottom_pane_desired_height(main_width)
            .min(available_chat_body_height.max(1))
            .max(1)
    } else {
        let draft = app.chat_widget.redesign_composer_text();
        let queued_messages = app.chat_widget.redesign_queued_message_texts();
        let desired_height = composer_desired_height(
            main_width,
            &draft,
            &queued_messages,
            app.chat_widget.redesign_task_running(),
        );
        desired_height
            .min(available_chat_body_height)
            .max(COMPOSER_ROWS.min(available_chat_body_height))
    };

    layout_for_dimensions_with_side(area, side_width, composer_height)
}

fn render_background(area: Rect, buf: &mut Buffer) {
    buf.set_style(area, Style::default().bg(Color::Reset));
}

fn render_background_terminal_window_from_app(area: Rect, buf: &mut Buffer, app: &App) {
    if !app.redesign_terminal_window_open_for_active_chat() {
        return;
    }
    let terminals = app.chat_widget.redesign_background_terminals();
    let selected_idx = app.redesign_terminal_window_selected_for_active_chat(terminals.len());
    let expanded_idx = app.redesign_terminal_window_expanded_for_active_chat(terminals.len());
    background_terminals::render_window(
        area,
        buf,
        &terminals,
        selected_idx,
        expanded_idx,
        app.redesign_terminal_window_scroll_for_active_chat(),
    );
}

pub(super) fn product_version_label(product: &str) -> String {
    if is_source_build_version_label(CODEX_CLI_VERSION) {
        format!("{product} dev")
    } else {
        format!("{product} v{CODEX_CLI_VERSION}")
    }
}

fn is_source_build_version_label(version: &str) -> bool {
    version.trim() == "0.0.0"
}

#[cfg(test)]
fn render_chat_bar(area: Rect, buf: &mut Buffer, context: &RedesignChromeContext) {
    render_chat_bar_with_identity(area, buf, context, None);
}

fn render_chat_bar_with_identity(
    area: Rect,
    buf: &mut Buffer,
    context: &RedesignChromeContext,
    identity: Option<&str>,
) {
    if area.is_empty() {
        return;
    }

    let model = format!("{} {}", context.model, context.reasoning);
    let context_left = compact_context_left_label(&context.context_left);
    let token_usage = chat_header_token_usage_label(area.width, context);

    let mut primary_spans = Vec::new();
    if let Some(identity) = identity {
        primary_spans.push(Span::from(identity.to_string()).magenta().bold());
        primary_spans.push("  ".into());
    }
    push_chat_header_item(&mut primary_spans, "Model: ", model);
    primary_spans.push(" ".into());
    push_chat_header_item(&mut primary_spans, "ctx: ", context_left);
    primary_spans.push(" ".into());
    push_chat_header_item(&mut primary_spans, "tokens: ", token_usage);
    if let Some(pricing) = &context.pricing {
        primary_spans.push(" ".into());
        push_chat_header_item(&mut primary_spans, "price: ", pricing.clone());
    }
    render_line(area, buf, area.y, Line::from(primary_spans));

    if area.height > 1 {
        let mut policy_spans = Vec::new();
        push_chat_header_item(
            &mut policy_spans,
            "permissions: ",
            context.permissions.clone(),
        );
        policy_spans.push(" ".into());
        push_chat_header_item(&mut policy_spans, "approval: ", context.approval.clone());
        render_line(
            area,
            buf,
            area.y.saturating_add(1),
            Line::from(policy_spans),
        );
    }
}

fn push_chat_header_item(
    spans: &mut Vec<Span<'static>>,
    label: &'static str,
    value: impl Into<String>,
) {
    spans.push(label.dim());
    spans.push(chat_info_span(value));
}

fn chat_info_span(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::new().fg(Color::Cyan))
}

fn compact_context_left_label(context_left: &str) -> String {
    let without_prefix = context_left
        .strip_prefix("context ")
        .unwrap_or(context_left);
    without_prefix
        .strip_suffix(" left")
        .unwrap_or(without_prefix)
        .to_string()
}

fn chat_header_token_usage_label(width: u16, context: &RedesignChromeContext) -> String {
    let detailed = detailed_token_usage_label(&context.token_usage);
    if chat_header_primary_width(context, &detailed) <= width as usize {
        detailed
    } else {
        compact_token_usage_label(&context.token_usage)
    }
}

fn detailed_token_usage_label(token_usage: &TokenUsage) -> String {
    let detailed = token_usage.to_string();
    detailed
        .strip_prefix("Token usage: ")
        .unwrap_or(detailed.as_str())
        .to_string()
}

fn compact_token_usage_label(token_usage: &TokenUsage) -> String {
    format_tokens_compact(token_usage.blended_total())
}

fn chat_header_primary_width(context: &RedesignChromeContext, token_usage: &str) -> usize {
    let model = format!("{} {}", context.model, context.reasoning);
    let context_left = compact_context_left_label(&context.context_left);
    let mut width = UnicodeWidthStr::width("Model: ")
        + UnicodeWidthStr::width(model.as_str())
        + 1
        + UnicodeWidthStr::width("ctx: ")
        + UnicodeWidthStr::width(context_left.as_str())
        + 1
        + UnicodeWidthStr::width("tokens: ")
        + UnicodeWidthStr::width(token_usage);
    if let Some(pricing) = &context.pricing {
        width += 1 + UnicodeWidthStr::width("price: ") + UnicodeWidthStr::width(pricing.as_str());
    }
    width
}

#[cfg(test)]
fn render_transcript(
    area: Rect,
    buf: &mut Buffer,
    blocks: &[TranscriptBlock],
    scroll_offset: usize,
) {
    render_transcript_with_assistant_label(area, buf, blocks, scroll_offset, "Codex");
}

#[cfg(test)]
fn render_transcript_with_assistant_label(
    area: Rect,
    buf: &mut Buffer,
    blocks: &[TranscriptBlock],
    scroll_offset: usize,
    assistant_label: &str,
) {
    if area.is_empty() {
        return;
    }

    let lines = transcript_display_lines_with_assistant_label(blocks, area.width, assistant_label);
    let scroll_limit = lines.len().saturating_sub(area.height as usize);
    let scroll_offset = scroll_offset.min(scroll_limit);
    let window = visible_tail_window(lines, area.height as usize, scroll_offset);
    Paragraph::new(window.lines).render(area, buf);
    render_transcript_scrollbar(
        area,
        buf,
        window.hidden_above,
        window.hidden_below,
        scroll_offset,
        Some(scroll_limit),
    );
}

fn render_transcript_from_app(
    area: Rect,
    buf: &mut Buffer,
    app: &App,
    route_system_cells_to_rail: bool,
    scroll_offset: usize,
    assistant_label: &str,
) {
    if area.is_empty() {
        return;
    }

    let window = transcript_display_window_from_app(
        app,
        area.width,
        area.height as usize,
        route_system_cells_to_rail,
        scroll_offset,
        assistant_label,
    );
    Paragraph::new(window.lines).render(area, buf);
    render_transcript_scrollbar(
        area,
        buf,
        window.hidden_above,
        window.hidden_below,
        scroll_offset,
        None,
    );
}

fn transcript_scroll_limit_for_blocks_with_assistant_label(
    area: Rect,
    blocks: &[TranscriptBlock],
    assistant_label: &str,
) -> usize {
    if area.is_empty() {
        return 0;
    }

    transcript_display_lines_with_assistant_label(blocks, area.width, assistant_label)
        .len()
        .saturating_sub(area.height as usize)
}

fn transcript_display_window_from_app(
    app: &App,
    width: u16,
    height: usize,
    route_system_cells_to_rail: bool,
    scroll_offset: usize,
    assistant_label: &str,
) -> TranscriptDisplayWindow {
    let target_lines = height.saturating_add(scroll_offset).saturating_add(1);
    let content_width = width.saturating_sub(4).max(1);
    let mut inputs_rev = Vec::new();
    let mut stopped_early = false;
    let mut lines = Vec::new();

    if let Some(input) = active_final_transcript_input(app, content_width) {
        inputs_rev.push(input);
        lines = transcript_display_lines_from_reverse_inputs(&inputs_rev, width, assistant_label);
    }

    if lines.len() < target_lines
        && !app.redesign_final_only_transcript
        && !route_system_cells_to_rail
        && let Some(input) = active_system_transcript_input(app, content_width)
    {
        inputs_rev.push(input);
        lines = transcript_display_lines_from_reverse_inputs(&inputs_rev, width, assistant_label);
    }

    if lines.len() < target_lines {
        for cell in app.transcript_cells.iter().rev() {
            let cell = cell.as_ref();
            let Some(input) =
                transcript_input_for_cell(app, cell, content_width, route_system_cells_to_rail)
            else {
                continue;
            };
            inputs_rev.push(input);
            lines =
                transcript_display_lines_from_reverse_inputs(&inputs_rev, width, assistant_label);
            if lines.len() >= target_lines {
                stopped_early = true;
                break;
            }
        }
    }

    if inputs_rev.is_empty() {
        lines = transcript_display_lines_with_assistant_label(&[], width, assistant_label);
    }

    let mut window = visible_tail_window(lines, height, scroll_offset);
    window.hidden_above |= stopped_early;
    window
}

fn transcript_display_lines_from_reverse_inputs(
    inputs_rev: &[TranscriptBlockInput],
    width: u16,
    assistant_label: &str,
) -> Vec<Line<'static>> {
    let mut blocks = Vec::new();
    for input in inputs_rev.iter().rev() {
        push_transcript_block(
            &mut blocks,
            input.role,
            input.lines.clone(),
            input.is_stream_continuation,
        );
    }
    transcript_display_lines_with_assistant_label(&blocks, width, assistant_label)
}

fn transcript_display_lines_with_assistant_label(
    blocks: &[TranscriptBlock],
    width: u16,
    assistant_label: &str,
) -> Vec<Line<'static>> {
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
            lines.extend(bubble_lines_with_assistant_label(
                block,
                width,
                assistant_label,
            ));
        }
    }
    lines
}

fn visible_tail_window(
    lines: Vec<Line<'static>>,
    height: usize,
    scroll_offset: usize,
) -> TranscriptDisplayWindow {
    let visible_start = lines
        .len()
        .saturating_sub(height.saturating_add(scroll_offset));
    let visible_end = visible_start.saturating_add(height).min(lines.len());
    let hidden_above = visible_start > 0;
    let hidden_below = visible_end < lines.len();
    let lines = lines
        .into_iter()
        .skip(visible_start)
        .take(visible_end.saturating_sub(visible_start))
        .collect();
    TranscriptDisplayWindow {
        lines,
        hidden_above,
        hidden_below,
    }
}

fn render_transcript_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    hidden_above: bool,
    hidden_below: bool,
    scroll_offset: usize,
    exact_scroll_limit: Option<usize>,
) {
    if area.width <= 2 || (!hidden_above && !hidden_below) {
        return;
    }

    let scrollbar_x = area.right().saturating_sub(1);
    for y in area.y..area.bottom() {
        buf[(scrollbar_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }

    let thumb_y =
        if let Some(scroll_limit) = exact_scroll_limit.filter(|scroll_limit| *scroll_limit > 0) {
            let scroll_offset = scroll_offset.min(scroll_limit);
            let thumb_range = area.height.saturating_sub(1) as usize;
            let thumb_offset = (scroll_limit - scroll_offset) * thumb_range / scroll_limit;
            area.y
                .saturating_add(thumb_offset as u16)
                .min(area.bottom().saturating_sub(1))
        } else {
            match (hidden_above, hidden_below) {
                (true, false) => area.bottom().saturating_sub(1),
                (false, true) => area.y,
                (true, true) => area.y.saturating_add(area.height.saturating_sub(1) / 2),
                (false, false) => area.y,
            }
        };
    buf[(scrollbar_x, thumb_y)].set_symbol("#");
}

#[cfg(test)]
fn render_system_rail(
    area: Rect,
    buf: &mut Buffer,
    _context: &RedesignChromeContext,
    blocks: &[SystemRailBlock],
) {
    let Some((content, header, body_capacity)) = system_rail_frame(area, buf) else {
        return;
    };
    let body = system_rail_display_lines(blocks, content.width);
    render_system_rail_lines(content, header, body, body_capacity, buf);
}

fn render_system_rail_from_app(
    area: Rect,
    buf: &mut Buffer,
    _context: &RedesignChromeContext,
    app: &App,
) {
    let Some((content, header, body_capacity)) = system_rail_frame(area, buf) else {
        return;
    };
    let body = system_rail_tail_display_lines(app, content.width, body_capacity);
    render_system_rail_lines(content, header, body, body_capacity, buf);
}

fn system_rail_frame(area: Rect, buf: &mut Buffer) -> Option<(Rect, Vec<Line<'static>>, usize)> {
    if area.is_empty() {
        return None;
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
        return None;
    }

    let header = vec![
        Line::from(vec![" ".into(), "ACTIVITY".cyan().bold()]),
        Line::from(vec![" ".into(), "latest messages".dim()]),
        Line::from(""),
    ];
    let body_capacity = content.height.saturating_sub(header.len() as u16) as usize;
    Some((content, header, body_capacity))
}

fn render_system_rail_lines(
    content: Rect,
    mut lines: Vec<Line<'static>>,
    body: Vec<Line<'static>>,
    body_capacity: usize,
    buf: &mut Buffer,
) {
    let body_start = body.len().saturating_sub(body_capacity);
    lines.extend(body.into_iter().skip(body_start).take(body_capacity));

    Paragraph::new(lines).render(content, buf);
}

fn system_rail_display_lines(blocks: &[SystemRailBlock], width: u16) -> Vec<Line<'static>> {
    if blocks.is_empty() {
        return vec![Line::from(vec![" ".into(), "No activity yet".dim()])];
    }

    let mut lines = Vec::new();
    for block in blocks {
        lines.extend(system_rail_event_lines(block, width));
    }
    lines
}

fn system_rail_event_lines(block: &SystemRailBlock, width: u16) -> Vec<Line<'static>> {
    let wrap_width = width.saturating_sub(4).max(1) as usize;
    let mut lines = vec![Line::from(vec![
        " ".into(),
        "•".cyan(),
        " ".into(),
        block.title.cyan().bold(),
    ])];
    let mut rendered_content_lines = 0usize;
    let mut truncated = false;

    'content: for line in &block.lines {
        for wrapped in
            adaptive_wrap_lines(std::iter::once(line.clone()), RtOptions::new(wrap_width))
        {
            if rendered_content_lines >= SYSTEM_RAIL_EVENT_LINE_LIMIT {
                truncated = true;
                break 'content;
            }
            let mut spans = vec!["   ".dim()];
            spans.extend(wrapped.spans);
            lines.push(Line::from(spans));
            rendered_content_lines += 1;
        }
    }

    if truncated {
        lines.push(Line::from(vec!["   ".dim(), "...".dim()]));
    }

    lines
}

fn system_rail_tail_display_lines(
    app: &App,
    width: u16,
    body_capacity: usize,
) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    if app.redesign_final_only_transcript {
        return vec![Line::from(vec![" ".into(), "No activity yet".dim()])];
    }

    if body_capacity == 0 {
        return Vec::new();
    }

    let target_lines = body_capacity.saturating_add(1);
    let content_width = width.saturating_sub(4).max(1);
    let mut blocks_rev = Vec::new();
    let mut lines = Vec::new();

    if let Some(active) = app
        .chat_widget
        .redesign_active_system_display(content_width)
    {
        let lines_for_block = display_lines_to_content(active.lines);
        if !lines_for_block.is_empty() {
            blocks_rev.push(SystemRailBlock {
                title: "ACTIVE",
                lines: lines_for_block,
            });
            lines = system_rail_display_lines_from_reverse_blocks(&blocks_rev, width);
        }
    }

    if lines.len() < target_lines {
        for cell in app.transcript_cells.iter().rev() {
            let cell = cell.as_ref();
            let Some(block) = system_rail_block_for_cell(cell, content_width) else {
                continue;
            };
            blocks_rev.push(block);
            lines = system_rail_display_lines_from_reverse_blocks(&blocks_rev, width);
            if lines.len() >= target_lines {
                break;
            }
        }
    }

    if blocks_rev.is_empty() {
        return vec![Line::from(vec![" ".into(), "No activity yet".dim()])];
    }
    lines
}

fn system_rail_display_lines_from_reverse_blocks(
    blocks_rev: &[SystemRailBlock],
    width: u16,
) -> Vec<Line<'static>> {
    let blocks = blocks_rev.iter().rev().cloned().collect::<Vec<_>>();
    system_rail_display_lines(&blocks, width)
}

fn system_rail_block_for_cell(
    cell: &dyn HistoryCell,
    content_width: u16,
) -> Option<SystemRailBlock> {
    if !is_system_rail_cell(cell) {
        return None;
    }

    let lines = cell_content_lines(cell, content_width);
    (!lines.is_empty()).then_some(SystemRailBlock {
        title: system_rail_title(cell),
        lines,
    })
}

#[cfg(test)]
fn render_footer(area: Rect, buf: &mut Buffer, context: &RedesignChromeContext) {
    render_footer_aligned(area, buf, context, 0, area.width);
}

fn render_footer_aligned(
    area: Rect,
    buf: &mut Buffer,
    context: &RedesignChromeContext,
    side_width: u16,
    main_width: u16,
) {
    if area.is_empty() {
        return;
    }

    if area.height == 1 {
        render_footer_shortcuts(area, buf, area.y, side_width, main_width);
        return;
    }

    render_footer_info(area, buf, area.y, context, side_width, main_width);
    render_footer_shortcuts(
        area,
        buf,
        area.bottom().saturating_sub(1),
        side_width,
        main_width,
    );
}

fn footer_info_line(width: u16, context: &RedesignChromeContext) -> Line<'static> {
    Span::from(compact_workspace_label(context, width))
        .dim()
        .into()
}

fn render_footer_info(
    area: Rect,
    buf: &mut Buffer,
    y: u16,
    context: &RedesignChromeContext,
    side_width: u16,
    main_width: u16,
) {
    if area.width == 0 || y >= area.bottom() {
        return;
    }

    let side_width = side_width.min(area.width);
    if side_width == 0 {
        render_line(area, buf, y, footer_info_line(area.width, context));
        return;
    }

    let divider_x = area.x.saturating_add(side_width.saturating_sub(1));
    if divider_x < area.right() {
        buf[(divider_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }

    let main_x = area.x.saturating_add(side_width);
    let main_width = main_width.min(area.right().saturating_sub(main_x));
    if main_width > 0 {
        let main_area = Rect::new(main_x, y, main_width, 1);
        render_line(main_area, buf, y, footer_info_line(main_width, context));
    }

    let right_divider_x = main_x.saturating_add(main_width);
    if right_divider_x < area.right() {
        buf[(right_divider_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }
}

fn footer_shortcuts_line(width: u16) -> Line<'static> {
    if width >= 100 {
        shortcut_line(&[
            ("Alt-H", "Help"),
            ("Alt-/", "Commands"),
            ("Alt-M", "Model"),
            ("Alt-P", "Plan"),
            ("Alt-T", "Terminal"),
            ("C-T", "Transcript"),
            ("C-C", "Exit"),
        ])
    } else if width >= 74 {
        shortcut_line(&[
            ("Alt-H", "Help"),
            ("Alt-/", "Cmds"),
            ("Alt-M", "Model"),
            ("Alt-P", "Plan"),
            ("Alt-T", "Term"),
            ("C-C", "Exit"),
        ])
    } else if width >= 64 {
        shortcut_line(&[
            ("Alt-H", "Help"),
            ("Alt-/", "Cmds"),
            ("Alt-T", "Term"),
            ("C-C", "Exit"),
        ])
    } else {
        shortcut_line(&[("Alt-H", "Help"), ("C-C", "Exit")])
    }
}

fn render_footer_shortcuts(area: Rect, buf: &mut Buffer, y: u16, side_width: u16, main_width: u16) {
    if area.width == 0 || y >= area.bottom() {
        return;
    }

    let side_width = side_width.min(area.width);
    if side_width == 0 {
        render_line(area, buf, y, footer_shortcuts_line(area.width));
        return;
    }

    let divider_x = area.x.saturating_add(side_width.saturating_sub(1));
    if divider_x < area.right() {
        buf[(divider_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }

    let main_x = area.x.saturating_add(side_width);
    let main_width = main_width.min(area.right().saturating_sub(main_x));
    if main_width > 0 {
        let main_area = Rect::new(main_x, y, main_width, 1);
        render_line(main_area, buf, y, footer_shortcuts_line(main_width));
    }

    let right_divider_x = main_x.saturating_add(main_width);
    if right_divider_x < area.right() {
        buf[(right_divider_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }
}

fn shortcut_line(items: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (idx, (key, label)) in items.iter().enumerate() {
        if idx > 0 {
            spans.push(" · ".dim());
        }
        spans.push((*key).cyan());
        spans.push(" ".into());
        spans.push((*label).dim());
    }
    Line::from(spans)
}

fn compact_workspace_label(context: &RedesignChromeContext, max_width: u16) -> String {
    let workspace = format!(
        "{} · {} · {} · {}",
        context.cwd, context.branch, context.changes, context.thread
    );
    truncate_text(&workspace, max_width)
}

fn should_route_system_cells_to_rail(right_rail_width: u16, app: &App) -> bool {
    right_rail_width > 0 && !app.redesign_final_only_transcript
}

fn active_final_transcript_input(app: &App, content_width: u16) -> Option<TranscriptBlockInput> {
    let active = app
        .chat_widget
        .redesign_active_final_output_display(content_width)?;
    let lines = display_lines_to_content(active.lines);
    (!lines.is_empty()).then_some(TranscriptBlockInput {
        role: TranscriptRole::Codex,
        lines,
        is_stream_continuation: active.is_stream_continuation,
    })
}

fn active_system_transcript_input(app: &App, content_width: u16) -> Option<TranscriptBlockInput> {
    let active = app
        .chat_widget
        .redesign_active_system_display(content_width)?;
    let lines = display_lines_to_content(active.lines);
    (!lines.is_empty()).then_some(TranscriptBlockInput {
        role: TranscriptRole::System,
        lines,
        is_stream_continuation: active.is_stream_continuation,
    })
}

fn transcript_input_for_cell(
    app: &App,
    cell: &dyn HistoryCell,
    content_width: u16,
    route_system_cells_to_rail: bool,
) -> Option<TranscriptBlockInput> {
    if is_startup_cell(cell)
        || app.redesign_final_only_transcript && !is_final_output_cell(cell)
        || route_system_cells_to_rail && is_system_rail_cell(cell)
    {
        return None;
    }

    let lines = cell_content_lines(cell, content_width);
    (!lines.is_empty()).then_some(TranscriptBlockInput {
        role: role_for_cell(cell),
        lines,
        is_stream_continuation: cell.is_stream_continuation(),
    })
}

fn transcript_blocks(
    app: &App,
    width: u16,
    route_system_cells_to_rail: bool,
) -> Vec<TranscriptBlock> {
    let mut blocks = Vec::new();
    let content_width = width.saturating_sub(4).max(1);
    for cell in &app.transcript_cells {
        let cell = cell.as_ref();
        if let Some(input) =
            transcript_input_for_cell(app, cell, content_width, route_system_cells_to_rail)
        {
            push_transcript_block(
                &mut blocks,
                input.role,
                input.lines,
                input.is_stream_continuation,
            );
        }
    }

    if !app.redesign_final_only_transcript
        && !route_system_cells_to_rail
        && let Some(input) = active_system_transcript_input(app, content_width)
    {
        push_transcript_block(
            &mut blocks,
            input.role,
            input.lines,
            input.is_stream_continuation,
        );
    }

    if let Some(input) = active_final_transcript_input(app, content_width) {
        push_transcript_block(
            &mut blocks,
            input.role,
            input.lines,
            input.is_stream_continuation,
        );
    }

    blocks
}

#[cfg(test)]
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
    mut lines: Vec<Line<'static>>,
    is_stream_continuation: bool,
) {
    let speaker_label = promote_speaker_prefix_to_label(role, &mut lines);
    if is_stream_continuation
        && let Some(previous) = blocks.last_mut()
        && previous.role == role
        && (speaker_label.is_none() || previous.speaker_label == speaker_label)
    {
        previous.lines.extend(lines);
        return;
    }

    blocks.push(TranscriptBlock {
        role,
        speaker_label,
        lines,
    });
}

fn promote_speaker_prefix_to_label(
    role: TranscriptRole,
    lines: &mut Vec<Line<'static>>,
) -> Option<String> {
    if role != TranscriptRole::Codex {
        return None;
    }

    let (speaker_label, bytes_to_drop) = speaker_prefix(lines.first()?)?;
    let stripped = drop_line_prefix(lines[0].clone(), bytes_to_drop);
    if plain_line_text(&stripped).trim().is_empty() && lines.len() > 1 {
        lines.remove(0);
    } else {
        lines[0] = stripped;
    }
    Some(speaker_label)
}

fn speaker_prefix(line: &Line<'_>) -> Option<(String, usize)> {
    let text = plain_line_text(line);
    let leading_bytes = text.len().saturating_sub(text.trim_start().len());
    let trimmed = &text[leading_bytes..];
    let colon_idx = trimmed.find(':')?;
    let after_colon = &trimmed[colon_idx + 1..];
    if after_colon
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_whitespace())
    {
        return None;
    }

    let label = trimmed[..colon_idx].trim();
    if !speaker_label_looks_like_agent(label) {
        return None;
    }

    let mut bytes_to_drop = leading_bytes + colon_idx + 1;
    if after_colon.starts_with(' ') {
        bytes_to_drop += 1;
    }
    Some((label.to_string(), bytes_to_drop))
}

fn speaker_label_looks_like_agent(label: &str) -> bool {
    let label = label.trim();
    if label.is_empty() || UnicodeWidthStr::width(label) > 64 {
        return false;
    }

    if let Some((name, role)) = label
        .strip_suffix(')')
        .and_then(|label| label.rsplit_once(" ("))
    {
        return !role.trim().is_empty() && speaker_name_looks_like_agent(name);
    }

    speaker_name_looks_like_agent(label) && !is_common_non_speaker_label(label)
}

fn speaker_name_looks_like_agent(name: &str) -> bool {
    let words = name.split_whitespace().collect::<Vec<_>>();
    !words.is_empty()
        && words.len() <= 2
        && !is_common_non_speaker_label(name)
        && words.iter().all(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return false;
            };
            first.is_uppercase()
                && chars.all(|ch| ch.is_alphabetic() || matches!(ch, '\'' | '-' | '.'))
        })
}

fn is_common_non_speaker_label(label: &str) -> bool {
    const COMMON_LABELS: &[&str] = &[
        "Action Items",
        "Answer",
        "Context",
        "Error",
        "Next Steps",
        "Note",
        "Notes",
        "Plan",
        "Question",
        "Reasoning",
        "Result",
        "Results",
        "Status",
        "Summary",
        "Todo",
        "Update",
        "Warning",
    ];

    COMMON_LABELS
        .iter()
        .any(|common_label| label.eq_ignore_ascii_case(common_label))
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
        trim_empty_edge_lines(cell.raw_lines())
    } else {
        display_lines_to_content(cell.display_lines(width))
    }
}

fn display_lines_to_content(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    trim_empty_edge_lines(
        lines
            .into_iter()
            .map(strip_legacy_prefix)
            .collect::<Vec<_>>(),
    )
}

fn trim_empty_edge_lines(mut out: Vec<Line<'static>>) -> Vec<Line<'static>> {
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
    for prefix in ["› ", "• ", "> "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return drop_line_prefix(line, text.len().saturating_sub(rest.len()));
        }
    }
    if text.starts_with("  ") {
        drop_line_prefix(line, /*bytes_to_drop*/ 2)
    } else {
        line
    }
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

#[cfg(test)]
fn bubble_lines(block: &TranscriptBlock, area_width: u16) -> Vec<Line<'static>> {
    bubble_lines_with_assistant_label(block, area_width, "Codex")
}

fn bubble_lines_with_assistant_label(
    block: &TranscriptBlock,
    area_width: u16,
    assistant_label: &str,
) -> Vec<Line<'static>> {
    let assistant_label = block.speaker_label.as_deref().unwrap_or(assistant_label);
    let viewport_width = area_width.saturating_sub(2).max(1) as usize;
    let mut wrapped = Vec::new();
    let content_lines = reflow_bubble_prose_lines(&block.lines);
    let contains_table_grid = content_lines.iter().any(is_prewrapped_table_grid_line);
    let max_bubble_width = bubble_max_width(block.role, viewport_width);
    let prose_wrap_width = max_bubble_width.saturating_sub(4).max(1);
    let full_inner_width = viewport_width.saturating_sub(4).max(1);
    let inner_width_limit = if contains_table_grid {
        full_inner_width
    } else {
        prose_wrap_width
    };
    let role_label = truncate_text(
        role_name(block.role, assistant_label),
        inner_width_limit.min(u16::MAX as usize) as u16,
    );

    for line in &content_lines {
        if plain_line_text(line).trim().is_empty() {
            wrapped.push(Line::from(""));
            continue;
        }
        if is_prewrapped_table_grid_line(line) && line_width(line) <= full_inner_width {
            wrapped.push(line.clone());
            continue;
        }
        let line_text = plain_line_text(line);
        let trimmed = line_text.trim_start();
        let leading_width = UnicodeWidthStr::width(&line_text[..line_text.len() - trimmed.len()]);
        let wrap_options = if let Some(marker_width) = list_marker_width(trimmed) {
            RtOptions::new(prose_wrap_width)
                .subsequent_indent(Line::from(" ".repeat(leading_width + marker_width)))
        } else {
            RtOptions::new(prose_wrap_width)
        };
        wrapped.extend(adaptive_wrap_lines(
            std::iter::once(line.clone()),
            wrap_options,
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
        .max(UnicodeWidthStr::width(role_label.as_str()))
        .min(inner_width_limit);
    let bubble_width = inner_width + 4;
    let prefix_width = bubble_prefix_width(block.role, viewport_width, bubble_width);
    let label_width = UnicodeWidthStr::width(role_label.as_str());
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
            role_label_span(block.role, role_label),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReflowRunKind {
    Prose,
    ListItem,
}

fn reflow_bubble_prose_lines(lines: &[Line<'static>]) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let mut prose_run: Option<(ReflowRunKind, Line<'static>)> = None;

    for line in lines {
        if starts_bubble_list_run(line) {
            if let Some((_, run)) = prose_run.take() {
                out.push(run);
            }
            prose_run = Some((ReflowRunKind::ListItem, line.clone()));
            continue;
        }

        if let Some((ReflowRunKind::ListItem, run)) = prose_run.as_mut()
            && is_list_continuation_line(line)
        {
            run.spans.push(" ".into());
            run.spans
                .extend(drop_line_prefix(line.clone(), leading_whitespace_bytes(line)).spans);
            continue;
        }

        let is_plain_prose = is_plain_prose_line(line);
        if is_plain_prose {
            if let Some((ReflowRunKind::Prose, run)) = prose_run.as_mut() {
                run.spans.push(" ".into());
                run.spans.extend(line.spans.clone());
                continue;
            }

            if let Some((_, run)) = prose_run.take() {
                out.push(run);
            }
            if starts_bubble_prose_run(line) {
                prose_run = Some((ReflowRunKind::Prose, line.clone()));
                continue;
            }
        }

        if let Some((_, run)) = prose_run.take() {
            out.push(run);
        }
        out.push(line.clone());
    }

    if let Some((_, run)) = prose_run {
        out.push(run);
    }

    out
}

fn starts_bubble_list_run(line: &Line<'_>) -> bool {
    list_marker_width(plain_line_text(line).trim_start()).is_some()
}

fn is_list_continuation_line(line: &Line<'_>) -> bool {
    let text = plain_line_text(line);
    text.chars().next().is_some_and(char::is_whitespace)
        && !text.trim().is_empty()
        && !is_structural_text_line(text.trim_start())
}

fn leading_whitespace_bytes(line: &Line<'_>) -> usize {
    let text = plain_line_text(line);
    text.len().saturating_sub(text.trim_start().len())
}

fn is_plain_prose_line(line: &Line<'_>) -> bool {
    let text = plain_line_text(line);
    if text.trim().is_empty() || text.chars().next().is_some_and(char::is_whitespace) {
        return false;
    }

    let trimmed = text.trim_start();
    !is_structural_text_line(trimmed)
}

fn starts_bubble_prose_run(line: &Line<'_>) -> bool {
    let text = plain_line_text(line);
    let trimmed = text.trim_end();
    trimmed.ends_with('-')
        || trimmed.contains(',')
        || trimmed
            .split_whitespace()
            .any(|token| token.chars().filter(|ch| ch.is_alphabetic()).count() >= 7)
}

fn is_structural_text_line(text: &str) -> bool {
    text.starts_with(['>', '|', '#'])
        || is_prewrapped_table_grid_text(text)
        || is_shell_command_text(text)
        || text.starts_with("```")
        || text.starts_with("---")
        || text.starts_with("***")
        || text.starts_with("- ")
        || text.starts_with("* ")
        || text.starts_with("+ ")
        || list_marker_width(text).is_some()
}

fn is_shell_command_text(text: &str) -> bool {
    let text = text.trim_start();
    if text.starts_with("$ ") || text.starts_with("./") || text.starts_with("../") {
        return true;
    }

    let Some(command) = first_command_token(text) else {
        return false;
    };
    let command = command.rsplit('/').next().unwrap_or(command);
    matches!(
        command,
        "awk"
            | "bash"
            | "bun"
            | "cargo"
            | "chmod"
            | "cp"
            | "curl"
            | "deno"
            | "docker"
            | "git"
            | "grep"
            | "just"
            | "kubectl"
            | "ls"
            | "mkdir"
            | "mv"
            | "nix"
            | "node"
            | "npm"
            | "pnpm"
            | "python"
            | "python3"
            | "rg"
            | "rm"
            | "sed"
            | "sh"
            | "uv"
            | "uvx"
            | "yarn"
            | "zsh"
    )
}

fn first_command_token(text: &str) -> Option<&str> {
    text.split_whitespace().find(|token| {
        !matches!(*token, "sudo" | "env" | "time" | "command") && !is_env_assignment_token(token)
    })
}

fn is_env_assignment_token(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && name.chars().any(|ch| ch.is_ascii_uppercase())
}

fn list_marker_width(text: &str) -> Option<usize> {
    if text.starts_with("- ") || text.starts_with("* ") || text.starts_with("+ ") {
        return Some(2);
    }

    let marker_end = text.find(". ")?;
    let marker = &text[..marker_end];
    (!marker.is_empty() && marker.chars().all(|ch| ch.is_ascii_digit()))
        .then_some(marker_end.saturating_add(2))
}

fn is_prewrapped_table_grid_line(line: &Line<'_>) -> bool {
    let text = plain_line_text(line);
    is_prewrapped_table_grid_text(text.trim_start())
}

fn is_prewrapped_table_grid_text(text: &str) -> bool {
    let table_text = strip_blockquote_markers(text);
    table_text.starts_with(['┌', '├', '└'])
        || table_text.starts_with('│') && table_text.contains('│')
        || is_markdown_table_separator_text(table_text)
}

fn is_markdown_table_separator_text(text: &str) -> bool {
    let mut has_separator = false;
    for ch in text.chars() {
        match ch {
            '━' => has_separator = true,
            ' ' => {}
            _ => return false,
        }
    }
    has_separator
}

fn strip_blockquote_markers(mut text: &str) -> &str {
    loop {
        text = text.trim_start();
        let Some(rest) = text.strip_prefix('>') else {
            return text;
        };
        text = rest.strip_prefix(' ').unwrap_or(rest);
    }
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

fn role_name(role: TranscriptRole, assistant_label: &str) -> &str {
    match role {
        TranscriptRole::User => "You",
        TranscriptRole::Codex => assistant_label,
        TranscriptRole::System => "System",
    }
}

fn role_label_span(role: TranscriptRole, label: String) -> Span<'static> {
    let label = Span::from(label);
    match role {
        TranscriptRole::User => label.cyan().bold(),
        TranscriptRole::Codex => label.magenta().bold(),
        TranscriptRole::System => label.dim().bold(),
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
    use codex_app_server_protocol::AskForApproval;
    use codex_protocol::ThreadId;
    use codex_protocol::config_types::ApprovalsReviewer;
    use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
    use codex_protocol::plan_tool::PlanItemArg;
    use codex_protocol::plan_tool::StepStatus;
    use codex_protocol::plan_tool::UpdatePlanArgs;
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    #[derive(Debug)]
    struct CountingHistoryCell {
        text: String,
        display_calls: Arc<AtomicUsize>,
    }

    impl HistoryCell for CountingHistoryCell {
        fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
            self.display_calls.fetch_add(1, Ordering::Relaxed);
            vec![Line::from(self.text.clone())]
        }

        fn raw_lines(&self) -> Vec<Line<'static>> {
            vec![Line::from(self.text.clone())]
        }
    }

    fn render_fixture(width: u16, height: u16) -> String {
        render_fixture_with_sidebar(width, height, RedesignSidebarState::default())
    }

    fn seed_test_thread(app: &mut App) {
        app.chat_widget
            .handle_thread_session(crate::session_state::ThreadSessionState {
                thread_id: ThreadId::new(),
                forked_from_id: None,
                fork_parent_title: None,
                thread_name: Some("Plan chat".to_string()),
                model: "test-model".to_string(),
                model_provider_id: "test-provider".to_string(),
                service_tier: None,
                approval_policy: AskForApproval::Never,
                approvals_reviewer: ApprovalsReviewer::User,
                permission_profile: PermissionProfile::read_only(),
                active_permission_profile: None,
                cwd: app.config.cwd.clone(),
                runtime_workspace_roots: Vec::new(),
                instruction_source_paths: Vec::new(),
                reasoning_effort: Some(ReasoningEffortConfig::default()),
                collaboration_mode: None,
                personality: None,
                message_history: None,
                network_proxy: None,
                rollout_path: None,
            });
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
                let work_status_line = render_work_status_line(&context);
                let side_width = side_width_for_state(area.width, sidebar);
                let base_layout =
                    layout_for_dimensions_with_side(area, side_width, COMPOSER_ROWS);
                let composer_height = composer_desired_height(
                    base_layout.composer.width,
                    "",
                    &[],
                    work_status_line.is_some(),
                );
                let layout = layout_for_dimensions_with_side(
                    area,
                    side_width,
                    composer_height,
                );
                let blocks = vec![
                    TranscriptBlock {
                        role: TranscriptRole::User,
                        speaker_label: None,
                        lines: vec![Line::from(
                            "Let's redesign the TUI to be more intuitive for CLI users."
                                .to_string(),
                        )],
                    },
                    TranscriptBlock {
                        role: TranscriptRole::Codex,
                        speaker_label: None,
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
                    /*draft_cursor*/ 0,
                    &[],
                    work_status_line.as_ref(),
                );
            })
            .expect("draw");
        terminal.backend().to_string()
    }

    fn render_transcript_fixture(scroll_offset: usize) -> String {
        let mut terminal = Terminal::new(TestBackend::new(40, 6)).expect("terminal");
        let blocks = vec![TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
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

    fn render_composer_fixture(width: u16, height: u16, draft: &str) -> String {
        render_composer_fixture_with_queue(width, height, draft, &[])
    }

    fn render_composer_fixture_with_queue(
        width: u16,
        height: u16,
        draft: &str,
        queued_messages: &[String],
    ) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_background(area, frame.buffer_mut());
                render_composer(
                    area,
                    frame.buffer_mut(),
                    draft,
                    draft.len(),
                    queued_messages,
                    None,
                );
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
    fn chat_header_renders_detailed_token_usage() {
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 180, /*height*/ 2)).expect("terminal");
        let mut context = RedesignChromeContext::fixture();
        context.token_usage = TokenUsage {
            input_tokens: 13_025_169,
            cached_input_tokens: 12_688_896,
            output_tokens: 24_465,
            reasoning_output_tokens: 8_790,
            total_tokens: 1_234_567,
        };

        terminal
            .draw(|frame| {
                render_chat_bar(frame.area(), frame.buffer_mut(), &context);
            })
            .expect("draw");

        let rendered = terminal.backend().to_string();
        assert!(
            rendered.contains(
                "tokens: total=360,738 input=336,273 (+ 12,688,896 cached) output=24,465 (reasoning 8,790)"
            ),
            "expected detailed token usage in chat header, got: {rendered:?}"
        );
        assert!(
            !rendered.contains("tok 1.23M"),
            "compact token usage should not be rendered in chat header: {rendered:?}"
        );
    }

    #[test]
    fn chat_header_uses_clear_labels_full_approval_and_consistent_info_color() {
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 132, /*height*/ 2)).expect("terminal");
        let mut context = RedesignChromeContext::fixture();
        context.permissions = "workspace-write".to_string();
        context.approval = "on-request/guardian_subagent".to_string();

        terminal
            .draw(|frame| {
                render_chat_bar(frame.area(), frame.buffer_mut(), &context);
            })
            .expect("draw");

        let rendered = terminal.backend().to_string();
        let mut rows = rendered.lines();
        let primary_row = rows.next().expect("primary chat header row");
        let policy_row = rows.next().expect("policy chat header row");
        assert!(
            primary_row.contains("Model: gpt-5.4 xhigh"),
            "model should use an explicit label, got: {rendered:?}"
        );
        assert!(
            primary_row.contains("ctx: 72%"),
            "context should use a colon-delimited label, got: {rendered:?}"
        );
        assert!(
            primary_row.contains("tokens: total=2,100"),
            "token usage should use a colon-delimited label, got: {rendered:?}"
        );
        assert!(
            !primary_row.contains("permissions") && !primary_row.contains("approval"),
            "policy fields should not render on the primary header row, got: {rendered:?}"
        );
        assert!(
            policy_row.contains("permissions: workspace-write"),
            "permissions should use a clear label, got: {rendered:?}"
        );
        assert!(
            policy_row.contains("approval: on-request/guardian_subagent"),
            "approval reviewer should not be pre-truncated, got: {rendered:?}"
        );
        assert!(
            !policy_row.contains("perm ")
                && !policy_row.contains("appv ")
                && !policy_row.contains("..."),
            "header should avoid unclear abbreviations and pre-truncation, got: {rendered:?}"
        );

        let buffer = terminal.backend().buffer();
        let model_label_x = primary_row.find("Model:").expect("model label") as u16;
        let model_x = primary_row.find("gpt-5.4").expect("model value") as u16;
        let context_label_x = primary_row.find("ctx:").expect("context label") as u16;
        let context_x = primary_row.find("72%").expect("context value") as u16;
        let usage_label_x = primary_row.find("tokens:").expect("token usage label") as u16;
        let usage_x = primary_row.find("total=2,100").expect("token usage value") as u16;
        let permissions_label_x =
            policy_row.find("permissions:").expect("permissions label") as u16;
        let permissions_x = policy_row
            .find("workspace-write")
            .expect("permissions value") as u16;
        let approval_label_x = policy_row.find("approval:").expect("approval label") as u16;
        let approval_x = policy_row
            .find("on-request/guardian_subagent")
            .expect("approval value") as u16;

        let label_fg = buffer[(model_label_x, 0)].fg;
        let value_fg = buffer[(model_x, 0)].fg;
        assert_ne!(
            label_fg, value_fg,
            "labels and values should use different colors"
        );
        assert_eq!(buffer[(context_label_x, 0)].fg, label_fg);
        assert_eq!(buffer[(usage_label_x, 0)].fg, label_fg);
        assert_eq!(buffer[(permissions_label_x, 1)].fg, label_fg);
        assert_eq!(buffer[(approval_label_x, 1)].fg, label_fg);
        assert_eq!(buffer[(context_x, 0)].fg, value_fg);
        assert_eq!(buffer[(usage_x, 0)].fg, value_fg);
        assert_eq!(buffer[(permissions_x, 1)].fg, value_fg);
        assert_eq!(buffer[(approval_x, 1)].fg, value_fg);
    }

    #[test]
    fn top_chrome_uses_sidebar_identity_and_chat_status_at_terminal_top() {
        let rendered = render_fixture(100, 24);
        let mut rows = rendered.lines();
        let primary = rows.next().expect("primary top row").trim_matches('"');
        let policy = rows.next().expect("policy top row").trim_matches('"');
        let separator = rows.next().expect("chat separator row").trim_matches('"');

        assert!(
            primary
                .starts_with("CODEX_CLI dev          |Model: gpt-5.4 xhigh ctx: 72% tokens: 2.1K"),
            "top row should combine the product identity and chat status, got: {primary:?}"
        );
        assert!(
            policy.starts_with(
                "                       |permissions: workspace-write approval: auto-review"
            ),
            "second row should combine left chrome space and chat policy, got: {policy:?}"
        );
        assert!(
            separator.starts_with("                       |---"),
            "chat separator should start after the sidebar boundary, got: {separator:?}"
        );
        assert!(
            !primary.contains("redesign-tui") && !policy.contains("redesign-tui"),
            "top chat chrome should not contain branch/workspace metadata: {rendered:?}"
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
    fn wrapped_composer_snapshot() {
        let draft = "Please review the recently modified rendering code before shipping.";
        assert_snapshot!(
            "redesign_chrome_wrapped_composer_44x4",
            render_composer_fixture(
                /*width*/ 44,
                composer_desired_height(
                    /*width*/ 44,
                    draft,
                    &[],
                    /*work_status_visible*/ false
                ),
                draft,
            )
        );
    }

    #[test]
    fn queued_messages_render_above_message_bar_snapshot() {
        let queued_messages = vec![
            "Queued follow-up question".to_string(),
            "Please also check the terminal resize case.".to_string(),
        ];
        assert_snapshot!(
            "redesign_chrome_queued_messages_52x7",
            render_composer_fixture_with_queue(
                /*width*/ 52,
                composer_desired_height(
                    /*width*/ 52,
                    "",
                    &queued_messages,
                    /*work_status_visible*/ false,
                ),
                "",
                &queued_messages,
            )
        );
    }

    #[test]
    fn layout_keeps_composer_anchored_above_footer_across_resize_classes() {
        for (width, height) in [(44, 4), (72, 18), (100, 24), (132, 24)] {
            let layout = layout_for_dimensions(Rect::new(0, 0, width, height), COMPOSER_ROWS);

            assert_eq!(
                layout.footer.height,
                2.min(height),
                "footer should reserve separate info and shortcut rows for {width}x{height}"
            );
            assert_eq!(
                layout.footer.bottom(),
                height,
                "footer should stay anchored at terminal bottom for {width}x{height}"
            );
            assert_eq!(
                layout.composer.bottom(),
                layout.footer.y,
                "composer should stay directly above footer for {width}x{height}"
            );
            if height >= 18 {
                assert!(
                    layout.composer.height > 0,
                    "composer should remain visible for {width}x{height}"
                );
            }
        }
    }

    #[test]
    fn layout_starts_sidebar_and_chat_header_at_terminal_top() {
        let layout = layout_for_dimensions(Rect::new(0, 0, 100, 24), COMPOSER_ROWS);

        assert_eq!(layout.side.y, 0, "sidebar should start at terminal top");
        assert_eq!(
            layout.main.y, 0,
            "main chat column should start at terminal top"
        );
        assert_eq!(
            layout.chat_header.y, 0,
            "chat status header should occupy the first main rows"
        );
        assert_eq!(
            layout.chat_separator.y, 2,
            "separator should sit directly below the two-row chat header"
        );
    }

    #[test]
    fn footer_splits_info_and_shortcuts_into_separate_rows() {
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 2)).expect("terminal");
        terminal
            .draw(|frame| {
                render_background(frame.area(), frame.buffer_mut());
                render_footer(
                    frame.area(),
                    frame.buffer_mut(),
                    &RedesignChromeContext::fixture(),
                );
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();
        let mut rows = rendered.lines();
        let info = rows.next().expect("info row");
        let shortcuts = rows.next().expect("shortcut row");

        assert!(info.contains("~/codes/codex"));
        assert!(info.contains("redesign-tui"));
        assert!(!info.contains("workspace-write"));
        assert!(!info.contains("auto-review"));
        assert!(!info.contains("Alt-H"));
        assert!(shortcuts.contains("Alt-H"));
        assert!(shortcuts.contains("Alt-/"));
        assert!(shortcuts.contains("Alt-M"));
        assert!(shortcuts.contains("Alt-P"));
        assert!(shortcuts.contains("Alt-T"));
        assert!(shortcuts.contains("C-T"));
        assert!(!shortcuts.contains("Alt-A"));
        assert!(!shortcuts.contains("workspace-write"));
        assert!(!shortcuts.contains("auto-review"));
        assert!(shortcuts.contains("C-C"));
        assert!(!shortcuts.contains("~/codes/codex"));
    }

    #[test]
    fn footer_shortcuts_align_to_chat_column_when_sidebar_is_visible() {
        let rendered = render_fixture(100, 24);
        let shortcuts = rendered
            .lines()
            .last()
            .expect("shortcut footer row")
            .trim_matches('"');

        assert!(
            shortcuts.starts_with("                       |Alt-H Help · Alt-/ Cmds"),
            "shortcut row should start at the chat column after the sidebar boundary, got: {shortcuts:?}"
        );
        assert!(
            shortcuts.contains("Alt-M Model")
                && shortcuts.contains("Alt-P Plan")
                && shortcuts.contains("Alt-T Term")
                && shortcuts.contains("C-C Exit"),
            "aligned shortcut row should keep core shortcuts visible, got: {shortcuts:?}"
        );
        assert!(
            !shortcuts.contains("C-T Transcript"),
            "aligned shortcut row should use the compact set when the chat column is too narrow, got: {shortcuts:?}"
        );
    }

    #[test]
    fn footer_info_aligns_to_chat_column_when_sidebar_is_visible() {
        let rendered = render_fixture(100, 24);
        let info = rendered
            .lines()
            .rev()
            .nth(1)
            .expect("footer info row")
            .trim_matches('"');

        assert!(
            info.starts_with("                       |~/codes/codex · redesign-tui"),
            "footer info row should start at the chat column after the sidebar boundary, got: {info:?}"
        );
        assert!(
            info.contains("3 files") && info.contains("Improve terminal UI"),
            "aligned footer info should keep workspace details visible, got: {info:?}"
        );
    }

    #[test]
    fn footer_shortcuts_align_between_sidebar_and_right_rail() {
        let rendered = render_fixture(132, 24);
        let shortcuts = rendered
            .lines()
            .last()
            .expect("shortcut footer row")
            .trim_matches('"');
        let divider_positions: Vec<usize> = shortcuts
            .chars()
            .enumerate()
            .filter_map(|(idx, ch)| (ch == '|').then_some(idx))
            .collect();

        assert_eq!(
            divider_positions,
            vec![23, 102],
            "footer shortcut row should share sidebar/main/right-rail dividers, got: {shortcuts:?}"
        );
        assert!(
            shortcuts[24..102].contains("Alt-H Help · Alt-/ Cmds"),
            "shortcut text should live inside the main chat column, got: {shortcuts:?}"
        );
    }

    #[test]
    fn sidebar_actions_pin_to_bottom_of_sidebar() {
        let rendered = render_fixture(100, 24);
        let rows: Vec<&str> = rendered
            .lines()
            .map(|line| line.trim_matches('"'))
            .collect();

        assert!(
            rows[12].starts_with("                       |"),
            "chat list should leave vertical breathing room above pinned actions, got: {:?}",
            rows[12]
        );
        assert!(
            rows[13].starts_with(" ACTIONS               |"),
            "actions heading should pin near the sidebar bottom, got: {:?}",
            rows[13]
        );
        assert!(
            rows[21].starts_with("  C-G    EDITOR        |"),
            "last action should sit on the final sidebar row above the footer, got: {:?}",
            rows[21]
        );
    }

    #[test]
    fn composer_height_grows_for_wrapped_draft() {
        let draft = "Please review the recently modified rendering code before shipping.";

        assert!(
            composer_desired_height(
                /*width*/ 32,
                draft,
                &[],
                /*work_status_visible*/ false
            ) > 3
        );
    }

    #[test]
    fn composer_input_area_uses_terminal_background() {
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 24, /*height*/ 4)).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_background(area, frame.buffer_mut());
                render_composer(area, frame.buffer_mut(), "draft", "draft".len(), &[], None);
            })
            .expect("draw");

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].bg, Color::Reset);
        assert_eq!(buffer[(0, 1)].bg, Color::Reset);
        assert_eq!(buffer[(0, 2)].bg, Color::Reset);
        assert_eq!(buffer[(0, 3)].bg, Color::Reset);
    }

    #[tokio::test]
    async fn slash_fallback_replaces_redesign_composer_chrome() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        app.chat_widget.insert_str("/");

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_app(area, frame.buffer_mut(), &app);
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();

        assert!(app.chat_widget.redesign_should_render_bottom_pane());
        assert!(!rendered.contains("Describe the next change..."));
        assert!(rendered.contains("/"));
    }

    #[tokio::test]
    async fn plan_window_renders_latest_plan_snapshot() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        seed_test_thread(&mut app);
        app.chat_widget
            .set_redesign_latest_plan_update_for_test(UpdatePlanArgs {
                explanation: Some("Current implementation checklist".to_string()),
                plan: vec![
                    PlanItemArg {
                        step: "Inspect existing plan rendering".to_string(),
                        status: StepStatus::Completed,
                    },
                    PlanItemArg {
                        step: "Add per-chat floating window".to_string(),
                        status: StepStatus::InProgress,
                    },
                    PlanItemArg {
                        step: "Cover shortcut and render behavior".to_string(),
                        status: StepStatus::Pending,
                    },
                ],
            });
        app.toggle_redesign_plan_window_for_active_chat();

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");

        let layout = layout_for(
            Rect::new(0, 0, 100, 24),
            &app,
            app.chat_widget.redesign_should_render_bottom_pane(),
        );
        let panel = plan_window_rect(layout.main).expect("plan window rect");
        assert_eq!(
            terminal.backend().buffer()[(panel.x, panel.y)].fg,
            Color::Cyan
        );

        assert_snapshot!(
            "redesign_chrome_plan_window_100x24",
            terminal.backend().to_string()
        );
    }

    #[tokio::test]
    async fn plan_window_falls_back_to_latest_proposed_plan() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        seed_test_thread(&mut app);
        app.chat_widget.set_redesign_latest_proposed_plan_for_test(
            "## Plan\n\n- Inspect plan sources\n- Render proposed plan fallback\n".to_string(),
        );
        app.toggle_redesign_plan_window_for_active_chat();

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();

        assert!(rendered.contains("Proposed Plan"));
        assert!(rendered.contains("Inspect plan sources"));
        assert!(rendered.contains("Render proposed plan fallback"));
    }

    #[tokio::test]
    async fn terminal_window_renders_background_process_output_snapshot() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        seed_test_thread(&mut app);
        app.chat_widget
            .set_redesign_background_terminals_for_test(vec![
                crate::chatwidget::RedesignBackgroundTerminal {
                    command_display: "cargo test -p codex-tui".to_string(),
                    output_lines: vec![
                        "running 2 tests".to_string(),
                        "test redraws_terminal_window ... ok".to_string(),
                        "test keeps_history_tail ... ok".to_string(),
                    ],
                    status: crate::chatwidget::RedesignBackgroundTerminalStatus::Running,
                    exit_code: None,
                },
            ]);
        app.toggle_redesign_terminal_window_for_active_chat();

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");

        assert_snapshot!(
            "redesign_chrome_terminal_window_100x24",
            terminal.backend().to_string()
        );
    }

    #[test]
    fn composer_cursor_advances_for_trailing_space() {
        let before =
            render_composer_cursor(/*width*/ 24, /*height*/ 3, "hello", "hello".len());
        let after = render_composer_cursor(
            /*width*/ 24,
            /*height*/ 3,
            "hello ",
            "hello ".len(),
        );

        assert_eq!(after, (before.0 + 1, before.1));
    }

    #[test]
    fn composer_cursor_uses_draft_cursor_offset() {
        let start = render_composer_cursor(
            /*width*/ 24,
            /*height*/ 3,
            "hello world",
            /*draft_cursor*/ 0,
        );
        let middle = render_composer_cursor(
            /*width*/ 24,
            /*height*/ 3,
            "hello world",
            "hello".len(),
        );

        assert_eq!(middle, (start.0 + 5, start.1));
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
            speaker_label: None,
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
    fn bubble_lines_uses_named_agent_label() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![Line::from("I found the issue.")],
        };

        let lines =
            bubble_lines_with_assistant_label(&block, /*area_width*/ 80, "Robie [explorer]")
                .iter()
                .map(plain_line_text)
                .collect::<Vec<_>>();

        assert_eq!(lines[0], " Robie [explorer]");
        assert!(
            lines.iter().all(|line| !line.contains("Codex")),
            "named agent transcript should not render the default Codex label: {lines:?}"
        );
    }

    #[test]
    fn bubble_lines_uses_message_speaker_label_over_thread_label() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: Some("Riley (Domain Expert)".to_string()),
            lines: vec![Line::from("Doing well, Shaun.")],
        };

        let lines = bubble_lines_with_assistant_label(&block, /*area_width*/ 80, "Codex")
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();

        assert_eq!(lines[0], " Riley (Domain Expert)");
        assert!(
            lines.iter().all(|line| !line.contains("Codex")),
            "message speaker label should override the default assistant label: {lines:?}"
        );
    }

    #[test]
    fn bubble_lines_reflow_prewrapped_prose() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![
                Line::from("It supports interactive"),
                Line::from("terminal"),
                Line::from("use, non-interactive"),
                Line::from("automation, MCP"),
            ],
        };

        let lines = bubble_lines(&block, /*area_width*/ 44)
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();

        assert!(
            lines.iter().any(|line| line.contains("terminal use,")),
            "expected prose to reflow into fuller bubble lines, got: {lines:?}"
        );
    }

    #[test]
    fn bubble_lines_reflow_prewrapped_ordered_list() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![
                Line::from("2. Core agent logic lives in"),
                Line::from("   crates"),
                Line::from("   like codex-core, but new"),
                Line::from("   code should avoid bloating"),
                Line::from("   it."),
            ],
        };

        let lines = bubble_lines(&block, /*area_width*/ 44)
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();

        assert!(
            lines
                .iter()
                .any(|line| line.contains("crates like codex-core")),
            "expected ordered-list continuations to reflow into fuller bubble lines, got: {lines:?}"
        );
    }

    #[test]
    fn bubble_lines_keeps_adjacent_list_items_distinct() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![
                Line::from("- First rendered item"),
                Line::from("- Second rendered item"),
            ],
        };

        let lines = bubble_lines(&block, /*area_width*/ 44)
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();
        let rendered = lines.join("\n");

        assert!(rendered.contains("- First rendered item"));
        assert!(rendered.contains("- Second rendered item"));
        assert!(
            !rendered.contains("First rendered item - Second rendered item"),
            "adjacent list items should not be reflowed into one paragraph: {lines:?}"
        );
    }

    #[test]
    fn bubble_lines_keeps_adjacent_shell_commands_distinct() {
        let block = TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![
                Line::from("cargo fmt -p codex-cli -p codex-tui"),
                Line::from("cargo check -p codex-cli -p codex-tui"),
            ],
        };

        let lines = bubble_lines(&block, /*area_width*/ 132)
            .iter()
            .map(plain_line_text)
            .collect::<Vec<_>>();
        let rendered = lines.join("\n");

        assert!(rendered.contains("cargo fmt -p codex-cli -p codex-tui"));
        assert!(rendered.contains("cargo check -p codex-cli -p codex-tui"));
        assert!(
            !rendered.contains("codex-tui cargo check"),
            "adjacent shell commands should not be reflowed into one paragraph: {lines:?}"
        );
    }

    #[tokio::test]
    async fn transcript_render_preserves_markdown_table_separator_rows() {
        let mut app = crate::app::test_support::make_test_app().await;
        let cwd = app.config.cwd.clone();
        app.transcript_cells = vec![std::sync::Arc::new(history_cell::AgentMarkdownCell::new(
            concat!(
                "| Command | Result |\n",
                "| --- | --- |\n",
                "| cargo test -p codex-tui markdown table rendering regression | ",
                "passed with focused table assertions |\n",
            )
            .to_string(),
            cwd.as_path(),
        ))];

        let lines = transcript_display_window_from_app(
            &app, /*width*/ 80, /*height*/ 18, /*route_system_cells_to_rail*/ true,
            /*scroll_offset*/ 0, "Codex",
        )
        .lines
        .iter()
        .map(plain_line_text)
        .collect::<Vec<_>>();

        let separator_rows = lines
            .iter()
            .filter(|line| line.contains('━'))
            .collect::<Vec<_>>();
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Command") && line.contains("Result")),
            "expected table header row to stay on one bubble line, got: {lines:?}"
        );
        assert_eq!(
            separator_rows.len(),
            1,
            "expected table separator row to stay on one bubble line, got: {lines:?}"
        );
        assert!(
            separator_rows[0].matches('━').count() > 40,
            "table separator should preserve both table columns, got: {lines:?}"
        );
    }

    #[tokio::test]
    async fn finalized_markdown_table_stays_within_redesign_columns_snapshot() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        let cwd = app.config.cwd.clone();
        app.transcript_cells = vec![std::sync::Arc::new(history_cell::AgentMarkdownCell::new(
            concat!(
                "The redesign owns the complete retained frame.\n\n",
                "| Component | Rendering owner |\n",
                "| --- | --- |\n",
                "| Sidebar | redesign frame |\n",
                "| Transcript | redesign frame |\n",
                "| Composer | redesign frame |\n",
                "| Footer | redesign frame |\n",
            )
            .to_string(),
            cwd.as_path(),
        ))];

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");

        assert_snapshot!(
            "redesign_chrome_finalized_markdown_table_100x24",
            terminal.backend().to_string()
        );
    }

    #[test]
    fn transcript_render_keeps_markdown_list_items_snapshot() {
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 72, /*height*/ 14)).expect("terminal");
        let blocks = vec![TranscriptBlock {
            role: TranscriptRole::Codex,
            speaker_label: None,
            lines: vec![
                Line::from("What changed:"),
                Line::from(""),
                Line::from("- Added /persistent-skill command support."),
                Line::from("- Added persistent skill resolution/status/clear logic."),
                Line::from("- Wired user turns to inject once, then keep compact instructions."),
                Line::from(""),
                Line::from("TAIL_SENTINEL"),
            ],
        }];

        terminal
            .draw(|frame| {
                render_transcript(
                    frame.area(),
                    frame.buffer_mut(),
                    &blocks,
                    /*scroll_offset*/ 0,
                );
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();

        assert!(rendered.contains("TAIL_SENTINEL"));
        assert_snapshot!("redesign_chrome_markdown_list_items", rendered);
    }

    #[test]
    fn product_version_label_uses_source_build_label_for_dev_version() {
        assert_eq!(
            product_version_label("CODEX_CLI"),
            if is_source_build_version_label(CODEX_CLI_VERSION) {
                "CODEX_CLI dev".to_string()
            } else {
                format!("CODEX_CLI v{CODEX_CLI_VERSION}")
            }
        );
    }

    #[test]
    fn work_activity_indicator_only_renders_while_working() {
        let mut context = RedesignChromeContext::fixture();
        assert_eq!(
            work_activity_indicator(&context)
                .expect("working indicator")
                .content
                .as_ref(),
            "⠋"
        );

        context.working = false;

        assert_eq!(work_activity_indicator(&context), None);
    }

    #[test]
    fn content_width_accounts_for_side_nav() {
        assert_eq!(content_width_for_terminal_width(100), 76);
        assert_eq!(content_width_for_terminal_width(132), 78);
        assert_eq!(content_width_for_terminal_width(72), 72);
    }

    #[tokio::test]
    async fn render_app_formats_only_visible_transcript_tail() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        let display_calls = Arc::new(AtomicUsize::new(0));
        app.transcript_cells = (0..1_000)
            .map(|idx| {
                Arc::new(CountingHistoryCell {
                    text: format!("message {idx}"),
                    display_calls: display_calls.clone(),
                }) as Arc<dyn HistoryCell>
            })
            .collect();

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 100, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();

        assert!(rendered.contains("message 999"));
        assert!(!rendered.contains("message 0"));
        assert!(
            display_calls.load(Ordering::Relaxed) < 100,
            "tail render should not format the full transcript"
        );
    }

    #[tokio::test]
    async fn render_app_formats_only_visible_system_rail_tail() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.redesign_chrome_enabled = true;
        let display_calls = Arc::new(AtomicUsize::new(0));
        app.transcript_cells = (0..1_000)
            .map(|idx| {
                Arc::new(CountingHistoryCell {
                    text: format!("rail {idx}"),
                    display_calls: display_calls.clone(),
                }) as Arc<dyn HistoryCell>
            })
            .collect();

        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 132, /*height*/ 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_app(frame.area(), frame.buffer_mut(), &app);
            })
            .expect("draw");
        let rendered = terminal.backend().to_string();

        assert!(rendered.contains("rail 999"));
        assert!(!rendered.contains("rail 0"));
        assert!(
            display_calls.load(Ordering::Relaxed) < 100,
            "tail render should not format the full system rail"
        );
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

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ true,
        );
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
                    speaker_label: None,
                    lines: vec![Line::from("Run the tests")],
                },
                TranscriptBlock {
                    role: TranscriptRole::Codex,
                    speaker_label: None,
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
    async fn transcript_inlines_system_cells_when_right_rail_is_hidden() {
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

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ false,
        );
        let summary = blocks
            .iter()
            .map(|block| {
                (
                    block.role,
                    block.lines.iter().map(plain_line_text).collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summary,
            vec![
                (TranscriptRole::User, vec!["Run the tests".to_string()]),
                (
                    TranscriptRole::Codex,
                    vec!["checking the changed crates".to_string()]
                ),
                (TranscriptRole::System, vec!["system notice".to_string()]),
                (TranscriptRole::Codex, vec!["Tests passed".to_string()]),
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

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ false,
        );
        let rail = system_rail_blocks(&app, /*width*/ 30);

        assert_eq!(
            blocks,
            vec![
                TranscriptBlock {
                    role: TranscriptRole::User,
                    speaker_label: None,
                    lines: vec![Line::from("Run the tests")],
                },
                TranscriptBlock {
                    role: TranscriptRole::Codex,
                    speaker_label: None,
                    lines: vec![Line::from("Tests passed")],
                },
            ]
        );
        assert_eq!(rail, Vec::<SystemRailBlock>::new());
    }

    #[tokio::test]
    async fn transcript_blocks_preserve_user_indentation() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.transcript_cells = vec![std::sync::Arc::new(history_cell::new_user_prompt(
            "Please keep:\n  indented line".to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ))];

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ true,
        );

        assert_eq!(
            blocks,
            vec![TranscriptBlock {
                role: TranscriptRole::User,
                speaker_label: None,
                lines: vec![Line::from("Please keep:"), Line::from("  indented line")],
            }]
        );
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

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ true,
        );

        assert_eq!(
            blocks,
            vec![TranscriptBlock {
                role: TranscriptRole::Codex,
                speaker_label: None,
                lines: vec![Line::from("First line"), Line::from("second line")],
            }]
        );
    }

    #[tokio::test]
    async fn transcript_blocks_promote_agent_prefix_to_speaker_label() {
        let mut app = crate::app::test_support::make_test_app().await;
        app.transcript_cells = vec![std::sync::Arc::new(history_cell::AgentMessageCell::new(
            vec![Line::from(
                "Riley (Domain Expert): Doing well, Shaun. I am ready.",
            )],
            /*is_first_line*/ true,
        ))];

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ true,
        );

        assert_eq!(
            blocks,
            vec![TranscriptBlock {
                role: TranscriptRole::Codex,
                speaker_label: Some("Riley (Domain Expert)".to_string()),
                lines: vec![Line::from("Doing well, Shaun. I am ready.")],
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

        let blocks = transcript_blocks(
            &app, /*width*/ 80, /*route_system_cells_to_rail*/ true,
        );
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
