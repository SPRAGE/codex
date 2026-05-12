use std::error::Error;
use std::io::IsTerminal;

use codex_tui_redesign::DemoMode;
use codex_tui_redesign::RedesignState;
use codex_tui_redesign::render_to_string;
use codex_tui_redesign::run_terminal_preview;

fn main() -> Result<(), Box<dyn Error>> {
    let mut width = 120;
    let mut height = 36;
    let mut mode = DemoMode::Approval;
    let mut force_dump = false;
    let mut force_live = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dump" => force_dump = true,
            "--live" => force_live = true,
            "--mode" => {
                let Some(value) = args.next() else {
                    return Err("--mode requires idle, running, or approval".into());
                };
                mode = parse_mode(&value)?;
            }
            "--width" => {
                let Some(value) = args.next() else {
                    return Err("--width requires a number".into());
                };
                width = value.parse()?;
            }
            "--height" => {
                let Some(value) = args.next() else {
                    return Err("--height requires a number".into());
                };
                height = value.parse()?;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                mode = parse_mode(other)?;
            }
        }
    }

    if force_live || (!force_dump && std::io::stdout().is_terminal()) {
        run_terminal_preview(mode)?;
    } else {
        let state = RedesignState::demo(mode);
        print!("{}", render_to_string(width, height, &state)?);
    }
    Ok(())
}

fn parse_mode(value: &str) -> Result<DemoMode, Box<dyn Error>> {
    match value {
        "idle" => Ok(DemoMode::Idle),
        "running" => Ok(DemoMode::Running),
        "approval" => Ok(DemoMode::Approval),
        _ => Err(format!("unknown mode `{value}`; expected idle, running, or approval").into()),
    }
}

fn print_help() {
    println!("Preview the Codex TUI redesign prototype.");
    println!();
    println!("Usage:");
    println!("  cargo run -p codex-tui-redesign -- [idle|running|approval]");
    println!("  cargo run -p codex-tui-redesign -- --live --mode approval");
    println!("  cargo run -p codex-tui-redesign -- --dump --mode approval --width 120 --height 36");
    println!();
    println!("Live controls:");
    println!("  Tab / Shift+Tab moves focus");
    println!("  Alt-/ or Ctrl+P opens the command palette");
    println!("  Alt-H opens shortcuts");
    println!("  ? opens shortcuts when the composer is not focused");
    println!("  1 idle, 2 running, 3 approval");
    println!("  Esc, q, or Ctrl+C exits");
}
