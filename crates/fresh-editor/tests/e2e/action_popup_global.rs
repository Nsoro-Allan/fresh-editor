//! E2E tests for plugin action popups (`editor.showActionPopup`).
//!
//! Action popups carry buffer-independent decisions (e.g. the
//! devcontainer plugin's "attach now?" prompt). They must remain visible
//! and actionable while the user is on _any_ buffer — including virtual
//! buffers like the Dashboard that own the whole split.
//!
//! Regression: the popup used to be attached to the active buffer's popup
//! stack at the moment showActionPopup ran, and would vanish as soon as a
//! plugin (e.g. the dashboard) made a different buffer active.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::services::plugins::api::{ActionPopupAction, PluginCommand};

fn show_devcontainer_attach_popup(harness: &mut EditorTestHarness) {
    harness
        .editor_mut()
        .handle_plugin_command(PluginCommand::ShowActionPopup {
            popup_id: "devcontainer-attach".to_string(),
            title: "Dev Container detected".to_string(),
            message: "Attach to dev container 'test-container'?".to_string(),
            actions: vec![
                ActionPopupAction {
                    id: "attach".to_string(),
                    label: "Attach".to_string(),
                },
                ActionPopupAction {
                    id: "dismiss".to_string(),
                    label: "Not now".to_string(),
                },
            ],
        })
        .unwrap();
}

/// The popup should render over a virtual buffer that owns the whole
/// split (Dashboard pattern), not just over file buffers. This is the
/// regression: previously the popup was scoped to the buffer that was
/// active at show-time, so a buffer switch hid it.
#[test]
fn action_popup_renders_over_virtual_buffer() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Create a virtual buffer mimicking the Dashboard plugin: a tab that
    // a plugin opens to fill the whole split before the popup appears.
    let dashboard_buffer = harness.editor_mut().create_virtual_buffer(
        "Dashboard".to_string(),
        "dashboard".to_string(),
        true,
    );
    harness
        .editor_mut()
        .set_virtual_buffer_content(
            dashboard_buffer,
            vec![fresh::primitives::text_property::TextPropertyEntry::text(
                "── Dashboard ──\n  weather: sunny\n  git: clean\n",
            )],
        )
        .unwrap();
    harness.editor_mut().switch_buffer(dashboard_buffer);
    harness.render().unwrap();

    // The dashboard text should be on screen before the popup is shown,
    // confirming the virtual buffer is the active split's content.
    let before = harness.screen_to_string();
    assert!(
        before.contains("Dashboard"),
        "Pre-popup screen should show the dashboard buffer. Screen:\n{}",
        before
    );

    // Now a plugin (e.g. devcontainer) shows its action popup. The
    // dashboard buffer is still active.
    show_devcontainer_attach_popup(&mut harness);
    harness.render().unwrap();

    // The popup body must be visible on screen even though the active
    // buffer is the virtual dashboard.
    let after = harness.screen_to_string();
    assert!(
        after.contains("Attach"),
        "Action popup should render over the dashboard. Screen:\n{}",
        after
    );
    assert!(
        after.contains("Not now"),
        "Action popup's dismiss action should be visible. Screen:\n{}",
        after
    );
    assert!(
        after.contains("Dev Container detected") || after.contains("Dev Container"),
        "Action popup title should be visible. Screen:\n{}",
        after
    );
}

/// Esc on a global action popup must dismiss it without falling through
/// to the buffer below.
#[test]
fn action_popup_dismisses_on_escape() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let dashboard_buffer = harness.editor_mut().create_virtual_buffer(
        "Dashboard".to_string(),
        "dashboard".to_string(),
        true,
    );
    harness
        .editor_mut()
        .set_virtual_buffer_content(
            dashboard_buffer,
            vec![fresh::primitives::text_property::TextPropertyEntry::text(
                "── Dashboard ──\n",
            )],
        )
        .unwrap();
    harness.editor_mut().switch_buffer(dashboard_buffer);
    harness.render().unwrap();

    show_devcontainer_attach_popup(&mut harness);
    harness.render().unwrap();
    assert!(
        harness.screen_to_string().contains("Attach"),
        "Sanity: popup is up before Esc."
    );

    // Esc should route to the global popup, not the buffer.
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();

    let after_esc = harness.screen_to_string();
    assert!(
        !after_esc.contains("Attach") && !after_esc.contains("Not now"),
        "Esc should dismiss the global action popup. Screen:\n{}",
        after_esc
    );
}

