use clap::Parser;
use codex_arg0::Arg0DispatchPaths;
use codex_arg0::arg0_dispatch_or_else;
use codex_config::LoaderOverrides;
use codex_tui::AppExitInfo;
use codex_tui::Cli;
use codex_tui::ExitReason;
use codex_tui::run_main;
use codex_utils_cli::CliConfigOverrides;
use codex_utils_cli::resume_command;
use std::ffi::OsString;
use supports_color::Stream;

fn format_exit_messages(exit_info: AppExitInfo, color_enabled: bool) -> Vec<String> {
    let AppExitInfo {
        token_usage,
        thread_id,
        ..
    } = exit_info;

    let mut lines = Vec::new();
    if !token_usage.is_zero() {
        lines.push(token_usage.to_string());
    }

    if let Some(resume_cmd) = resume_command(/*thread_name*/ None, thread_id) {
        let command = if color_enabled {
            format!("\u{1b}[36m{resume_cmd}\u{1b}[39m")
        } else {
            resume_cmd
        };
        lines.push(format!("To continue this session, run {command}"));
    }

    lines
}

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli =
            TopCli::parse_from(args_with_joined_prompt_after_terminator(std::env::args_os()));
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let exit_info = run_main(
            inner,
            arg0_paths,
            LoaderOverrides::default(),
            /*explicit_remote_endpoint*/ None,
        )
        .await?;
        match exit_info.exit_reason {
            ExitReason::Fatal(message) => {
                eprintln!("ERROR: {message}");
                std::process::exit(1);
            }
            ExitReason::UserRequested => {}
        }

        let color_enabled = supports_color::on(Stream::Stdout).is_some();
        for line in format_exit_messages(exit_info, color_enabled) {
            println!("{line}");
        }
        Ok(())
    })
}

fn args_with_joined_prompt_after_terminator(
    args: impl IntoIterator<Item = OsString>,
) -> Vec<OsString> {
    let args = args.into_iter().collect::<Vec<_>>();
    let Some(separator_index) = args.iter().position(|arg| arg == "--") else {
        return args;
    };
    if args.len().saturating_sub(separator_index + 1) <= 1 {
        return args;
    }

    let mut joined = String::new();
    for arg in &args[separator_index + 1..] {
        if !joined.is_empty() {
            joined.push(' ');
        }
        joined.push_str(&arg.to_string_lossy());
    }

    let mut normalized = args[..=separator_index].to_vec();
    normalized.push(OsString::from(joined));
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_prompt_words_after_option_terminator() {
        let args = args_with_joined_prompt_after_terminator(
            ["codex-tui", "--redesign-tui", "--", "resume", "optimize"].map(OsString::from),
        );

        let top_cli = TopCli::try_parse_from(args).expect("valid cli");
        assert_eq!(top_cli.inner.prompt.as_deref(), Some("resume optimize"));
    }

    #[test]
    fn leaves_single_prompt_after_option_terminator_unchanged() {
        let args = args_with_joined_prompt_after_terminator(
            ["codex-tui", "--redesign-tui", "--", "resume"].map(OsString::from),
        );

        let top_cli = TopCli::try_parse_from(args).expect("valid cli");
        assert_eq!(top_cli.inner.prompt.as_deref(), Some("resume"));
    }
}
