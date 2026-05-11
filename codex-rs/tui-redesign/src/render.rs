use std::io;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::ApprovalChoice;
use crate::ApprovalRequest;
use crate::FocusTarget;
use crate::FooterShortcut;
use crate::RedesignState;
use crate::Role;
use crate::TopContext;
use crate::TranscriptEntry;
use crate::WorkStatus;
use crate::WorkspaceContext;
use crate::theme;

mod overlay;
#[cfg(test)]
mod tests;

const SIDE_NAV_WIDTH: u16 = 24;
const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
struct ChromeHeights {
    top: u16,
    footer: u16,
    secondary: u16,
    work_strip: u16,
    composer: u16,
}

impl ChromeHeights {
    fn for_area_height(height: u16, has_work: bool) -> Self {
        if height >= 30 {
            return Self {
                top: 2,
                footer: 2,
                secondary: 2,
                work_strip: if has_work { 3 } else { 0 },
                composer: 2,
            };
        }

        Self {
            top: 1,
            footer: 1,
            secondary: 2,
            work_strip: if has_work { 1 } else { 0 },
            composer: 2,
        }
    }
}

pub struct RedesignApp<'a> {
    state: &'a RedesignState,
}

impl<'a> RedesignApp<'a> {
    pub fn new(state: &'a RedesignState) -> Self {
        Self { state }
    }
}

pub fn render_to_string(width: u16, height: u16, state: &RedesignState) -> io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| {
        frame.render_widget(RedesignApp::new(state), frame.area());
    })?;
    Ok(buffer_to_plain_text(terminal.backend().buffer()))
}

impl Widget for RedesignApp<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        buf.set_style(area, theme::app());
        let chrome = ChromeHeights::for_area_height(area.height, self.state.work.is_some());

        let [top_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(chrome.top),
            Constraint::Min(0),
            Constraint::Length(chrome.footer),
        ])
        .areas(area);

        render_top_app_bar(top_area, buf, &self.state.top);
        render_body(body_area, buf, self.state, chrome);
        render_global_footer(footer_area, buf, &self.state.footer_shortcuts);
        overlay::render_overlay(area, buf, self.state);
    }
}

fn render_body(area: Rect, buf: &mut Buffer, state: &RedesignState, chrome: ChromeHeights) {
    if area.is_empty() {
        return;
    }

    let show_side_nav = area.width >= 100;
    let [side_area, main_area] = if show_side_nav {
        Layout::horizontal([Constraint::Length(SIDE_NAV_WIDTH), Constraint::Min(0)]).areas(area)
    } else {
        [Rect::new(area.x, area.y, 0, area.height), area]
    };

    if show_side_nav {
        render_side_nav(side_area, buf);
    }

    let [secondary_area, transcript_area, work_area, composer_area] = Layout::vertical([
        Constraint::Length(chrome.secondary),
        Constraint::Min(0),
        Constraint::Length(chrome.work_strip),
        Constraint::Length(chrome.composer),
    ])
    .areas(main_area);

    render_secondary_header(secondary_area, buf, &state.workspace);
    render_transcript(
        transcript_area,
        buf,
        &state.transcript,
        state.approval.as_ref(),
        state.approval_choice,
        state.focus == FocusTarget::Transcript,
        state.focus == FocusTarget::Approval,
    );
    if let Some(work) = &state.work {
        render_work_strip(work_area, buf, work, &state.status);
    }
    render_composer(
        composer_area,
        buf,
        state,
        state.focus == FocusTarget::Composer,
    );
}

