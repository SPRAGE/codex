use crate::chatwidget::RedesignBackgroundTerminal;
use crate::chatwidget::RedesignBackgroundTerminalStatus;
use crate::terminal_display_sanitize::sanitize_terminal_display_text;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_lines;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use super::truncate_text;
use super::window;

pub(super) fn render_window(
    area: Rect,
    buf: &mut Buffer,
    terminals: &[RedesignBackgroundTerminal],
    selected_idx: usize,
    expanded_idx: Option<usize>,
    scroll_offset: usize,
) {
    let Some(panel) = terminal_window_rect(area) else {
        return;
    };
    let block = window::overlay_block(" Terminal Sessions  Enter view  Esc collapse/close ");
    let inner = block.inner(panel);
    let selected_idx = clamp_terminal_index(selected_idx, terminals.len());
    let expanded_idx = expanded_idx.filter(|idx| *idx < terminals.len());
    let lines = terminal_window_display_lines(terminals, inner.width, selected_idx, expanded_idx);
    let scroll_limit = if expanded_idx.is_some() {
        lines.len().saturating_sub(inner.height as usize)
    } else {
        0
    };
    let window = if expanded_idx.is_some() {
        visible_window(
            lines,
            inner.height as usize,
            scroll_offset.min(scroll_limit),
        )
    } else {
        visible_window_around_selection(
            lines,
            inner.height as usize,
            selected_idx.saturating_mul(2),
        )
    };

    Clear.render(panel, buf);
    block.render(panel, buf);
    Paragraph::new(window.lines).render(inner, buf);
    render_scrollbar(
        inner,
        buf,
        window.hidden_above,
        window.hidden_below,
        scroll_offset.min(scroll_limit),
        scroll_limit,
    );
}

pub(super) fn scroll_limit(
    area: Rect,
    terminals: &[RedesignBackgroundTerminal],
    selected_idx: usize,
    expanded_idx: Option<usize>,
) -> usize {
    let Some(panel) = terminal_window_rect(area) else {
        return 0;
    };
    let inner_height = panel.height.saturating_sub(2) as usize;
    let inner_width = panel.width.saturating_sub(2);
    let selected_idx = clamp_terminal_index(selected_idx, terminals.len());
    let expanded_idx = expanded_idx.filter(|idx| *idx < terminals.len());
    if expanded_idx.is_none() {
        return 0;
    }
    terminal_window_display_lines(terminals, inner_width, selected_idx, expanded_idx)
        .len()
        .saturating_sub(inner_height)
}

fn terminal_window_rect(area: Rect) -> Option<Rect> {
    if area.width < 32 || area.height < 8 {
        return None;
    }

    let preferred_width = area.width.saturating_sub(2).clamp(32, 100);
    let width = if area.width.saturating_sub(preferred_width) <= 2 {
        area.width
    } else {
        preferred_width
    };
    let height = area.height.saturating_sub(2).clamp(8, 24);
    Some(Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    })
}

fn terminal_window_display_lines(
    terminals: &[RedesignBackgroundTerminal],
    width: u16,
    selected_idx: usize,
    expanded_idx: Option<usize>,
) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    if terminals.is_empty() {
        return vec!["No terminal sessions yet.".dim().italic().into()];
    }

    let mut lines = Vec::new();
    for (idx, terminal) in terminals.iter().enumerate() {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        let selected = idx == selected_idx;
        let expanded = expanded_idx == Some(idx);
        lines.push(terminal_header_line(
            idx, terminal, width, selected, expanded,
        ));
        if !expanded {
            continue;
        }
        if terminal.output_lines.is_empty() {
            lines.push(Line::from(vec![
                "  ".into(),
                "no output yet".dim().italic(),
            ]));
            continue;
        }

        let output_lines: Vec<Line<'static>> = terminal
            .output_lines
            .iter()
            .map(|line| sanitize_terminal_display_text(line))
            .filter(|line| !line.trim().is_empty())
            .map(Line::from)
            .collect();
        if output_lines.is_empty() {
            lines.push(Line::from(vec![
                "  ".into(),
                "no printable output yet".dim().italic(),
            ]));
            continue;
        }

        lines.extend(adaptive_wrap_lines(
            output_lines,
            RtOptions::new(width as usize)
                .initial_indent(Line::from("  │ ".dim()))
                .subsequent_indent(Line::from("    ".dim())),
        ));
    }
    lines
}

