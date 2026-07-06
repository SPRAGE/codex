use crate::motion::MotionMode;
use crate::motion::ReducedMotionIndicator;
use crate::motion::rotating_activity_indicator;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_lines;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use super::RedesignChromeContext;
use super::draw_horizontal_rule;
use super::render_line;

const COMPOSER_TOP_RULE_ROWS: u16 = 1;
const COMPOSER_BOTTOM_RULE_ROWS: u16 = 1;
pub(super) const COMPOSER_CHROME_ROWS: u16 = COMPOSER_TOP_RULE_ROWS + COMPOSER_BOTTOM_RULE_ROWS;
pub(super) const COMPOSER_ROWS: u16 = COMPOSER_CHROME_ROWS + 1;
const COMPOSER_LABEL: &str = "MSG> ";
const COMPOSER_PLACEHOLDER: &str = "Describe the next change...";
const MESSAGE_QUEUE_MAX_ROWS: usize = 4;
const MESSAGE_QUEUE_MESSAGE_LINE_LIMIT: usize = 2;

pub(super) fn render_work_status_line(context: &RedesignChromeContext) -> Option<Line<'static>> {
    if !context.working {
        return None;
    }

    if let Some(line) = &context.work_status_line {
        return Some(line.clone());
    }

    let mut spans = Vec::new();
    if let Some(indicator) = work_activity_indicator(context) {
        spans.push(indicator);
        spans.push(" ".into());
    }
    spans.push("Working".into());
    Some(Line::from(spans))
}

pub(super) fn render_composer(
    area: Rect,
    buf: &mut Buffer,
    draft: &str,
    draft_cursor: usize,
    queued_messages: &[String],
    work_status_line: Option<&Line<'static>>,
) -> Option<(u16, u16)> {
    if area.is_empty() {
        return None;
    }

    let minimum_composer_height = COMPOSER_ROWS.min(area.height);
    let status_height = u16::from(work_status_line.is_some())
        .min(area.height.saturating_sub(minimum_composer_height));
    let queue_height = message_queue_desired_height(area.width, queued_messages).min(
        area.height
            .saturating_sub(minimum_composer_height)
            .saturating_sub(status_height),
    );
    if queue_height > 0 {
        let queue_area = Rect::new(area.x, area.y, area.width, queue_height);
        render_message_queue(queue_area, buf, queued_messages);
    }
    if status_height > 0
        && let Some(line) = work_status_line
    {
        let status_area = Rect::new(
            area.x,
            area.y.saturating_add(queue_height),
            area.width,
            status_height,
        );
        render_work_status(status_area, buf, line);
    }

    let area = Rect::new(
        area.x,
        area.y
            .saturating_add(queue_height)
            .saturating_add(status_height),
        area.width,
        area.height.saturating_sub(queue_height + status_height),
    );
    if area.is_empty() {
        return None;
    }

    draw_horizontal_rule(area, buf, area.y);
    draw_horizontal_rule(area, buf, area.bottom().saturating_sub(1));

    let input_height = area.height.saturating_sub(COMPOSER_CHROME_ROWS);
    if input_height == 0 {
        return None;
    }

    let input_y = area.y.saturating_add(COMPOSER_TOP_RULE_ROWS);
    let input_area = Rect::new(area.x, input_y, area.width, input_height);
    buf.set_style(input_area, Style::default().bg(Color::Reset));
    let prefix_width = composer_prefix_width();
    let lines = composer_input_lines(area.width, draft);
    let (cursor_line_idx, cursor_width) =
        composer_cursor_line_and_width(area.width, draft, draft_cursor, prefix_width);
    let visible_height = usize::from(input_height);
    let max_visible_start = lines.len().saturating_sub(visible_height);
    let mut visible_start = max_visible_start;
    if cursor_line_idx < visible_start {
        visible_start = cursor_line_idx;
    } else if cursor_line_idx >= visible_start + visible_height {
        visible_start = cursor_line_idx + 1 - visible_height;
    }
    visible_start = visible_start.min(max_visible_start);
    let visible_lines = lines
        .iter()
        .skip(visible_start)
        .cloned()
        .collect::<Vec<_>>();
    Paragraph::new(visible_lines).render(input_area, buf);

    let cursor_y = input_area.y.saturating_add(
        cursor_line_idx
            .saturating_sub(visible_start)
            .min(input_height.saturating_sub(1) as usize) as u16,
    );
    let cursor_x = input_area
        .x
        .saturating_add(cursor_width as u16)
        .min(input_area.right().saturating_sub(1));
    Some((cursor_x, cursor_y))
}

