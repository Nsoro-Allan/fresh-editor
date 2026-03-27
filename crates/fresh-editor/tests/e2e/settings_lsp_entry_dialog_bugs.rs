//! E2E tests reproducing settings UI bugs found during the TUI UX audit (Track Two).
//!
//! Bug A: ObjectArray `[+] Add new` is unreachable via keyboard in entry dialogs.
//!        Specifically, in the LSP Edit Value dialog, Down arrow skips from the
//!        ObjectArray entries directly to buttons, making it impossible to add a
//!        second LSP server for a language.
//!
//! Bug B: Down navigation in the Edit Item dialog is inconsistent — some items are
//!        visited twice (text fields auto-enter edit mode on first Down) and the
//!        navigation cycle doesn't cover all fields reliably.

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

/// Helper: open settings, search for "lsp", jump to the LSP section,
/// then navigate Down to the "python" entry and press Enter to open Edit Value.
fn open_python_lsp_edit_value(harness: &mut EditorTestHarness) {
    harness.open_settings().unwrap();

    // Search for "lsp" to jump directly to the LSP map
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    harness.type_text("lsp").unwrap();
    harness.render().unwrap();

    // Jump to LSP section
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Lsp");

    // Navigate down through LSP entries until we find "python"
    for _ in 0..50 {
        harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
        harness.render().unwrap();

        let screen = harness.screen_to_string();
        if screen.contains("python") && screen.contains("[Enter to edit]") {
            // Check that the [Enter to edit] hint is on the python line
            for line in screen.lines() {
                if line.contains("python") && line.contains("[Enter to edit]") {
                    // Found and focused — press Enter to open Edit Value
                    harness
                        .send_key(KeyCode::Enter, KeyModifiers::NONE)
                        .unwrap();
                    harness.render().unwrap();
                    harness.assert_screen_contains("Edit Value");
                    harness.assert_screen_contains("Key:python");
                    return;
                }
            }
        }
    }

    panic!(
        "Could not navigate to python LSP entry. Screen:\n{}",
        harness.screen_to_string()
    );
}

// ---------------------------------------------------------------------------
// Bug A: ObjectArray [+] Add new unreachable via keyboard
// ---------------------------------------------------------------------------