fn terminal_header_line(
    idx: usize,
    terminal: &RedesignBackgroundTerminal,
    width: u16,
    selected: bool,
    expanded: bool,
) -> Line<'static> {
    let cursor = if selected { ">" } else { " " };
    let marker = if expanded { "[-]" } else { "[+]" };
    let label = format!("{cursor} {marker} TERM {}", idx + 1);
    let status = terminal_status_span(terminal);
    let label_width = UnicodeWidthStr::width(label.as_str());
    let status_width = UnicodeWidthStr::width(status.content.as_ref());
    let command_width = width.saturating_sub(label_width as u16 + status_width as u16 + 4);
    let command_display = sanitize_terminal_display_text(&terminal.command_display);
    let line = Line::from(vec![
        Span::from(label).cyan().bold(),
        " ".into(),
        status,
        "  ".into(),
        truncate_text(&command_display, command_width).magenta(),
    ]);
    if selected {
        line.style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        line
    }
}

fn terminal_status_span(terminal: &RedesignBackgroundTerminal) -> Span<'static> {
    match terminal.status {
        RedesignBackgroundTerminalStatus::Running => "running".green(),
        RedesignBackgroundTerminalStatus::Completed => match terminal.exit_code {
            Some(0) | None => "done".dim(),
            Some(code) => Span::from(format!("exit {code}")).yellow(),
        },
        RedesignBackgroundTerminalStatus::Failed => match terminal.exit_code {
            Some(code) => Span::from(format!("failed {code}")).red(),
            None => "failed".red(),
        },
        RedesignBackgroundTerminalStatus::Declined => "declined".yellow(),
    }
}

fn clamp_terminal_index(idx: usize, terminal_count: usize) -> usize {
    idx.min(terminal_count.saturating_sub(1))
}

struct DisplayWindow {
    lines: Vec<Line<'static>>,
    hidden_above: bool,
    hidden_below: bool,
}

fn visible_window(lines: Vec<Line<'static>>, height: usize, scroll_offset: usize) -> DisplayWindow {
    let visible_start = lines
        .len()
        .saturating_sub(height.saturating_add(scroll_offset));
    let visible_end = visible_start.saturating_add(height).min(lines.len());
    DisplayWindow {
        hidden_above: visible_start > 0,
        hidden_below: visible_end < lines.len(),
        lines: lines
            .into_iter()
            .skip(visible_start)
            .take(visible_end.saturating_sub(visible_start))
            .collect(),
    }
}

fn visible_window_around_selection(
    lines: Vec<Line<'static>>,
    height: usize,
    selected_line_idx: usize,
) -> DisplayWindow {
    if height == 0 {
        return DisplayWindow {
            hidden_above: !lines.is_empty(),
            hidden_below: false,
            lines: Vec::new(),
        };
    }
    let selected_line_idx = selected_line_idx.min(lines.len().saturating_sub(1));
    let max_start = lines.len().saturating_sub(height);
    let visible_start = selected_line_idx
        .saturating_sub(height.saturating_sub(1))
        .min(max_start);
    let visible_end = visible_start.saturating_add(height).min(lines.len());
    DisplayWindow {
        hidden_above: visible_start > 0,
        hidden_below: visible_end < lines.len(),
        lines: lines
            .into_iter()
            .skip(visible_start)
            .take(visible_end.saturating_sub(visible_start))
            .collect(),
    }
}