fn render_top_app_bar(area: Rect, buf: &mut Buffer, top: &TopContext) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::app());
    if area.height > 1 {
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(theme::border())
            .style(theme::app())
            .render(area, buf);
    }

    let content_area = bottom_bordered_content_line(area, 1);
    let line = if area.width < 100 {
        Line::from(vec![
            Span::styled(codex_cli_version_label(), theme::primary_container()),
            sep(),
            Span::styled("NETWORK", theme::secondary_fixed()),
            sep(),
            format!("MOD: {} {}", top.model, top.reasoning).into(),
            sep(),
            format!("CTX: {}", top.context_left).into(),
        ])
    } else if area.width < 112 {
        Line::from(vec![
            Span::styled(codex_cli_version_label(), theme::primary_container()),
            sep(),
            Span::styled("NETWORK", theme::secondary_fixed()),
            sep(),
            format!("MOD: {} {}", top.model, top.reasoning).into(),
            sep(),
            Span::styled("PERM: ", theme::metadata()),
            Span::styled(top.permissions.clone(), theme::secondary_fixed()),
            sep(),
            format!("CTX: {}", top.context_left).into(),
        ])
    } else if area.width < 132 {
        Line::from(vec![
            Span::styled(codex_cli_version_label(), theme::primary_container()),
            sep(),
            Span::styled("NETWORK", theme::secondary_fixed()),
            sep(),
            format!("MOD: {} {}", top.model, top.reasoning).into(),
            sep(),
            Span::styled("PERM: ", theme::metadata()),
            Span::styled(top.permissions.clone(), theme::secondary_fixed()),
            sep(),
            Span::styled("APPV: ", theme::metadata()),
            Span::styled(top.approval_mode.clone(), theme::tertiary()),
            sep(),
            format!("CTX: {}", top.context_left).into(),
        ])
    } else {
        Line::from(vec![
            Span::styled(codex_cli_version_label(), theme::primary_container()),
            "  ".into(),
            Span::styled("SYSTEM", theme::metadata()),
            "  ".into(),
            Span::styled("NETWORK", theme::secondary_fixed()),
            "  ".into(),
            Span::styled("CONTEXT", theme::metadata()),
            "  ".into(),
            Span::styled("MOD: ", theme::metadata()),
            Span::styled(format!("{} {}", top.model, top.reasoning), theme::primary()),
            "  ".into(),
            Span::styled("PERM: ", theme::metadata()),
            Span::styled(top.permissions.clone(), theme::secondary_fixed()),
            "  ".into(),
            Span::styled("APPV: ", theme::metadata()),
            Span::styled(top.approval_mode.clone(), theme::tertiary()),
            "  ".into(),
            Span::styled("CTX: ", theme::metadata()),
            Span::styled(top.context_left.clone(), theme::secondary()),
        ])
    };

    Paragraph::new(line)
        .style(theme::app())
        .render(content_area, buf);
}

fn codex_cli_version_label() -> String {
    format!("CODEX_CLI v{CODEX_CLI_VERSION}")
}

fn render_side_nav(area: Rect, buf: &mut Buffer) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::side_nav());
    let block = Block::new()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme::OUTLINE_VARIANT))
        .style(theme::side_nav());
    let inner = block.inner(area);
    block.render(area, buf);

    let lines = vec![
        Line::from(Span::styled("CTX_MGR", theme::secondary_fixed())),
        Line::from(Span::styled("PATH: ~/root/dev", theme::metadata())),
        Line::from(""),
        Line::from(vec![
            "  ◌ ".into(),
            Span::styled("THREADS", theme::metadata()),
        ]),
        Line::from(vec![
            "  □ ".into(),
            Span::styled("PROJECTS", theme::metadata()),
        ]),
        Line::from(vec![
            "  ◇ ".into(),
            Span::styled("MODELS", theme::metadata()),
        ]),
        Line::from(vec![
            Span::styled("▌", theme::secondary_fixed()),
            Span::styled(" ◷ HISTORY", theme::side_nav_active()),
        ])
        .style(theme::side_nav_active()),
    ];

    Paragraph::new(Text::from(lines))
        .style(theme::side_nav())
        .render(inner, buf);
}

fn render_secondary_header(area: Rect, buf: &mut Buffer, workspace: &WorkspaceContext) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::surface_lowest());
    let line = if area.width < 90 {
        Line::from(vec![
            Span::styled("DIR: ", theme::metadata()),
            compact_path(&workspace.path).into(),
            sep(),
            Span::styled("BRN: ", theme::metadata()),
            Span::styled(workspace.branch.clone(), theme::secondary()),
            sep(),
            Span::styled("CHG: ", theme::metadata()),
            Span::styled(workspace.changed_files.clone(), theme::tertiary()),
        ])
    } else {
        Line::from(vec![
            Span::styled("DIR: ", theme::metadata()),
            workspace.path.clone().into(),
            sep(),
            Span::styled("BRN: ", theme::metadata()),
            Span::styled(workspace.branch.clone(), theme::secondary()),
            sep(),
            Span::styled("CHG: ", theme::metadata()),
            Span::styled(workspace.changed_files.clone(), theme::tertiary()),
            sep(),
            Span::styled("THR: ", theme::metadata()),
            Span::styled(workspace.thread_title.clone(), theme::secondary()),
        ])
    };

    if area.height > 1 {
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(theme::border())
            .style(theme::surface_lowest())
            .render(area, buf);
    }
    Paragraph::new(line)
        .style(theme::surface_lowest())
        .render(bottom_bordered_content_line(area, 1), buf);
}

