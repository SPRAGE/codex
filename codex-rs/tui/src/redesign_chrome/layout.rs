use ratatui::layout::Rect;

use super::RedesignSidebarState;

const CHAT_SEPARATOR_ROWS: u16 = 1;
const CHAT_HEADER_ROWS: u16 = 2;
const FOOTER_ROWS: u16 = 2;
const WIDE_SIDE_WIDTH: u16 = 24;
const RIGHT_RAIL_WIDTH: u16 = 30;
const MIN_WIDE_WIDTH: u16 = 88;
const MIN_RIGHT_RAIL_WIDTH: u16 = 120;
const COMPACT_SIDE_WIDTH: u16 = 22;
const MIN_COMPACT_SIDEBAR_WIDTH: u16 = 56;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RedesignLayout {
    pub(super) side: Rect,
    pub(super) main: Rect,
    pub(super) right: Rect,
    pub(super) chat_separator: Rect,
    pub(super) chat_header: Rect,
    pub(super) transcript: Rect,
    pub(super) composer: Rect,
    pub(super) footer: Rect,
}

pub(super) fn available_chat_body_height(area: Rect) -> u16 {
    let available_body_height = area.height.saturating_sub(FOOTER_ROWS);
    available_body_height.saturating_sub(CHAT_SEPARATOR_ROWS + CHAT_HEADER_ROWS)
}

pub(super) fn layout_for_dimensions(area: Rect, composer_height: u16) -> RedesignLayout {
    layout_for_dimensions_with_side(area, side_width(area.width), composer_height)
}

pub(super) fn layout_for_dimensions_with_side(
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
    let body_y = area.y;
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
    let chat_header_height = CHAT_HEADER_ROWS.min(main.height);
    let chat_header = Rect::new(main.x, main.y, main.width, chat_header_height);
    let chat_separator_y = main.y.saturating_add(chat_header_height);
    let chat_separator_height =
        CHAT_SEPARATOR_ROWS.min(main.height.saturating_sub(chat_header_height));
    let chat_separator = Rect::new(main.x, chat_separator_y, main.width, chat_separator_height);
    let chat_body_y = chat_separator_y.saturating_add(chat_separator_height);
    let chat_body_height = main
        .height
        .saturating_sub(chat_header_height + chat_separator_height);
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

fn side_width(width: u16) -> u16 {
    if width >= MIN_WIDE_WIDTH {
        WIDE_SIDE_WIDTH
    } else {
        0
    }
}

pub(super) fn right_rail_width(width: u16, side_width: u16) -> u16 {
    let center_width = width.saturating_sub(side_width + RIGHT_RAIL_WIDTH);
    if width >= MIN_RIGHT_RAIL_WIDTH && center_width >= 64 {
        RIGHT_RAIL_WIDTH
    } else {
        0
    }
}

pub(super) fn side_width_for_state(width: u16, sidebar: RedesignSidebarState) -> u16 {
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