fn render_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    hidden_above: bool,
    hidden_below: bool,
    scroll_offset: usize,
    scroll_limit: usize,
) {
    if area.width <= 2 || scroll_limit == 0 || (!hidden_above && !hidden_below) {
        return;
    }

    let scrollbar_x = area.right().saturating_sub(1);
    for y in area.y..area.bottom() {
        buf[(scrollbar_x, y)]
            .set_symbol("|")
            .set_style(Style::new().dim());
    }

    let thumb_range = area.height.saturating_sub(1) as usize;
    let thumb_offset =
        (scroll_limit - scroll_offset.min(scroll_limit)) * thumb_range / scroll_limit;
    let thumb_y = area
        .y
        .saturating_add(thumb_offset as u16)
        .min(area.bottom().saturating_sub(1));
    buf[(scrollbar_x, thumb_y)].set_symbol("#");
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn background_terminal_window_snapshot() {
        let terminals = vec![
            RedesignBackgroundTerminal {
                command_display: "cargo test -p codex-tui".to_string(),
                output_lines: vec![
                    "running 3 tests".to_string(),
                    "test redraw_updates_background_terminal_window ... ok".to_string(),
                    "test keeps_terminal_output_bounded ... ok".to_string(),
                ],
                status: RedesignBackgroundTerminalStatus::Running,
                exit_code: None,
            },
            RedesignBackgroundTerminal {
                command_display: "just fix -p codex-tui".to_string(),
                output_lines: Vec::new(),
                status: RedesignBackgroundTerminalStatus::Running,
                exit_code: None,
            },
        ];
        let mut terminal =
            Terminal::new(TestBackend::new(/*width*/ 80, /*height*/ 18)).expect("terminal");
        terminal
            .draw(|frame| {
                render_window(
                    frame.area(),
                    frame.buffer_mut(),
                    &terminals,
                    /*selected_idx*/ 0,
                    /*expanded_idx*/ Some(0),
                    /*scroll_offset*/ 0,
                );
            })
            .expect("draw");

        insta::with_settings!({snapshot_path => "../snapshots"}, {
            assert_snapshot!(
                "redesign_chrome_background_terminal_window_80x18",
                terminal.backend().to_string()
            );
        });
    }

    #[test]
    fn background_terminal_window_scroll_limit_uses_wrapped_output() {
        let terminals = vec![RedesignBackgroundTerminal {
            command_display: "rg query".to_string(),
            output_lines: (0..20).map(|idx| format!("line {idx}")).collect(),
            status: RedesignBackgroundTerminalStatus::Running,
            exit_code: None,
        }];
        let limit = scroll_limit(
            Rect::new(0, 0, 60, 12),
            &terminals,
            /*selected_idx*/ 0,
            /*expanded_idx*/ Some(0),
        );

        assert!(limit > 0);
    }

    #[test]
    fn background_terminal_window_strips_terminal_controls() {
        let terminals = vec![RedesignBackgroundTerminal {
            command_display: "\x1b]0;bad\x07cargo test".to_string(),
            output_lines: vec![
                "\x1b[2J\x1b[Hcleared".to_string(),
                "\x1b[31mred\x1b[0m\rnext".to_string(),
                "\x1bPqraw-sixel\x1b\\done".to_string(),
            ],
            status: RedesignBackgroundTerminalStatus::Running,
            exit_code: None,
        }];

        let rendered = terminal_window_display_lines(&terminals, 80, 0, Some(0))
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.into_owned())
            .collect::<String>();

        assert!(rendered.contains("cargo test"));
        assert!(rendered.contains("cleared"));
        assert!(rendered.contains("red next"));
        assert!(rendered.contains("done"));
        assert!(!rendered.contains('\x1b'));
        assert!(!rendered.chars().any(char::is_control));
    }

    #[test]
    fn background_terminal_window_only_renders_expanded_output() {
        let terminals = vec![
            RedesignBackgroundTerminal {
                command_display: "cargo test".to_string(),
                output_lines: vec!["cargo output".to_string()],
                status: RedesignBackgroundTerminalStatus::Running,
                exit_code: None,
            },
            RedesignBackgroundTerminal {
                command_display: "rg query".to_string(),
                output_lines: vec!["rg output".to_string()],
                status: RedesignBackgroundTerminalStatus::Running,
                exit_code: None,
            },
        ];

        let collapsed = terminal_window_display_lines(&terminals, 80, 0, None)
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.into_owned())
            .collect::<String>();
        assert!(collapsed.contains("cargo test"));
        assert!(collapsed.contains("rg query"));
        assert!(!collapsed.contains("cargo output"));
        assert!(!collapsed.contains("rg output"));

        let expanded = terminal_window_display_lines(&terminals, 80, 1, Some(1))
            .into_iter()
            .flat_map(|line| line.spans.into_iter())
            .map(|span| span.content.into_owned())
            .collect::<String>();
        assert!(expanded.contains("cargo test"));
        assert!(expanded.contains("rg query"));
        assert!(!expanded.contains("cargo output"));
        assert!(expanded.contains("rg output"));
    }
}