fn render_transcript(
    area: Rect,
    buf: &mut Buffer,
    entries: &[TranscriptEntry],
    approval: Option<&ApprovalRequest>,
    approval_choice: ApprovalChoice,
    transcript_focused: bool,
    approval_focused: bool,
) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::app());
    let mut lines = Vec::new();
    for entry in entries {
        push_stitch_message(&mut lines, entry, area.width);
        lines.push(Line::from(""));
    }
    if let Some(approval) = approval {
        push_inline_approval(
            &mut lines,
            approval,
            approval_choice,
            approval_focused,
            area.width,
        );
    }
    if lines.last().is_some_and(|line| line.width() == 0) {
        lines.pop();
    }

    let visible_lines = top_lines(lines, area.height as usize);
    Paragraph::new(Text::from(visible_lines))
        .style(theme::app())
        .block(
            Block::new()
                .borders(Borders::RIGHT)
                .border_style(if transcript_focused {
                    theme::focus()
                } else {
                    theme::border()
                })
                .style(theme::app()),
        )
        .render(area, buf);
}

fn push_stitch_message(lines: &mut Vec<Line<'static>>, entry: &TranscriptEntry, width: u16) {
    let body_width = usize::from(width).saturating_sub(4).max(12);
    let wrapped = textwrap::wrap(&entry.text, body_width);
    let wrapped: Vec<String> = if wrapped.is_empty() {
        vec![String::new()]
    } else {
        wrapped
            .into_iter()
            .map(std::borrow::Cow::into_owned)
            .collect()
    };

    lines.push(Line::from(vec![
        message_label(entry.role),
        " ".into(),
        Span::styled("────", theme::border()),
    ]));
    for line in wrapped {
        lines.push(Line::from(vec![
            "  ".into(),
            Span::styled("│ ", theme::border()),
            body_span(entry.role, line),
        ]));
    }
}

fn push_inline_approval(
    lines: &mut Vec<Line<'static>>,
    approval: &ApprovalRequest,
    selected_choice: ApprovalChoice,
    focused: bool,
    width: u16,
) {
    let box_width = usize::from(width).saturating_sub(3).clamp(36, 78);
    let title = format!("[ {} ]", approval.title);
    let title_fill = box_width.saturating_sub(3 + title.chars().count());
    let inner_width = box_width.saturating_sub(8);
    let command = trim_to_width(&approval.command, inner_width);
    let command_fill = box_width.saturating_sub(8 + command.chars().count());
    let reason = trim_to_width(&approval.reason, inner_width);
    let reason_fill = box_width.saturating_sub(8 + reason.chars().count());
    let actions_width = "[ APPROVE ]  [ APV_SESS ]  [ DENY ]".chars().count();
    let actions_fill = box_width.saturating_sub(3 + actions_width);

    lines.push(Line::from(vec![
        Span::styled("┌─", theme::border()),
        Span::styled(
            title,
            if focused {
                theme::focus()
            } else {
                theme::tertiary_fixed()
            },
        ),
        Span::styled("─".repeat(title_fill), theme::border()),
        Span::styled("┐", theme::border()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("│ ", theme::border()),
        Span::styled("CMD: ", theme::metadata()),
        Span::styled(command, theme::secondary_fixed()),
        " ".repeat(command_fill).into(),
        Span::styled("│", theme::border()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("│ ", theme::border()),
        Span::styled("RSN: ", theme::metadata()),
        Span::styled(reason, theme::metadata()),
        " ".repeat(reason_fill).into(),
        Span::styled("│", theme::border()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("│ ", theme::border()),
        approval_action_span(
            "[ APPROVE ]",
            ApprovalChoice::Approve,
            selected_choice,
            focused,
        ),
        "  ".into(),
        approval_action_span(
            "[ APV_SESS ]",
            ApprovalChoice::ApproveSession,
            selected_choice,
            focused,
        ),
        "  ".into(),
        approval_action_span("[ DENY ]", ApprovalChoice::Deny, selected_choice, focused),
        " ".repeat(actions_fill).into(),
        Span::styled("│", theme::border()),
    ]));
    lines.push(Line::from(Span::styled(
        "└".to_string() + &"─".repeat(box_width.saturating_sub(2)) + "┘",
        theme::border(),
    )));
}

fn approval_action_span(
    label: &'static str,
    choice: ApprovalChoice,
    selected_choice: ApprovalChoice,
    focused: bool,
) -> Span<'static> {
    if focused && choice == selected_choice {
        return Span::styled(label, theme::selected());
    }

    match choice {
        ApprovalChoice::Approve => Span::styled(label, theme::tertiary()),
        ApprovalChoice::ApproveSession => Span::styled(label, theme::secondary_fixed()),
        ApprovalChoice::Deny => Span::styled(label, Style::default().fg(theme::ERROR)),
    }
}

fn render_composer(area: Rect, buf: &mut Buffer, state: &RedesignState, focused: bool) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::app());
    let input = if state.composer.draft.is_empty() {
        Span::styled(state.composer.placeholder.clone(), theme::metadata())
    } else {
        Span::from(state.composer.draft.clone())
    };

    if area.height > 1 {
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(theme::border())
            .style(theme::app())
            .render(area, buf);
    }

    Paragraph::new(Line::from(vec![
        Span::styled(
            "MSG> ",
            if focused {
                theme::focus()
            } else {
                theme::primary_container()
            },
        ),
        input,
    ]))
    .style(theme::app())
    .render(bottom_bordered_content_line(area, 1), buf);
}

