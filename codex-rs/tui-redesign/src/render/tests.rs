use insta::assert_snapshot;
use pretty_assertions::assert_eq;

use super::*;
use crate::CommandChoice;
use crate::DemoMode;
use crate::FocusTarget;
use crate::Overlay;

#[test]
fn demo_mode_controls_blocking_surfaces() {
    let idle = RedesignState::demo(DemoMode::Idle);
    let approval = RedesignState::demo(DemoMode::Approval);

    assert_eq!(idle.approval, None);
    assert_eq!(idle.work, None);
    assert!(approval.approval.is_some());
    assert!(approval.work.is_some());
    assert_eq!(approval.focus, FocusTarget::Approval);
}

#[test]
fn composer_submit_records_local_turn() {
    let mut state = RedesignState::demo(DemoMode::Idle);
    state.composer.draft = "Make approval choices keyboard-first".to_string();

    state.submit_composer();

    assert_eq!(state.composer.draft, "");
    assert_eq!(state.transcript.len(), 4);
    assert!(state.work.is_some());
    assert_eq!(state.focus, FocusTarget::Composer);
}

#[test]
fn selected_approval_action_clears_request() {
    let mut state = RedesignState::demo(DemoMode::Approval);

    state.select_next_approval_action();
    state.apply_selected_approval_action();

    assert_eq!(state.approval, None);
    assert_eq!(state.focus, FocusTarget::Composer);
    assert_eq!(state.transcript.len(), 3);
}

#[test]
fn command_palette_can_simulate_approval() {
    let mut state = RedesignState::demo(DemoMode::Idle);
    state.command_choice = CommandChoice::SimulateApproval;

    state.apply_selected_command();

    assert!(state.approval.is_some());
    assert_eq!(state.focus, FocusTarget::Approval);
    assert_eq!(state.overlay, Overlay::None);
}

#[test]
fn idle_snapshot() {
    let state = RedesignState::demo(DemoMode::Idle);
    assert_snapshot!(
        "idle_100x24",
        render_to_string(/*width*/ 100, /*height*/ 24, &state).expect("render")
    );
}

#[test]
fn running_snapshot() {
    let state = RedesignState::demo(DemoMode::Running);
    assert_snapshot!(
        "running_100x24",
        render_to_string(/*width*/ 100, /*height*/ 24, &state).expect("render")
    );
}

#[test]
fn approval_snapshot() {
    let state = RedesignState::demo(DemoMode::Approval);
    assert_snapshot!(
        "approval_120x32",
        render_to_string(/*width*/ 120, /*height*/ 32, &state).expect("render")
    );
}

#[test]
fn narrow_snapshot() {
    let state = RedesignState::demo(DemoMode::Approval);
    assert_snapshot!(
        "approval_72x24",
        render_to_string(/*width*/ 72, /*height*/ 24, &state).expect("render")
    );
}

#[test]
fn help_overlay_snapshot() {
    let mut state = RedesignState::demo(DemoMode::Approval);
    state.overlay = Overlay::Help;
    assert_snapshot!(
        "help_overlay_100x28",
        render_to_string(/*width*/ 100, /*height*/ 28, &state).expect("render")
    );
}

#[test]
fn commands_overlay_snapshot() {
    let mut state = RedesignState::demo(DemoMode::Approval);
    state.overlay = Overlay::Commands;
    assert_snapshot!(
        "commands_overlay_100x28",
        render_to_string(/*width*/ 100, /*height*/ 28, &state).expect("render")
    );
}

#[test]
fn commands_filtered_overlay_snapshot() {
    let mut state = RedesignState::demo(DemoMode::Approval);
    state.overlay = Overlay::Commands;
    state.command_query = "approval".to_string();
    state.command_choice = CommandChoice::SimulateApproval;
    assert_snapshot!(
        "commands_filtered_overlay_100x28",
        render_to_string(/*width*/ 100, /*height*/ 28, &state).expect("render")
    );
}