pub(super) fn composer_desired_height(
    width: u16,
    draft: &str,
    queued_messages: &[String],
    work_status_visible: bool,
) -> u16 {
    if width == 0 {
        return 0;
    }

    let prefix_width = UnicodeWidthStr::width(COMPOSER_LABEL);
    let line_count =
        u16::try_from(composer_line_count(width, draft, prefix_width)).unwrap_or(u16::MAX);
    message_queue_desired_height(width, queued_messages)
        .saturating_add(u16::from(work_status_visible))
        .saturating_add(COMPOSER_CHROME_ROWS)
        .saturating_add(line_count)
}

#[cfg(test)]
pub(super) fn render_composer_cursor(
    width: u16,
    height: u16,
    draft: &str,
    draft_cursor: usize,
) -> (u16, u16) {
    let area = Rect::new(0, 0, width, height);
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("terminal");
    let mut cursor = None;
    terminal
        .draw(|frame| {
            cursor = render_composer(area, frame.buffer_mut(), draft, draft_cursor, &[], None);
        })
        .expect("draw");
    cursor.expect("cursor")
}

pub(super) fn work_activity_indicator(context: &RedesignChromeContext) -> Option<Span<'static>> {
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

fn message_queue_desired_height(width: u16, queued_messages: &[String]) -> u16 {
    if width == 0 || queued_messages.is_empty() {
        return 0;
    }

    1 + u16::try_from(message_queue_lines(width, queued_messages).len()).unwrap_or(u16::MAX)
}

fn render_message_queue(area: Rect, buf: &mut Buffer, queued_messages: &[String]) {
    if area.is_empty() || queued_messages.is_empty() {
        return;
    }

    draw_horizontal_rule(area, buf, area.y);
    let content_area = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    if content_area.is_empty() {
        return;
    }

    let lines = message_queue_lines(content_area.width, queued_messages)
        .into_iter()
        .take(content_area.height as usize)
        .collect::<Vec<_>>();
    Paragraph::new(lines).render(content_area, buf);
}

fn message_queue_lines(width: u16, queued_messages: &[String]) -> Vec<Line<'static>> {
    if width == 0 || queued_messages.is_empty() {
        return Vec::new();
    }

    let mut lines = vec![Line::from(vec![
        "QUE> ".cyan().bold(),
        queued_message_count_label(queued_messages.len()).dim(),
    ])];

    for message in queued_messages {
        let wrapped = adaptive_wrap_lines(
            message.lines().map(|line| Line::from(line.dim().italic())),
            RtOptions::new(width as usize)
                .initial_indent(Line::from("  ↳ ".dim()))
                .subsequent_indent(Line::from("    ")),
        );
        let wrapped_len = wrapped.len();
        lines.extend(wrapped.into_iter().take(MESSAGE_QUEUE_MESSAGE_LINE_LIMIT));
        if wrapped_len > MESSAGE_QUEUE_MESSAGE_LINE_LIMIT {
            lines.push(Line::from("    …".dim().italic()));
        }
        if lines.len() >= MESSAGE_QUEUE_MAX_ROWS {
            break;
        }
    }

    if lines.len() > MESSAGE_QUEUE_MAX_ROWS {
        lines.truncate(MESSAGE_QUEUE_MAX_ROWS);
    }
    if queued_messages.len() > 1
        && lines.len() == MESSAGE_QUEUE_MAX_ROWS
        && queued_messages.len() > rendered_queue_message_count(&lines)
    {
        let remaining = queued_messages
            .len()
            .saturating_sub(rendered_queue_message_count(&lines));
        if remaining > 0
            && let Some(last) = lines.last_mut()
        {
            *last = Line::from(format!("  +{remaining} more queued").dim());
        }
    }

    lines
}

fn queued_message_count_label(count: usize) -> String {
    if count == 1 {
        "1 queued message".to_string()
    } else {
        format!("{count} queued messages")
    }
}