fn render_work_strip(area: Rect, buf: &mut Buffer, work: &WorkStatus, status: &str) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::surface_highest());
    let line = if area.width < 80 {
        Line::from(vec![
            Span::styled("▶ ", theme::tertiary()),
            Span::styled(work.label.clone(), theme::surface_highest()),
            Span::styled(": ", theme::surface_highest()),
            compact_work_detail(&work.detail).into(),
            sep(),
            Span::styled("Esc", theme::secondary_fixed()),
            Span::styled(" INT", theme::metadata()),
            "  ".into(),
            Span::styled("Tab", theme::secondary_fixed()),
            Span::styled(" QUE", theme::metadata()),
        ])
    } else if area.width < 112 {
        Line::from(vec![
            Span::styled("▶ ", theme::tertiary()),
            Span::styled(work.label.clone(), theme::surface_highest()),
            Span::styled(": ", theme::surface_highest()),
            work.detail.clone().into(),
            sep(),
            Span::styled(format!("ELAPSED: {}", work.elapsed), theme::metadata()),
            sep(),
            Span::styled("Esc", theme::secondary_fixed()),
            Span::styled(" INT", theme::metadata()),
            "  ".into(),
            Span::styled("Tab", theme::secondary_fixed()),
            Span::styled(" QUE", theme::metadata()),
        ])
    } else {
        let status = trim_to_width(status, usize::from(area.width).saturating_sub(88));
        Line::from(vec![
            Span::styled("▶ ", theme::tertiary()),
            Span::styled(work.label.clone(), theme::surface_highest()),
            Span::styled(": ", theme::surface_highest()),
            work.detail.clone().into(),
            sep(),
            Span::styled(format!("ELAPSED: {}", work.elapsed), theme::metadata()),
            sep(),
            Span::styled("Esc", theme::secondary_fixed()),
            Span::styled(" INT", theme::metadata()),
            "  ".into(),
            Span::styled("Tab", theme::secondary_fixed()),
            Span::styled(" QUE", theme::metadata()),
            sep(),
            Span::styled(status, theme::metadata()),
        ])
    };

    if area.height > 1 {
        Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(theme::border())
            .style(theme::surface_highest())
            .render(area, buf);
    }

    let padding = padded_width(area, 1);
    let content_area = Rect::new(
        area.x + padding,
        area.y + area.height.saturating_sub(1) / 2,
        area.width.saturating_sub(padding * 2),
        1,
    );

    Paragraph::new(line)
        .style(theme::surface_highest())
        .render(content_area, buf);
}

