//! Sanitization for untrusted terminal output shown inside the TUI.
//!
//! Command output can contain terminal control sequences. Rendering those bytes as
//! ordinary Ratatui text can cause the real terminal to interpret them while
//! Codex is drawing a frame, so display surfaces must strip them first.

use std::iter::Peekable;
use std::str::Chars;

pub(crate) fn sanitize_terminal_display_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\x1b' => skip_escape_sequence(&mut chars),
            '\u{009b}' => skip_csi_sequence(&mut chars),
            '\u{0090}' | '\u{0098}' | '\u{009d}' | '\u{009e}' | '\u{009f}' => {
                skip_string_control_sequence(&mut chars);
            }
            '\t' => output.push_str("    "),
            '\r' | '\n' => push_space_if_needed(&mut output),
            '\x08' => {
                output.pop();
            }
            ch if ch.is_control() || is_invisible_format_char(ch) => {}
            ch => output.push(ch),
        }
    }

    output
}

fn skip_escape_sequence(chars: &mut Peekable<Chars<'_>>) {
    match chars.next() {
        Some('[') => skip_csi_sequence(chars),
        Some(']') => skip_string_control_sequence(chars),
        Some('P' | 'X' | '^' | '_') => skip_string_control_sequence(chars),
        Some('(' | ')' | '*' | '+' | '-' | '.' | '/') => {
            // Character-set designation escapes have one following final byte.
            chars.next();
        }
        Some(_) | None => {}
    }
}

fn skip_csi_sequence(chars: &mut Peekable<Chars<'_>>) {
    while let Some(ch) = chars.next() {
        if ('\u{0040}'..='\u{007e}').contains(&ch) {
            break;
        }
        if ch == '\x1b' {
            skip_escape_sequence(chars);
            break;
        }
    }
}

fn skip_string_control_sequence(chars: &mut Peekable<Chars<'_>>) {
    let mut escaped = false;
    for ch in chars.by_ref() {
        if escaped {
            if ch == '\\' {
                break;
            }
            escaped = ch == '\x1b';
            continue;
        }

        match ch {
            '\x07' | '\u{009c}' => break,
            '\x1b' => escaped = true,
            _ => {}
        }
    }
}

fn push_space_if_needed(output: &mut String) {
    if !output.ends_with(' ') {
        output.push(' ');
    }
}

fn is_invisible_format_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{061c}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{feff}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_terminal_control_sequences() {
        let input = concat!(
            "before ",
            "\x1b]0;bad title\x07",
            "\x1b[2J",
            "\x1b[31mred\x1b[0m",
            "\rnext ",
            "\x1bPqraw-sixel\x1b\\",
            "done",
        );

        let sanitized = sanitize_terminal_display_text(input);

        assert_eq!(sanitized, "before red next done");
        assert!(!sanitized.contains('\x1b'));
        assert!(!sanitized.chars().any(char::is_control));
    }

    #[test]
    fn incomplete_escape_sequence_cannot_leak_escape_byte() {
        let sanitized = sanitize_terminal_display_text("ok \x1b[31");

        assert_eq!(sanitized, "ok ");
        assert!(!sanitized.contains('\x1b'));
    }
}
