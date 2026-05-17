use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use codex_utils_cli::ApprovalModeCliArg;
use codex_utils_cli::CliConfigOverrides;
use codex_utils_cli::SharedCliOptions;

#[derive(Parser, Clone, Debug)]
#[command(version)]
pub struct Cli {
    /// Optional user prompt to start the session.
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,

    /// Error out when config.toml contains fields that are not recognized by this version of Codex.
    #[arg(long = "strict-config", default_value_t = false)]
    pub strict_config: bool,

    // Internal controls set by the top-level `codex resume` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub resume_picker: bool,

    #[clap(skip)]
    pub resume_last: bool,

    /// Internal: resume a specific recorded session by id (UUID). Set by the
    /// top-level `codex resume <SESSION_ID>` wrapper; not exposed as a public flag.
    #[clap(skip)]
    pub resume_session_id: Option<String>,

    /// Internal: show all sessions (disables cwd filtering and shows CWD column).
    #[clap(skip)]
    pub resume_show_all: bool,

    /// Internal: include non-interactive sessions in resume listings.
    #[clap(skip)]
    pub resume_include_non_interactive: bool,

    // Internal controls set by the top-level `codex fork` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub fork_picker: bool,

    #[clap(skip)]
    pub fork_last: bool,

    /// Internal: fork a specific recorded session by id (UUID). Set by the
    /// top-level `codex fork <SESSION_ID>` wrapper; not exposed as a public flag.
    #[clap(skip)]
    pub fork_session_id: Option<String>,

    /// Internal: show all sessions (disables cwd filtering and shows CWD column).
    #[clap(skip)]
    pub fork_show_all: bool,

    #[clap(flatten)]
    pub shared: TuiSharedCliOptions,

    /// Configure when the model requires human approval before executing a command.
    #[arg(long = "ask-for-approval", short = 'a')]
    pub approval_policy: Option<ApprovalModeCliArg>,

    /// Enable live web search. When enabled, the native Responses `web_search` tool is available to the model (no per‑call approval).
    #[arg(long = "search", default_value_t = false)]
    pub web_search: bool,

    /// Disable alternate screen mode
    ///
    /// Runs the TUI in inline mode, preserving terminal scrollback history.
    #[arg(long = "no-alt-screen", default_value_t = false)]
    pub no_alt_screen: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Use the redesigned full-screen TUI.
    #[arg(long = "redesign-tui", alias = "redesign_TUI", default_value_t = false)]
    pub redesign_tui: bool,

    /// Deprecated: the legacy inline chat UI is the default.
    #[arg(long = "legacy-ui", default_value_t = false, hide = true)]
    pub legacy_ui: bool,

    /// Deprecated: use --redesign-tui.
    #[arg(
        long = "experimental-redesigned-ui",
        default_value_t = false,
        hide = true
    )]
    pub experimental_redesigned_ui: bool,
}

impl Cli {
    pub(crate) fn redesigned_ui_enabled(&self) -> bool {
        self.redesign_tui || self.experimental_redesigned_ui
    }
}

impl std::ops::Deref for Cli {
    type Target = SharedCliOptions;

    fn deref(&self) -> &Self::Target {
        &self.shared.0
    }
}

impl std::ops::DerefMut for Cli {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct TuiSharedCliOptions(SharedCliOptions);

impl TuiSharedCliOptions {
    pub fn into_inner(self) -> SharedCliOptions {
        self.0
    }
}

impl std::ops::Deref for TuiSharedCliOptions {
    type Target = SharedCliOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TuiSharedCliOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Args for TuiSharedCliOptions {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        mark_tui_args(SharedCliOptions::augment_args(cmd))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        mark_tui_args(SharedCliOptions::augment_args_for_update(cmd))
    }
}

impl FromArgMatches for TuiSharedCliOptions {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        SharedCliOptions::from_arg_matches(matches).map(Self)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        self.0.update_from_arg_matches(matches)
    }
}

fn mark_tui_args(cmd: clap::Command) -> clap::Command {
    cmd.mut_arg("dangerously_bypass_approvals_and_sandbox", |arg| {
        arg.conflicts_with("approval_policy")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redesigned_ui_is_disabled_by_default() {
        let cli = Cli::try_parse_from(["codex"]).expect("valid cli");

        assert!(!cli.redesigned_ui_enabled());
    }

    #[test]
    fn redesign_tui_flag_enables_redesign() {
        let cli = Cli::try_parse_from(["codex", "--redesign-tui"]).expect("valid cli");

        assert!(cli.redesigned_ui_enabled());
    }

    #[test]
    fn redesign_tui_underscore_alias_enables_redesign() {
        let cli = Cli::try_parse_from(["codex", "--redesign_TUI"]).expect("valid cli");

        assert!(cli.redesigned_ui_enabled());
    }

    #[test]
    fn legacy_ui_flag_keeps_redesign_disabled() {
        let cli = Cli::try_parse_from(["codex", "--legacy-ui"]).expect("valid cli");

        assert!(!cli.redesigned_ui_enabled());
    }

    #[test]
    fn old_experimental_flag_still_enables_redesign() {
        let cli =
            Cli::try_parse_from(["codex", "--experimental-redesigned-ui"]).expect("valid cli");

        assert!(cli.redesigned_ui_enabled());
    }
}
