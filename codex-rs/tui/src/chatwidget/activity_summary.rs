//! Compact live-activity summaries for the bottom status indicator.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use codex_app_server_protocol::CommandExecutionSource as ExecCommandSource;
use codex_app_server_protocol::McpToolCallAppContext;
use codex_protocol::parse_command::ParsedCommand;

use crate::diff_model::FileChange;
use crate::status_indicator_widget::STATUS_DETAILS_DEFAULT_MAX_LINES;
use crate::terminal_display_sanitize::sanitize_terminal_display_text;
use crate::text_formatting::truncate_text;

const HEADER_MAX_GRAPHEMES: usize = 64;
const DETAIL_MAX_GRAPHEMES: usize = 96;
const PATCH_DETAIL_LIMIT: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActivityStatus {
    pub(super) header: String,
    pub(super) details: Option<String>,
    pub(super) details_max_lines: usize,
}

impl ActivityStatus {
    pub(super) fn thinking() -> Self {
        Self::new(
            "Thinking".to_string(),
            Some("Planning next step".to_string()),
        )
    }

    pub(super) fn command(
        command: &[String],
        parsed_commands: &[ParsedCommand],
        source: ExecCommandSource,
        cwd: &Path,
    ) -> Self {
        let command_line = command_line(command);
        let header = parsed_command_header(parsed_commands, cwd)
            .unwrap_or_else(|| classified_command_header(command, source));
        let details = non_duplicate_detail(&command_line, &header);
        Self::new(header, details)
    }

    pub(super) fn patch(changes: &HashMap<PathBuf, FileChange>, cwd: &Path) -> Self {
        if changes.is_empty() {
            return Self::new("Editing files".to_string(), None);
        }

        let mut files = changes
            .keys()
            .map(|path| display_path(path, cwd))
            .collect::<Vec<_>>();
        files.sort();

        let header = if files.len() == 1 {
            format!("Editing {}", files[0])
        } else {
            format!("Editing {} files", files.len())
        };
        let details = if files.len() == 1 {
            None
        } else {
            let mut visible = files
                .iter()
                .take(PATCH_DETAIL_LIMIT)
                .cloned()
                .collect::<Vec<_>>();
            let remaining = files.len().saturating_sub(PATCH_DETAIL_LIMIT);
            if remaining > 0 {
                visible.push(format!("+{remaining} more"));
            }
            Some(truncate_text(&visible.join(", "), DETAIL_MAX_GRAPHEMES))
        };
        Self::new(header, details)
    }

    pub(super) fn mcp(
        server: &str,
        tool: &str,
        app_context: Option<&McpToolCallAppContext>,
    ) -> Self {
        let details = Some(format!("{server}.{tool}"));
        let header = match app_context {
            Some(context) => match (
                context
                    .app_name
                    .as_deref()
                    .filter(|value| !value.is_empty()),
                context
                    .action_name
                    .as_deref()
                    .filter(|value| !value.is_empty()),
            ) {
                (Some(app), Some(action)) => format!("Using {app}: {action}"),
                (Some(app), None) => format!("Using {app}"),
                (None, Some(action)) => format!("Calling {action}"),
                (None, None) => format!("Calling MCP {tool}"),
            },
            None => format!("Calling MCP {tool}"),
        };
        Self::new(header, details)
    }

    fn new(header: String, details: Option<String>) -> Self {
        Self {
            header: truncate_text(header.trim(), HEADER_MAX_GRAPHEMES),
            details: details
                .map(|details| truncate_text(details.trim(), DETAIL_MAX_GRAPHEMES))
                .filter(|details| !details.is_empty()),
            details_max_lines: STATUS_DETAILS_DEFAULT_MAX_LINES,
        }
    }
}

fn parsed_command_header(parsed_commands: &[ParsedCommand], cwd: &Path) -> Option<String> {
    parsed_commands.iter().find_map(|parsed| match parsed {
        ParsedCommand::Read { path, .. } => Some(format!("Reading {}", display_path(path, cwd))),
        ParsedCommand::ListFiles { path, .. } => Some(match path {
            Some(path) if !path.is_empty() => format!("Listing {path}"),
            Some(_) | None => "Listing files".to_string(),
        }),
        ParsedCommand::Search { query, path, .. } => {
            let query = query.as_deref().filter(|query| !query.is_empty());
            let path = path.as_deref().filter(|path| !path.is_empty());
            match (query, path) {
                (Some(query), Some(path)) => Some(format!(
                    "Searching {path} for \"{}\"",
                    truncate_text(query, 32)
                )),
                (Some(query), None) => {
                    Some(format!("Searching for \"{}\"", truncate_text(query, 40)))
                }
                (None, Some(path)) => Some(format!("Searching {path}")),
                (None, None) => Some("Searching".to_string()),
            }
        }
        ParsedCommand::Unknown { .. } => None,
    })
}

fn classified_command_header(command: &[String], source: ExecCommandSource) -> String {
    let Some(program) = command.first().map(String::as_str) else {
        return "Running command".to_string();
    };
    let program = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program);
    let program_lower = program.to_ascii_lowercase();
    let args = command
        .iter()
        .skip(1)
        .map(|arg| arg.to_ascii_lowercase())
        .collect::<Vec<_>>();

    if matches!(source, ExecCommandSource::UserShell) {
        return "Running shell command".to_string();
    }
    if command_mentions(&args, &["test", "nextest", "pytest", "vitest", "jest"]) {
        return "Running tests".to_string();
    }
    if command_mentions(&args, &["fmt", "format", "rustfmt", "prettier"])
        || program_lower == "rustfmt"
    {
        return "Formatting".to_string();
    }
    if command_mentions(&args, &["fix", "clippy", "lint"]) || program_lower == "clippy" {
        return "Linting".to_string();
    }
    if command_mentions(&args, &["check", "build"]) {
        return "Checking build".to_string();
    }
    if matches!(program_lower.as_str(), "rg" | "grep" | "ag") {
        return "Searching files".to_string();
    }
    if matches!(
        program_lower.as_str(),
        "cat" | "sed" | "nl" | "head" | "tail"
    ) {
        return "Reading files".to_string();
    }
    if matches!(program_lower.as_str(), "ls" | "find" | "fd" | "tree") {
        return "Listing files".to_string();
    }
    if program_lower == "git" {
        return match args.first().map(String::as_str) {
            Some("status") => "Checking git status".to_string(),
            Some("diff") | Some("show") => "Inspecting git diff".to_string(),
            Some("log") => "Reading git history".to_string(),
            Some("add") => "Staging changes".to_string(),
            Some("commit") => "Creating commit".to_string(),
            Some("push") => "Pushing branch".to_string(),
            _ => "Running git".to_string(),
        };
    }

    format!("Running {program}")
}

fn command_mentions(args: &[String], needles: &[&str]) -> bool {
    args.iter().any(|arg| {
        needles
            .iter()
            .any(|needle| arg == needle || arg.contains(needle))
    })
}

fn command_line(command: &[String]) -> String {
    sanitize_terminal_display_text(&command.join(" "))
}

fn non_duplicate_detail(detail: &str, header: &str) -> Option<String> {
    let detail = truncate_text(detail.trim(), DETAIL_MAX_GRAPHEMES);
    (!detail.is_empty() && detail != header).then_some(detail)
}

fn display_path(path: &Path, cwd: &Path) -> String {
    let path = path.strip_prefix(cwd).unwrap_or(path);
    let display = path.display().to_string();
    if display.is_empty() {
        ".".to_string()
    } else {
        truncate_text(&display, HEADER_MAX_GRAPHEMES)
    }
}
