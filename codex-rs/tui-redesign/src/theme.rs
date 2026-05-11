use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;

pub(crate) const BACKGROUND: Color = Color::Black;
pub(crate) const ERROR: Color = Color::LightRed;
pub(crate) const ON_BACKGROUND: Color = Color::White;
pub(crate) const ON_SECONDARY_CONTAINER: Color = Color::Black;
pub(crate) const ON_SURFACE_VARIANT: Color = Color::Gray;
pub(crate) const OUTLINE: Color = Color::Gray;
pub(crate) const OUTLINE_VARIANT: Color = Color::DarkGray;
pub(crate) const PRIMARY: Color = Color::LightMagenta;
pub(crate) const PRIMARY_CONTAINER: Color = Color::Magenta;
pub(crate) const SECONDARY: Color = Color::White;
pub(crate) const SECONDARY_CONTAINER: Color = Color::Cyan;
pub(crate) const SECONDARY_FIXED: Color = Color::LightCyan;
pub(crate) const SURFACE_CONTAINER_HIGHEST: Color = Color::DarkGray;
pub(crate) const SURFACE_CONTAINER_LOW: Color = Color::DarkGray;
pub(crate) const SURFACE_CONTAINER_LOWEST: Color = Color::Black;
pub(crate) const TERTIARY: Color = Color::Yellow;
pub(crate) const TERTIARY_FIXED: Color = Color::LightYellow;

pub(crate) fn app() -> Style {
    Style::default().bg(BACKGROUND).fg(ON_BACKGROUND)
}

pub(crate) fn border() -> Style {
    Style::default().fg(OUTLINE_VARIANT)
}

pub(crate) fn footer() -> Style {
    Style::default()
        .fg(ON_BACKGROUND)
        .bg(SURFACE_CONTAINER_LOWEST)
}

pub(crate) fn metadata() -> Style {
    Style::default().fg(ON_SURFACE_VARIANT)
}

pub(crate) fn primary() -> Style {
    Style::default().fg(PRIMARY)
}

pub(crate) fn primary_container() -> Style {
    Style::default()
        .fg(PRIMARY_CONTAINER)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn secondary() -> Style {
    Style::default().fg(SECONDARY)
}

pub(crate) fn secondary_fixed() -> Style {
    Style::default().fg(SECONDARY_FIXED)
}

pub(crate) fn focus() -> Style {
    Style::default()
        .fg(SECONDARY_FIXED)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn selected() -> Style {
    Style::default()
        .fg(ON_SECONDARY_CONTAINER)
        .bg(SECONDARY_CONTAINER)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn side_nav() -> Style {
    Style::default()
        .fg(ON_SURFACE_VARIANT)
        .bg(SURFACE_CONTAINER_LOW)
}

pub(crate) fn side_nav_active() -> Style {
    Style::default()
        .fg(ON_SECONDARY_CONTAINER)
        .bg(SECONDARY_CONTAINER)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn surface_highest() -> Style {
    Style::default()
        .fg(ON_BACKGROUND)
        .bg(SURFACE_CONTAINER_HIGHEST)
}

pub(crate) fn surface_lowest() -> Style {
    Style::default()
        .fg(ON_SURFACE_VARIANT)
        .bg(SURFACE_CONTAINER_LOWEST)
}

pub(crate) fn tertiary() -> Style {
    Style::default().fg(TERTIARY)
}

pub(crate) fn tertiary_fixed() -> Style {
    Style::default()
        .fg(TERTIARY_FIXED)
        .add_modifier(Modifier::BOLD)
}