fn rendered_queue_message_count(lines: &[Line<'_>]) -> usize {
    lines
        .iter()
        .filter(|line| {
            line.spans
                .first()
                .is_some_and(|span| span.content.as_ref().starts_with("  ↳ "))
        })
        .count()
}

fn render_work_status(area: Rect, buf: &mut Buffer, line: &Line<'static>) {
    if area.is_empty() {
        return;
    }
    render_line(area, buf, area.y, line.clone());
}

fn composer_cursor_line_and_width(
    width: u16,
    draft: &str,
    draft_cursor: usize,
    prefix_width: usize,
) -> (usize, usize) {
    if draft.is_empty() {
        return (0, prefix_width);
    }

    let mut cursor = draft_cursor.min(draft.len());
    while !draft.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let mut line_count = 0usize;
    let mut cursor_width = prefix_width;
    for source_line in draft[..cursor].split('\n') {
        let segments = composer_wrap_segments(width, source_line, prefix_width);
        line_count += segments.len();
        cursor_width = segments
            .last()
            .map(|segment| prefix_width + UnicodeWidthStr::width(segment.as_str()))
            .unwrap_or(prefix_width);
    }

    (line_count.saturating_sub(1), cursor_width)
}

fn composer_input_lines(width: u16, draft: &str) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }

    let prefix_width = composer_prefix_width();
    let prefix = vec![COMPOSER_LABEL.magenta().bold()];

    if draft.is_empty() {
        return composer_wrap_line(
            width,
            Line::from(prefix),
            COMPOSER_PLACEHOLDER,
            Style::new().dim(),
            prefix_width,
        );
    }

    let mut lines = Vec::new();
    for (idx, source_line) in draft.split('\n').enumerate() {
        let prefix = if idx == 0 {
            Line::from(prefix.clone())
        } else {
            Line::from(" ".repeat(prefix_width))
        };
        lines.extend(composer_wrap_line(
            width,
            prefix,
            source_line,
            Style::new(),
            prefix_width,
        ));
    }
    lines
}

fn composer_wrap_line(
    width: u16,
    first_prefix: Line<'static>,
    text: &str,
    text_style: Style,
    subsequent_prefix_width: usize,
) -> Vec<Line<'static>> {
    let mut prefix = first_prefix;
    composer_wrap_segments(width, text, subsequent_prefix_width)
        .into_iter()
        .map(|segment| {
            let line = composer_line_with_segment(prefix.clone(), segment, text_style);
            prefix = Line::from(" ".repeat(subsequent_prefix_width));
            line
        })
        .collect()
}

fn composer_wrap_segments(width: u16, text: &str, prefix_width: usize) -> Vec<String> {
    let capacity = (width as usize).saturating_sub(prefix_width).max(1);
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut segment_width = 0usize;

    for ch in text.chars() {
        let ch_start = segment.len();
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        segment.push(ch);
        segment_width = segment_width.saturating_add(ch_width);
        if segment_width <= capacity {
            continue;
        }

        if !ch.is_whitespace()
            && let Some((break_start, break_end)) = composer_last_separator(&segment)
        {
            let next_segment = segment[break_end..].to_string();
            segment.truncate(break_start);
            segments.push(std::mem::take(&mut segment));
            segment = next_segment;
        } else if ch_start > 0 {
            let next_segment = segment[ch_start..].to_string();
            segment.truncate(ch_start);
            segments.push(std::mem::take(&mut segment));
            segment = next_segment;
        } else {
            segments.push(std::mem::take(&mut segment));
        }

        segment_width = UnicodeWidthStr::width(segment.as_str());
    }

    segments.push(segment);
    segments
}

fn composer_last_separator(text: &str) -> Option<(usize, usize)> {
    let mut separator_start = None;
    let mut last_separator = None;

    for (idx, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if separator_start.is_none() && idx > 0 {
                separator_start = Some(idx);
            }
        } else if let Some(start) = separator_start.take() {
            last_separator = Some((start, idx));
        }
    }

    last_separator
}

fn composer_line_with_segment(
    prefix: Line<'static>,
    segment: String,
    text_style: Style,
) -> Line<'static> {
    let style = prefix.style;
    let mut spans = prefix.spans;
    if !segment.is_empty() {
        spans.push(Span::styled(segment, text_style));
    }
    Line::from(spans).style(style)
}

fn composer_line_count(width: u16, draft: &str, prefix_width: usize) -> usize {
    let text = if draft.is_empty() {
        COMPOSER_PLACEHOLDER
    } else {
        draft
    };
    text.split('\n')
        .map(|line| composer_wrap_segments(width, line, prefix_width).len())
        .sum::<usize>()
        .max(1)
}

fn composer_prefix_width() -> usize {
    UnicodeWidthStr::width(COMPOSER_LABEL)
}