fn render_global_footer(area: Rect, buf: &mut Buffer, shortcuts: &[FooterShortcut]) {
    if area.is_empty() {
        return;
    }

    buf.set_style(area, theme::footer());
    if area.height > 1 {
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme::OUTLINE))
            .style(theme::footer())
            .render(area, buf);
    }

    let spans = if area.width < 130 {
        vec![
            Span::styled("(C) OPENAI_CODEX", theme::primary()),
            "  ".into(),
            Span::styled("F1 HELP", theme::metadata()),
            "  ".into(),
            Span::styled("F2 CMD", theme::secondary()),
            "  ".into(),
            Span::styled("F3 CLR", theme::metadata()),
            "  ".into(),
            Span::styled("ESC EXIT", theme::metadata()),
            "  ".into(),
            Span::styled("?", theme::secondary_fixed()),
            Span::styled(" shortcuts", theme::metadata()),
            "  ".into(),
            Span::styled("C-R", theme::secondary_fixed()),
            Span::styled(" history", theme::metadata()),
            "  ".into(),
            Span::styled("C-T", theme::secondary_fixed()),
            Span::styled(" transcript", theme::metadata()),
            "  ".into(),
            Span::styled("S-Tab", theme::secondary_fixed()),
            Span::styled(" mode", theme::metadata()),
        ]
    } else {
        let mut spans = vec![
            Span::styled("(C) 2024 OPENAI_CODEX", theme::primary()),
            "  ".into(),
        ];
        spans.extend([
            Span::styled("F1: HELP", theme::metadata()),
            "  ".into(),
            Span::styled("F2: CMD", theme::secondary()),
            "  ".into(),
            Span::styled("F3: CLR", theme::metadata()),
            "  ".into(),
            Span::styled("ESC: EXIT", theme::metadata()),
            "  ".into(),
        ]);
        for (index, shortcut) in shortcuts.iter().enumerate() {
            if index > 0 {
                spans.push("  ".into());
            }
            spans.push(Span::styled(shortcut.key.clone(), theme::secondary_fixed()));
            spans.push(" ".into());
            spans.push(Span::styled(shortcut.label.clone(), theme::metadata()));
        }
        spans
    };
    Paragraph::new(Line::from(spans))
        .style(theme::footer())
        .render(top_bordered_content_line(area, 1), buf);
}

fn top_lines(lines: Vec<Line<'static>>, max_len: usize) -> Vec<Line<'static>> {
    if lines.len() <= max_len {
        return lines;
    }
    lines[..max_len].to_vec()
}

pub(super) fn message_label(role: Role) -> Span<'static> {
    let label = match role {
        Role::User => "You",
        Role::Codex => "Codex",
        Role::Running => "Running",
        Role::ApprovalNeeded => "Approval Needed",
        Role::Error => "Error",
    };
    match role {
        Role::User => Span::styled(label, theme::secondary_fixed()),
        Role::Codex | Role::Running | Role::ApprovalNeeded => {
            Span::styled(label, theme::primary_container())
        }
        Role::Error => Span::styled(label, Style::default().fg(theme::ERROR)),
    }
}

fn body_span(role: Role, text: String) -> Span<'static> {
    match role {
        Role::Running => Span::styled(text, theme::metadata()),
        Role::ApprovalNeeded => Span::styled(text, theme::secondary_fixed()),
        Role::Error => Span::styled(text, Style::default().fg(theme::ERROR)),
        Role::User | Role::Codex => Span::from(text),
    }
}

fn sep() -> Span<'static> {
    Span::styled(" | ", theme::metadata())
}

fn bottom_bordered_content_line(area: Rect, horizontal_padding: u16) -> Rect {
    if area.is_empty() {
        return area;
    }

    let padding = padded_width(area, horizontal_padding);
    let y = if area.height <= 2 {
        area.y
    } else {
        area.y + area.height / 2
    };

    Rect::new(
        area.x + padding,
        y,
        area.width.saturating_sub(padding * 2),
        1,
    )
}

fn top_bordered_content_line(area: Rect, horizontal_padding: u16) -> Rect {
    if area.is_empty() {
        return area;
    }

    let padding = padded_width(area, horizontal_padding);
    let y = if area.height <= 1 { area.y } else { area.y + 1 };

    Rect::new(
        area.x + padding,
        y,
        area.width.saturating_sub(padding * 2),
        1,
    )
}

fn padded_width(area: Rect, horizontal_padding: u16) -> u16 {
    if area.height <= 1 {
        0
    } else {
        horizontal_padding.min(area.width / 2)
    }
}

fn buffer_to_plain_text(buffer: &Buffer) -> String {
    let width = buffer.area.width as usize;
    let mut out = String::new();
    for row in buffer.content.chunks(width) {
        for cell in row {
            out.push_str(cell.symbol());
        }
        out.push('\n');
    }
    out
}

fn compact_path(path: &str) -> String {
    path.rsplit_once('/').map_or_else(
        || path.to_string(),
        |(_, basename)| format!(".../{basename}"),
    )
}

fn compact_work_detail(detail: &str) -> String {
    if detail.contains("approval") {
        "waiting for approval".to_string()
    } else if detail.contains("linting") {
        "linting project".to_string()
    } else {
        detail.to_string()
    }
}

pub(super) fn trim_to_width(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}
