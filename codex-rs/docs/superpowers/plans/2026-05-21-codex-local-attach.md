# Codex Local Attach Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a same-host SSH-friendly attach flow so a desktop Codex session can be continued from another terminal through the local app-server Unix socket.

**Architecture:** The first iteration reuses the existing app-server daemon, Unix socket transport, `thread/loaded/list`, `thread/read`, and `thread/resume` paths. `codex --attachable` guarantees the interactive session runs against the local daemon; `codex attach` connects to the daemon and resumes the most recently updated loaded interactive thread. It deliberately does not implement exclusive controller ownership yet.

**Tech Stack:** Rust, clap, Tokio, existing `codex-cli`, `codex-tui`, `codex-app-server-daemon`, `codex-app-server-client`, `codex-app-server-protocol`.

---

## File Structure

- Modify `tui/src/cli.rs`: add TUI-internal attach startup fields and the public `--attachable` flag.
- Modify `cli/src/main.rs`: add the `attach` subcommand, parse tests, attach finalizer, and attachable remote routing helper.
- Modify `tui/src/lib.rs`: add latest-loaded-thread lookup and startup selection behavior for attach mode.
- Test with focused package tests in `codex-cli` and `codex-tui`.

## Scope Boundaries

This v0 intentionally does not add a server-side input lock, desktop read-only mode, controller transfer, network listeners, or bearer-token auth. Same-host SSH safety comes from using the existing `unix://` app-server control socket.

`codex attach` should attach only to a currently loaded daemon thread. If the daemon is missing or no loaded interactive thread exists, it should exit with a clear message instead of silently starting a new conversation.

---

### Task 1: Add CLI State for Attachable and Attach Mode

**Files:**
- Modify: `tui/src/cli.rs`
- Test: `tui/src/cli.rs`

- [ ] **Step 1: Write failing TUI CLI tests**

Add these tests inside `#[cfg(test)] mod tests` in `tui/src/cli.rs`:

```rust
    #[test]
    fn attachable_flag_enables_attachable_mode() {
        let cli = Cli::try_parse_from(["codex", "--attachable"]).expect("valid cli");

        assert!(cli.attachable);
    }

    #[test]
    fn attach_internal_mode_is_not_user_parseable() {
        let cli = Cli::try_parse_from(["codex"]).expect("valid cli");

        assert!(!cli.attach_latest_loaded);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p codex-tui attachable_flag_enables_attachable_mode attach_internal_mode_is_not_user_parseable
```

Expected: fail because `Cli` does not have `attachable` or `attach_latest_loaded`.

- [ ] **Step 3: Add the CLI fields**

In `tui/src/cli.rs`, add `attach_latest_loaded` near the other internal startup controls:

```rust
    /// Internal: attach to the most recently updated loaded thread in the remote app server.
    #[clap(skip)]
    pub attach_latest_loaded: bool,
```

Add the public flag near the other user-visible TUI flags:

```rust
    /// Run this TUI through the local app-server daemon so another same-host terminal can attach.
    #[arg(long = "attachable", default_value_t = false)]
    pub attachable: bool,
```

- [ ] **Step 4: Run tests to verify they pass**

Run:

```bash
cargo test -p codex-tui attachable
```

Expected: the new `attachable_*` tests pass.

- [ ] **Step 5: Commit**

```bash
git add codex-rs/tui/src/cli.rs
git commit -m "feat(tui): add local attach startup flags"
```

---

### Task 2: Add `codex attach` and `--attachable` Routing in the CLI

**Files:**
- Modify: `cli/src/main.rs`
- Test: `cli/src/main.rs`

- [ ] **Step 1: Write failing parse and helper tests**

Add these tests in `cli/src/main.rs` near the existing remote/resume tests:

```rust
    #[test]
    fn attach_subcommand_parses_last() {
        let cli = MultitoolCli::try_parse_from(["codex", "attach", "--last"]).expect("parse");
        let Some(Subcommand::Attach(AttachCommand { last, .. })) = cli.subcommand else {
            panic!("expected attach subcommand");
        };

        assert!(last);
    }

    #[test]
    fn attach_finalizer_sets_latest_loaded_attach_mode() {
        let interactive = finalize_attach_interactive(
            TuiCli::try_parse_from(["codex"]).expect("base interactive"),
            CliConfigOverrides::default(),
            TuiCli::try_parse_from(["codex"]).expect("attach overrides"),
        );

        assert!(interactive.attach_latest_loaded);
        assert!(!interactive.resume_picker);
        assert!(!interactive.resume_last);
        assert_eq!(interactive.resume_session_id, None);
    }

    #[test]
    fn attachable_local_remote_arg_uses_default_unix_socket() {
        let resolved = resolve_attachable_remote_arg(
            /*attachable*/ true,
            /*remote*/ None,
            /*remote_auth_token_env*/ None,
        )
        .expect("attachable remote should resolve");

        assert_eq!(resolved.as_deref(), Some("unix://"));
    }

    #[test]
    fn attachable_rejects_explicit_remote() {
        let err = resolve_attachable_remote_arg(
            /*attachable*/ true,
            Some("unix:///tmp/codex.sock".to_string()),
            /*remote_auth_token_env*/ None,
        )
        .expect_err("attachable should reject explicit remote");

        assert!(
            err.contains("`--attachable` cannot be combined with `--remote`"),
            "{err}"
        );
    }

    #[test]
    fn attachable_rejects_remote_auth_token_env() {
        let err = resolve_attachable_remote_arg(
            /*attachable*/ true,
            /*remote*/ None,
            Some("CODEX_REMOTE_AUTH_TOKEN"),
        )
        .expect_err("attachable should reject remote auth token env");

        assert!(
            err.contains("`--attachable` cannot be combined with `--remote-auth-token-env`"),
            "{err}"
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p codex-cli attach
```

Expected: fail because `AttachCommand`, `Subcommand::Attach`, `finalize_attach_interactive`, and `resolve_attachable_remote_arg` do not exist.

- [ ] **Step 3: Add the attach command type and subcommand**

In `cli/src/main.rs`, add the subcommand beside `Resume` and `Fork`:

```rust
    /// Attach to the most recently updated live interactive session from the local app-server daemon.
    Attach(AttachCommand),
```

Add the command struct near `ResumeCommand`:

```rust
#[derive(Debug, Parser)]
struct AttachCommand {
    /// Attach to the most recently updated loaded interactive session.
    ///
    /// This flag is accepted for readability; v0 attach always selects the latest loaded session.
    #[arg(long = "last", default_value_t = false)]
    last: bool,

    #[clap(flatten)]
    config_overrides: TuiCli,
}
```

- [ ] **Step 4: Add finalizer and attachable remote helper**

In `cli/src/main.rs`, add these helpers near `finalize_resume_interactive`:

```rust
fn finalize_attach_interactive(
    mut interactive: TuiCli,
    root_config_overrides: CliConfigOverrides,
    attach_cli: TuiCli,
) -> TuiCli {
    interactive.attach_latest_loaded = true;
    interactive.resume_picker = false;
    interactive.resume_last = false;
    interactive.resume_session_id = None;
    interactive.resume_show_all = true;
    interactive.resume_include_non_interactive = false;

    merge_interactive_cli_flags(&mut interactive, attach_cli);
    prepend_config_flags(&mut interactive.config_overrides, root_config_overrides);
    interactive
}

fn resolve_attachable_remote_arg(
    attachable: bool,
    remote: Option<String>,
    remote_auth_token_env: Option<&str>,
) -> Result<Option<String>, String> {
    if !attachable {
        return Ok(remote);
    }
    if remote.is_some() {
        return Err("`--attachable` cannot be combined with `--remote`; it starts and connects to the local app-server daemon via `unix://`.".to_string());
    }
    if remote_auth_token_env.is_some() {
        return Err("`--attachable` cannot be combined with `--remote-auth-token-env`; local Unix sockets do not use bearer-token auth.".to_string());
    }
    Ok(Some("unix://".to_string()))
}
```

- [ ] **Step 5: Route the `attach` subcommand**

In the main `match` in `cli/src/main.rs`, add this arm near `Resume`:

```rust
        Some(Subcommand::Attach(AttachCommand {
            last: _,
            config_overrides,
        })) => {
            reject_remote_mode_for_subcommand(
                root_remote.as_deref(),
                root_remote_auth_token_env.as_deref(),
                "attach",
            )?;
            interactive = finalize_attach_interactive(
                interactive,
                root_config_overrides.clone(),
                config_overrides,
            );
            let exit_info = run_interactive_tui(
                interactive,
                Some("unix://".to_string()),
                None,
                arg0_paths.clone(),
            )
            .await?;
            print_exit_info(exit_info);
        }
