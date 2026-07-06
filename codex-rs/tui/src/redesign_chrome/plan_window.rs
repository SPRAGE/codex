use crate::app::App;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::window;

pub(super) fn render_plan_window_from_app(area: Rect, buf: &mut Buffer, app: &App) {
    if !app.redesign_plan_window_open_for_active_chat() || area.width < 24 || area.height < 8 {
        return;
    }

    let Some(panel) = overlay_rect(area) else {
        return;
    };
    let content_width = panel.width.saturating_sub(4).max(1);
    let mut lines = if let Some(lines) = app
        .chat_widget
        .redesign_latest_plan_display_lines(content_width)
    {
        lines
    } else {
        vec![
            "No plan for this chat yet.".dim().italic().into(),
            "Waiting for update_plan or Plan mode output.".dim().into(),
        ]
    };

    let max_content_rows = panel.height.saturating_sub(2) as usize;
    if lines.len() > max_content_rows {
        lines.truncate(max_content_rows.saturating_sub(1));
        lines.push("...".dim().into());
    }

    render_plan_window(panel, lines, buf);
}

#[cfg(test)]
pub(super) fn plan_window_rect(area: Rect) -> Option<Rect> {
    overlay_rect(area)
}

fn overlay_rect(area: Rect) -> Option<Rect> {
    let width = area.width.saturating_sub(4).clamp(24, 72);
    let height = area.height.saturating_sub(4).clamp(6, 16);
    if width > area.width || height > area.height {
        return None;
    }
    Some(Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 3,
        width,
        height,
    })
}

fn render_plan_window(panel: Rect, lines: Vec<Line<'static>>, buf: &mut Buffer) {
    let block = window::overlay_block(" Plan  Alt-P close ");
    let inner = block.inner(panel);
    Clear.render(panel, buf);
    block.render(panel, buf);
    Paragraph::new(lines).render(inner, buf);
}
