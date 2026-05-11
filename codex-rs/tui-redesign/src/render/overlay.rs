use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::message_label;
use super::trim_to_width;
use crate::CommandChoice;
use crate::Overlay;
use crate::RedesignState;
use crate::TranscriptEntry;
use crate::theme;

pub(super) fn render_overlay(area: Rect, buf: &mut Buffer, state: &RedesignState) {
    if state.overlay == Overlay::None || area.width < 40 || area.height < 12 {
        return;
    }

    let (title, lines, height) = match state.overlay {
        Overlay::None => return,
        Overlay::Commands => ("COMMANDS", command_overlay_lines(state), 12),
        Overlay::Help => (
            "SHORTCUTS",
            vec![
                Line::from(vec![
                    Span::styled("Tab / Shift+Tab", theme::secondary_fixed()),
                    "  move focus".into(),
                ]),
                Line::from(vec![
                    Span::styled("Enter", theme::secondary_fixed()),
                    "  submit composer or select approval action".into(),
                ]),
                Line::from(vec![
                    Span::styled("Left / Right", theme::secondary_fixed()),
                    "  choose approval action".into(),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+R", theme::secondary_fixed()),
                    "  history overlay".into(),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+T", theme::secondary_fixed()),
                    "  transcript overlay".into(),
                ]),
                Line::from(vec![
                    Span::styled("1 / 2 / 3", theme::secondary_fixed()),
                    "  idle / running / approval demo".into(),
                ]),
                Line::from(vec![
                    Span::styled("F2 / Ctrl+P", theme::secondary_fixed()),
                    "  command palette".into(),
                ]),
                Line::from(vec![
                    Span::styled("Esc / q", theme::secondary_fixed()),
                    "  close overlay or exit".into(),
                ]),
            ],
            12,
        ),
        Overlay::History => (
            "HISTORY",
            vec![
                Line::from("Recent prompts"),
                Line::from(vec![
                    Span::styled("1 ", theme::metadata()),
                    "Redesign terminal UI from Stitch".into(),
                ]),
                Line::from(vec![
                    Span::styled("2 ", theme::metadata()),
                    "Check approval flow ergonomics".into(),
                ]),
                Line::from(vec![
                    Span::styled("3 ", theme::metadata()),
                    "Compare compact footer shortcuts".into(),
                ]),
                Line::from(""),
                Line::from(Span::styled("Esc closes", theme::metadata())),
            ],
            10,
        ),
        Overlay::Transcript => (
            "TRANSCRIPT",
            transcript_overlay_lines(&state.transcript),
            12,
        ),
    };

    let popup = centered_rect(area, area.width.min(78), height);
    Clear.render(popup, buf);
    let block = Block::new()
        .title(Span::styled(format!(" {title} "), theme::focus()))
        .borders(Borders::ALL)
        .border_style(theme::focus())
        .style(theme::surface_lowest());
    let inner = block.inner(popup);
    block.render(popup, buf);

    Paragraph::new(Text::from(lines))
        .style(theme::surface_lowest())
        .render(inner, buf);
}

fn command_overlay_lines(state: &RedesignState) -> Vec<Line<'static>> {
    let query = if state.command_query.is_empty() {
        Span::styled("type to filter", theme::metadata())
    } else {
        Span::styled(state.command_query.clone(), theme::secondary_fixed())
    };
    let mut lines = vec![
        Line::from(vec![Span::styled("CMD> ", theme::focus()), query]),
        Line::from(""),
    ];

    let commands = state.visible_commands();
    if commands.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching commands",
            theme::metadata(),
        )));
    } else {
        lines.extend(
            commands
                .into_iter()
                .map(|choice| command_line(choice, state.command_choice)),
        );
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Type filter  Up/Down choose  Enter run  Esc close",
        theme::metadata(),
    )));
    lines
}

fn command_line(choice: CommandChoice, selected: CommandChoice) -> Line<'static> {
    let marker = if choice == selected { ">" } else { " " };
    let name_style = if choice == selected {
        theme::selected()
    } else {
        theme::secondary_fixed()
    };

    Line::from(vec![
        Span::styled(marker, theme::focus()),
        " ".into(),
        Span::styled(choice.label(), name_style),
        "  ".into(),
        Span::styled(choice.description(), theme::metadata()),
    ])
}

fn transcript_overlay_lines(entries: &[TranscriptEntry]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for entry in entries.iter().rev().take(5).rev() {
        lines.push(Line::from(vec![
            message_label(entry.role),
            " ".into(),
            Span::styled(trim_to_width(&entry.text, 58), theme::metadata()),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No transcript entries yet",
            theme::metadata(),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Esc closes", theme::metadata())));
    lines
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