```

- [ ] **Step 6: Make `--attachable` start and use the local daemon**

In `run_interactive_tui`, before resolving `remote_endpoint`, add:

```rust
    if interactive.attachable {
        codex_app_server_daemon::ensure_remote_control_started()
            .await
            .map_err(std::io::Error::other)?;
    }

    let remote = resolve_attachable_remote_arg(
        interactive.attachable,
        remote,
        remote_auth_token_env.as_deref(),
    )
    .map_err(std::io::Error::other)?;
```

Then let the existing `let mut remote_endpoint = remote.as_deref()...` block use this new `remote` binding. Keep the existing auth-token handling unchanged after this helper call.

- [ ] **Step 7: Run tests to verify they pass**

Run:

```bash
cargo test -p codex-cli attach
```

Expected: all new attach and attachable helper tests pass.

- [ ] **Step 8: Commit**

```bash
git add codex-rs/cli/src/main.rs
git commit -m "feat(cli): add local attach command"
```

---

### Task 3: Attach to the Latest Loaded Interactive Thread

**Files:**
- Modify: `tui/src/lib.rs`
- Test: `tui/src/lib.rs`

- [ ] **Step 1: Write failing unit tests for loaded-thread selection**

Add these tests inside the existing `#[cfg(test)]` module in `tui/src/lib.rs`:

```rust
    #[test]
    fn attachable_thread_filter_accepts_cli_and_vscode_sources() {
        assert!(is_attachable_interactive_source(
            &AppServerSessionSource::Cli
        ));
        assert!(is_attachable_interactive_source(
            &AppServerSessionSource::VsCode
        ));
    }

    #[test]
    fn attachable_thread_filter_rejects_non_interactive_sources() {
        assert!(!is_attachable_interactive_source(
            &AppServerSessionSource::Exec
        ));
        assert!(!is_attachable_interactive_source(
            &AppServerSessionSource::AppServer
        ));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p codex-tui attachable_thread_filter
```

Expected: fail because `AppServerSessionSource` and `is_attachable_interactive_source` are not defined in `tui/src/lib.rs`.

- [ ] **Step 3: Add imports**

At the top of `tui/src/lib.rs`, extend app-server protocol imports:

```rust
use codex_app_server_protocol::SessionSource as AppServerSessionSource;
use codex_app_server_protocol::ThreadLoadedListParams;
```

- [ ] **Step 4: Add the source filter and lookup helper**

Add this helper near `lookup_latest_session_target_with_app_server`:

```rust
fn is_attachable_interactive_source(source: &AppServerSessionSource) -> bool {
    matches!(
        source,
        AppServerSessionSource::Cli | AppServerSessionSource::VsCode
    )
}

async fn lookup_latest_loaded_session_target_with_app_server(
    app_server: &mut AppServerSession,
) -> color_eyre::Result<Option<resume_picker::SessionTarget>> {
    let mut cursor = None;
    let mut best: Option<AppServerThread> = None;

    loop {
        let response = app_server
            .thread_loaded_list(ThreadLoadedListParams {
                cursor: cursor.clone(),
                limit: Some(100),
            })
            .await?;

        for id in response.data {
            let Ok(thread_id) = ThreadId::from_string(&id) else {
                warn!(thread_id = id, "Ignoring loaded thread with invalid thread id");
                continue;
            };
            let thread = match app_server.thread_read(thread_id, /*include_turns*/ false).await {
                Ok(thread) => thread,
                Err(err) => {
                    warn!(%err, %thread_id, "Failed to read loaded thread during attach lookup");
                    continue;
                }
            };
            if thread.ephemeral || !is_attachable_interactive_source(&thread.source) {
                continue;
            }
            let replace_best = best
                .as_ref()
                .is_none_or(|current| thread.updated_at > current.updated_at);
            if replace_best {
                best = Some(thread);
            }
        }

        let Some(next_cursor) = response.next_cursor else {
            break;
        };
        cursor = Some(next_cursor);
    }

    Ok(best.and_then(session_target_from_app_server_thread))
}
```

- [ ] **Step 5: Route startup attach mode**

In `tui/src/lib.rs`, in the startup `session_selection` chain before `let use_fork = ...`, add:

```rust
    let use_attach = cli.attach_latest_loaded;
```

Then change the `session_selection` expression so attach mode is checked first:

```rust
    let session_selection = if use_attach {
        let Some(startup_app_server) = app_server.as_mut() else {
            unreachable!("app server should be initialized for attach");
        };
        match lookup_latest_loaded_session_target_with_app_server(startup_app_server).await? {
            Some(target_session) => resume_picker::SessionSelection::Resume(target_session),
            None => {
                terminal_restore_guard.restore_silently();
                session_log::log_session_end();
                let _ = tui.terminal.clear();
                return Ok(AppExitInfo {
                    token_usage: crate::token_usage::TokenUsage::default(),
                    thread_id: None,
                    thread_name: None,
                    update_action: None,
                    exit_reason: ExitReason::Fatal(
                        "No attachable Codex session is currently loaded. Start the desktop session with `codex --attachable`, then run `codex attach` from the SSH terminal.".to_string(),
                    ),
                });
            }
        }
    } else if use_fork {
```

Keep the rest of the existing fork/resume picker logic unchanged.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test -p codex-tui attachable_thread_filter
```

Expected: pass.

- [ ] **Step 7: Run existing startup/resume tests that guard nearby behavior**

Run:

```bash
cargo test -p codex-tui startup resume_picker_logic latest_session_lookup
```

Expected: pass. If the filter syntax does not match test names exactly, run:

```bash
cargo test -p codex-tui startup
cargo test -p codex-tui latest_session_lookup
```

- [ ] **Step 8: Commit**

```bash
git add codex-rs/tui/src/lib.rs
git commit -m "feat(tui): attach to latest loaded local thread"
```

---

### Task 4: Manual Same-Host SSH Verification

**Files:**
- No source files required.
- Use two local terminals, or one desktop terminal and one SSH terminal on the same host.

- [ ] **Step 1: Build the CLI**

Run:

```bash
cargo build -p codex-cli
```

Expected: build completes successfully.

- [ ] **Step 2: Start an attachable desktop session**

Terminal A:

```bash
cargo run -p codex-cli -- --attachable
```

Expected: Codex starts normally. It should be backed by the local app-server daemon because `--attachable` started the daemon and connected over `unix://`.

- [ ] **Step 3: Attach from a second same-host terminal**

Terminal B:

```bash
cargo run -p codex-cli -- attach --last
```

Expected: Terminal B opens the same latest loaded interactive thread from Terminal A. It should replay the current transcript and continue receiving live events from the app-server listener.

- [ ] **Step 4: Verify no-session failure is clear**

Stop the daemon:

```bash
cargo run -p codex-cli -- remote-control stop
```

Then run:

```bash
cargo run -p codex-cli -- attach --last
```

Expected: the command fails clearly because the local daemon socket is unavailable, or it reaches the attach startup path and reports:

```text
No attachable Codex session is currently loaded. Start the desktop session with `codex --attachable`, then run `codex attach` from the SSH terminal.
```

- [ ] **Step 5: Commit verification message polish**

When manual verification exposes message polish or command-shape fixes, commit them:

```bash
git add codex-rs/cli/src/main.rs codex-rs/tui/src/lib.rs codex-rs/tui/src/cli.rs
git commit -m "fix(cli): polish local attach startup errors"
```

---

### Task 5: Final Verification

**Files:**
- No source files required.

- [ ] **Step 1: Run focused package tests**

Run:

```bash
cargo test -p codex-cli attach
cargo test -p codex-tui attachable
cargo test -p codex-tui latest_session_lookup
```

Expected: all focused tests pass.

- [ ] **Step 2: Run formatting**

Run:

```bash
cargo fmt --check
```

Expected: no formatting diffs.

- [ ] **Step 3: Run workspace-adjacent check**

Run:

```bash
cargo check -p codex-cli -p codex-tui
```

Expected: both packages check successfully.

---

## Self-Review

- Spec coverage: the plan covers `codex --attachable`, `codex attach --last`, same-host Unix socket transport, latest loaded thread selection, and a clear missing-session path.
- Scope: controller ownership, read-only desktop mode, remote TCP, and auth tokens are explicitly excluded from v0.
- Type consistency: the plan uses existing app-server protocol names: `ThreadLoadedListParams`, `Thread`, `SessionSource::{Cli, VsCode}`, `thread_loaded_list`, `thread_read`, and `ThreadId::from_string`.
- Risk: the manual verification step is required because this feature depends on cross-client daemon behavior that unit tests only partially cover.