/// Switching to a different buffer after the popup is shown must NOT
/// hide it — the popup is editor-level, not buffer-local.
#[test]
fn action_popup_persists_across_buffer_switch() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Start on a file-style buffer.
    let scratch = harness.editor_mut().create_virtual_buffer(
        "scratch".to_string(),
        "text".to_string(),
        false,
    );
    harness
        .editor_mut()
        .set_virtual_buffer_content(
            scratch,
            vec![fresh::primitives::text_property::TextPropertyEntry::text(
                "scratch buffer\n",
            )],
        )
        .unwrap();
    harness.editor_mut().switch_buffer(scratch);
    harness.render().unwrap();

    // Show the popup while `scratch` is active.
    show_devcontainer_attach_popup(&mut harness);
    harness.render().unwrap();
    assert!(
        harness.screen_to_string().contains("Attach"),
        "Sanity: popup visible on the scratch buffer."
    );

    // Open a Dashboard-style virtual buffer and switch to it. With the old
    // buffer-scoped popup the popup would be lost here.
    let dashboard = harness.editor_mut().create_virtual_buffer(
        "Dashboard".to_string(),
        "dashboard".to_string(),
        true,
    );
    harness
        .editor_mut()
        .set_virtual_buffer_content(
            dashboard,
            vec![fresh::primitives::text_property::TextPropertyEntry::text(
                "── Dashboard ──\n",
            )],
        )
        .unwrap();
    harness.editor_mut().switch_buffer(dashboard);
    harness.render().unwrap();

    let after_switch = harness.screen_to_string();
    assert!(
        after_switch.contains("Attach"),
        "Action popup must survive a buffer switch. Screen:\n{}",
        after_switch
    );
}

fn show_generic_popup(
    harness: &mut EditorTestHarness,
    popup_id: &str,
    title: &str,
    body: &str,
    actions: Vec<(&str, &str)>,
) {
    harness
        .editor_mut()
        .handle_plugin_command(PluginCommand::ShowActionPopup {
            popup_id: popup_id.to_string(),
            title: title.to_string(),
            message: body.to_string(),
            actions: actions
                .into_iter()
                .map(|(id, label)| ActionPopupAction {
                    id: id.to_string(),
                    label: label.to_string(),
                })
                .collect(),
        })
        .unwrap();
}

/// Two popups pushed concurrently (e.g. two plugins both deciding the
/// session needs attention) must queue LIFO: only the top is interactive,
/// dismissing it surfaces the next, and each one fires its own
/// `action_popup_result` hook instead of the second clobbering the first's
/// tracking.
#[test]
fn action_popups_queue_and_each_resolves_independently() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // First plugin pops up an attach prompt.
    show_generic_popup(
        &mut harness,
        "devcontainer-attach",
        "Attach?",
        "Attach to 'alpha'?",
        vec![("attach", "Attach alpha"), ("dismiss", "Not now alpha")],
    );
    // Second plugin pops up right after — e.g. a different plugin's warning.
    show_generic_popup(
        &mut harness,
        "pkg-install",
        "Install?",
        "Install bravo package?",
        vec![("yes", "Install bravo"), ("no", "Skip bravo")],
    );
    harness.render().unwrap();

    // Only the second (top-of-stack) popup is visible. The first is queued
    // underneath and re-surfaces when the top is dismissed.
    let frame1 = harness.screen_to_string();
    assert!(
        frame1.contains("Install bravo"),
        "Top-of-stack popup must render. Screen:\n{}",
        frame1
    );
    assert!(
        !frame1.contains("Attach alpha"),
        "Queued popup must not leak through underneath. Screen:\n{}",
        frame1
    );

    // Esc dismisses the top (fires action_popup_result for 'pkg-install').
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Now the first popup (`devcontainer-attach`) takes its place.
    let frame2 = harness.screen_to_string();
    assert!(
        frame2.contains("Attach alpha"),
        "Queued popup must surface after the top is dismissed. Screen:\n{}",
        frame2
    );
    assert!(
        !frame2.contains("Install bravo"),
        "Dismissed popup must not be re-drawn. Screen:\n{}",
        frame2
    );

    // Dismiss the remaining popup too.
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    let frame3 = harness.screen_to_string();
    assert!(
        !frame3.contains("Attach alpha") && !frame3.contains("Install bravo"),
        "Both popups must be gone after two Esc presses. Screen:\n{}",
        frame3
    );
}

/// Same as above, but each popup is confirmed via Enter rather than Esc,
/// exercising the confirm path's parallel-stack pop.
#[test]
fn action_popups_queue_confirms_preserve_per_popup_identity() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    show_generic_popup(
        &mut harness,
        "first",
        "First?",
        "First popup body",
        vec![("ok", "OK first"), ("cancel", "Cancel first")],
    );
    show_generic_popup(
        &mut harness,
        "second",
        "Second?",
        "Second popup body",
        vec![("ok", "OK second"), ("cancel", "Cancel second")],
    );
    harness.render().unwrap();

    // Enter on the top popup ('second') — default selection is index 0 'OK'.
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let after_first_confirm = harness.screen_to_string();
    assert!(
        !after_first_confirm.contains("Second popup body"),
        "Top popup must be gone after Enter. Screen:\n{}",
        after_first_confirm
    );
    assert!(
        after_first_confirm.contains("First popup body"),
        "Queued popup must surface after top is confirmed. Screen:\n{}",
        after_first_confirm
    );

    // Enter again — this time on the first popup.
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    let after_second_confirm = harness.screen_to_string();
    assert!(
        !after_second_confirm.contains("First popup body"),
        "Both popups must be resolved. Screen:\n{}",
        after_second_confirm
    );
}