/// Reproduce Bug A: In the LSP Edit Value dialog for python, the `[+] Add new`
/// row inside the ObjectArray should be reachable via Down arrow navigation.
///
/// The ObjectArray shows existing entries (e.g., `pylsp`) and a `[+] Add new` row.
/// Down arrow should cycle: existing entries → [+] Add new → buttons.
/// Currently, Down skips [+] Add new and goes directly to buttons.
#[test]
fn test_lsp_edit_value_add_new_reachable_via_keyboard() {
    let mut harness = EditorTestHarness::new(120, 50).unwrap();
    harness.render().unwrap();

    open_python_lsp_edit_value(&mut harness);

    // We're now in the Edit Value dialog for python.
    // It should show: Key:python, Value: (ObjectArray with pylsp), [+] Add new, buttons
    harness.assert_screen_contains("pylsp");
    harness.assert_screen_contains("[+] Add new");

    // Navigate Down through the ObjectArray entries.
    // We should be able to reach [+] Add new before hitting the buttons.
    let mut found_add_new_focused = false;

    // Press Down up to 10 times. At some point, [+] Add new should get a focus
    // indicator or [Enter to add] hint, before we reach the buttons.
    for i in 0..10 {
        harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
        harness.render().unwrap();

        let screen = harness.screen_to_string();

        // Check if [+] Add new is focused (has ">" indicator on its line or [Enter to add] hint)
        for line in screen.lines() {
            if line.contains("[+] Add new")
                && (line.contains(">") || line.contains("[Enter to add]"))
            {
                found_add_new_focused = true;
                break;
            }
        }

        if found_add_new_focused {
            eprintln!("[+] Add new became focused after {} Down presses", i + 1);
            break;
        }

        // If we've already reached the buttons, the bug is reproduced
        for line in screen.lines() {
            if line.contains("> [ Save ]") || line.contains("> [ Cancel ]") {
                panic!(
                    "BUG A REPRODUCED: Down arrow reached buttons without ever focusing \
                     '[+] Add new'. After {} Down presses, focus jumped to buttons.\n\
                     This means adding a new LSP server via keyboard is impossible.\n\
                     Screen:\n{}",
                    i + 1,
                    screen
                );
            }
        }
    }

    assert!(
        found_add_new_focused,
        "Expected '[+] Add new' to become focused via Down arrow navigation, \
         but it was never reached in 10 Down presses.\nScreen:\n{}",
        harness.screen_to_string()
    );

    // Verify that pressing Enter on [+] Add new opens the Add Item dialog
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Add Item");

    // Clean up
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Reproduce Bug A (variant): the full workflow of adding a second LSP server for
/// python should be completable entirely via keyboard.
#[test]
fn test_add_second_lsp_server_for_python_via_keyboard() {
    let mut harness = EditorTestHarness::new(120, 50).unwrap();
    harness.render().unwrap();

    open_python_lsp_edit_value(&mut harness);

    // Verify pylsp is the existing server
    harness.assert_screen_contains("pylsp");
    harness.assert_screen_contains("[+] Add new");

    // Navigate to [+] Add new via Down arrows
    let mut reached_add_new = false;
    for _ in 0..10 {
        harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
        harness.render().unwrap();

        let screen = harness.screen_to_string();
        for line in screen.lines() {
            if line.contains("[+] Add new")
                && (line.contains(">") || line.contains("[Enter to add]"))
            {
                reached_add_new = true;
                break;
            }
        }
        if reached_add_new {
            break;
        }
    }

    assert!(
        reached_add_new,
        "Could not reach '[+] Add new' via keyboard. Screen:\n{}",
        harness.screen_to_string()
    );

    // Press Enter to open Add Item dialog for the new server
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Add Item");

    // The Add Item dialog should have a Command field. Fill it in.
    // Command should be the first (or near-first) field.
    harness.assert_screen_contains("Command");

    // Navigate to Command if not already focused
    let screen = harness.screen_to_string();
    if !screen
        .lines()
        .any(|l| l.contains(">") && l.contains("Command"))
    {
        // Down through fields until we find Command
        for _ in 0..5 {
            harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
            harness.render().unwrap();
            let s = harness.screen_to_string();
            if s.lines().any(|l| l.contains(">") && l.contains("Command")) {
                break;
            }
        }
    }

    // Type "pyright-langserver" into the Command field
    harness.type_text("pyright-langserver").unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("pyright-langserver");

    // Save with Ctrl+S
    harness
        .send_key(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Should be back in Edit Value dialog with both servers listed
    harness.assert_screen_contains("Edit Value");
    harness.assert_screen_contains("pylsp");
    harness.assert_screen_contains("pyright-langserver");

    // Clean up
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

// ---------------------------------------------------------------------------
// Bug B: Down navigation inconsistencies in Edit Item dialog
// ---------------------------------------------------------------------------

/// Reproduce Bug B: Each Down press should advance focus to a different field.
///
/// Text fields currently auto-enter edit mode when they receive focus via Down,
/// which means the first Down "enters" the field (auto-edit) and a second Down
/// is needed to leave it. This test counts the total Down presses needed to go
/// from the first field to the buttons, and asserts it equals the number of
/// distinct fields (one Down per field).
#[test]
fn test_entry_dialog_down_visits_every_field_once() {
    let mut harness = EditorTestHarness::new(120, 50).unwrap();
    harness.render().unwrap();

    open_python_lsp_edit_value(&mut harness);

    // Open the Edit Item dialog for the pylsp server.
    // Enter on the Value/ObjectArray label should open the nested Edit Item.
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let screen = harness.screen_to_string();
    if !screen.contains("Edit Item") {
        // The Enter might have triggered something else; try again from scratch
        harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
        harness.render().unwrap();
        open_python_lsp_edit_value(&mut harness);
        harness
            .send_key(KeyCode::Enter, KeyModifiers::NONE)
            .unwrap();
        harness.render().unwrap();
    }

    harness.assert_screen_contains("Command");
    harness.assert_screen_contains("Enabled");

    // Known fields in the Edit Item dialog (ordered by importance, post-rebase).
    let known_fields = [
        "Command",
        "Enabled",
        "Name",
        "Args",
        "Auto Start",
        "Root Markers",
        "Env",
        "Language Id Overrides",
        "Initialization Options",
        "Only Features",
        "Except Features",
        "Process Limits",
    ];

    // Helper: identify which known field (if any) has the focus indicator ">"
    let identify_focused = |screen: &str| -> Option<String> {
        for line in screen.lines() {
            if !line.contains(">") {
                continue;
            }
            if line.contains("[ Save ]") || line.contains("[ Cancel ]") {
                return Some("__BUTTONS__".to_string());
            }
            for field in &known_fields {
                if line.contains(field) {
                    return Some(field.to_string());
                }
            }
        }
        None
    };

    // Record the focus after every single Down press.
    // This captures ALL positions, including duplicates from auto-edit.
    let mut focus_trace: Vec<String> = Vec::new();

    // Record the initial focus
    harness.render().unwrap();
    if let Some(f) = identify_focused(&harness.screen_to_string()) {
        focus_trace.push(f.clone());
    }

    // Press Down repeatedly until we hit buttons or exhaust attempts
    for _ in 0..40 {
        harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
        harness.render().unwrap();

        if let Some(f) = identify_focused(&harness.screen_to_string()) {
            focus_trace.push(f.clone());
            if f == "__BUTTONS__" {
                break;
            }
        }
    }

    // Count how many Down presses it took to reach buttons (excluding buttons entry)
    let field_presses: Vec<&String> = focus_trace.iter().filter(|f| *f != "__BUTTONS__").collect();

    // Deduplicate consecutive fields to get the distinct field visit order
    let mut distinct_fields: Vec<&String> = Vec::new();
    for f in &field_presses {
        if distinct_fields.last() != Some(f) {
            distinct_fields.push(f);
        }
    }

    // Assert: every known field was visited
    let mut missing: Vec<&str> = Vec::new();
    for field in &known_fields {
        if !distinct_fields.iter().any(|f| f.as_str() == *field) {
            missing.push(field);
        }
    }
    assert!(
        missing.is_empty(),
        "Fields never visited during Down navigation: {:?}\n\
         Distinct fields visited: {:?}\n\
         Full trace: {:?}",
        missing,
        distinct_fields,
        focus_trace
    );

    // Assert: the number of Down presses to traverse all fields should equal
    // the number of distinct fields. If text fields require extra Downs
    // (auto-edit consumes the first Down), total presses will exceed field count.
    let total_presses = field_presses.len();
    let distinct_count = distinct_fields.len();

    assert_eq!(
        total_presses, distinct_count,
        "BUG B REPRODUCED: {} Down presses were needed to traverse {} distinct fields.\n\
         Each field should require exactly 1 Down press, but some consumed extra presses.\n\
         Distinct fields: {:?}\n\
         Full trace (showing duplicates): {:?}",
        total_presses, distinct_count, distinct_fields, focus_trace
    );

    // Clean up
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}
