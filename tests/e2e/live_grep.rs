use crate::common::fixtures::TestFixture;
use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use std::fs;

/// Test Live Grep plugin - basic search and preview functionality
#[test]
fn test_live_grep_basic_search() {
    // Create a temporary project directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    fs::create_dir(&project_root).unwrap();

    // Create plugins directory and copy the live_grep plugin
    let plugins_dir = project_root.join("plugins");
    fs::create_dir(&plugins_dir).unwrap();

    let plugin_source = std::env::current_dir()
        .unwrap()
        .join("plugins/live_grep.ts");
    let plugin_dest = plugins_dir.join("live_grep.ts");
    fs::copy(&plugin_source, &plugin_dest).unwrap();

    // Create test files with searchable content
    let file1_content = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let file2_content = "fn helper() {\n    println!(\"Helper function\");\n}\n";
    let file3_content = "// This file contains UNIQUE_MARKER for testing\nlet x = 42;\n";

    fs::write(project_root.join("main.rs"), file1_content).unwrap();
    fs::write(project_root.join("helper.rs"), file2_content).unwrap();
    fs::write(project_root.join("test.rs"), file3_content).unwrap();

    // Create a file to open initially
    let fixture = TestFixture::new("initial.txt", "Initial file content\n").unwrap();

    // Create harness with the project directory (so plugins load)
    let mut harness =
        EditorTestHarness::with_config_and_working_dir(100, 30, Default::default(), project_root)
            .unwrap();

    // Open the initial file
    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Open command palette and find Live Grep
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    harness.type_text("Live Grep").unwrap();

    // Wait for Live Grep to appear in palette (plugin loaded)
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("Live Grep") || s.contains("Find in Files")
        })
        .unwrap();

    // Execute the command
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Now we should be in the live grep prompt
    // Type a search query
    harness.type_text("UNIQUE_MARKER").unwrap();

    // Wait for search results to appear
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("test.rs") || s.contains("UNIQUE_MARKER")
        })
        .unwrap();

    // Press Escape to cancel
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Verify we're back to normal state
    let final_screen = harness.screen_to_string();
    assert!(
        final_screen.contains("Initial file content")
            || final_screen.contains("Live grep cancelled"),
        "Should return to normal state after ESC. Got:\n{}",
        final_screen
    );
}

/// Test Live Grep - selecting a result opens the file
#[test]
#[ignore = "flaky test - times out intermittently"]
fn test_live_grep_select_result() {
    // Create a temporary project directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    fs::create_dir(&project_root).unwrap();

    // Create plugins directory and copy the live_grep plugin
    let plugins_dir = project_root.join("plugins");
    fs::create_dir(&plugins_dir).unwrap();

    let plugin_source = std::env::current_dir()
        .unwrap()
        .join("plugins/live_grep.ts");
    let plugin_dest = plugins_dir.join("live_grep.ts");
    fs::copy(&plugin_source, &plugin_dest).unwrap();

    // Create a test file with unique content
    let target_content = "// TARGET_FILE\nfn target_function() {\n    let result = 123;\n}\n";
    fs::write(project_root.join("target.rs"), target_content).unwrap();

    // Create initial file
    let fixture = TestFixture::new("start.txt", "Starting point\n").unwrap();

    // Create harness
    let mut harness =
        EditorTestHarness::with_config_and_working_dir(100, 30, Default::default(), project_root)
            .unwrap();

    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Start Live Grep via command palette
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Live Grep").unwrap();

    // Wait for Live Grep command to appear (plugin loaded)
    harness
        .wait_until(|h| h.screen_to_string().contains("Live Grep"))
        .unwrap();

    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Search for the target
    harness.type_text("TARGET_FILE").unwrap();

    // Wait for results to appear
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("target.rs") || s.contains("TARGET_FILE")
        })
        .unwrap();

    // Press Enter to select the result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();

    // Wait for target file to open
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("TARGET_FILE") || s.contains("target_function")
        })
        .unwrap();
}

/// Test Live Grep - preview split appears and closes on ESC
#[test]
#[ignore = "flaky test - times out intermittently"]
fn test_live_grep_preview_split() {
    // Create a temporary project directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    fs::create_dir(&project_root).unwrap();

    // Create plugins directory and copy the live_grep plugin
    let plugins_dir = project_root.join("plugins");
    fs::create_dir(&plugins_dir).unwrap();

    let plugin_source = std::env::current_dir()
        .unwrap()
        .join("plugins/live_grep.ts");
    let plugin_dest = plugins_dir.join("live_grep.ts");
    fs::copy(&plugin_source, &plugin_dest).unwrap();

    // Create a test file with content to search
    let search_content = "PREVIEW_TEST_CONTENT\nLine 2\nLine 3\nLine 4\nLine 5\n";
    fs::write(project_root.join("preview_test.txt"), search_content).unwrap();

    // Create initial file
    let fixture = TestFixture::new("main.txt", "Main file\n").unwrap();

    // Create harness
    let mut harness =
        EditorTestHarness::with_config_and_working_dir(120, 30, Default::default(), project_root)
            .unwrap();

    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Start Live Grep via command palette
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Live Grep").unwrap();

    // Wait for Live Grep command to appear (plugin loaded)
    harness
        .wait_until(|h| h.screen_to_string().contains("Live Grep"))
        .unwrap();

    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Search for content
    harness.type_text("PREVIEW_TEST").unwrap();

    // Wait for preview split to appear
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("*Preview*") || s.contains("PREVIEW_TEST_CONTENT")
        })
        .unwrap();

    // Press ESC to cancel
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();

    // Wait for preview split to close
    harness
        .wait_until(|h| !h.screen_to_string().contains("*Preview*"))
        .unwrap();
}

/// Test Live Grep - input is preserved when navigating results
#[test]
fn test_live_grep_input_preserved() {
    // Create a temporary project directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project_root");
    fs::create_dir(&project_root).unwrap();

    // Create plugins directory and copy the live_grep plugin
    let plugins_dir = project_root.join("plugins");
    fs::create_dir(&plugins_dir).unwrap();

    let plugin_source = std::env::current_dir()
        .unwrap()
        .join("plugins/live_grep.ts");
    let plugin_dest = plugins_dir.join("live_grep.ts");
    fs::copy(&plugin_source, &plugin_dest).unwrap();

    // Create multiple files with matching content
    for i in 1..=5 {
        let content = format!("MULTI_MATCH line in file {}\n", i);
        fs::write(project_root.join(format!("file{}.txt", i)), content).unwrap();
    }

    // Create initial file
    let fixture = TestFixture::new("start.txt", "Start\n").unwrap();

    // Create harness
    let mut harness =
        EditorTestHarness::with_config_and_working_dir(100, 30, Default::default(), project_root)
            .unwrap();

    harness.open_file(&fixture.path).unwrap();
    harness.render().unwrap();

    // Start Live Grep via command palette
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.type_text("Live Grep").unwrap();

    // Wait for Live Grep command to appear (plugin loaded)
    harness
        .wait_until(|h| h.screen_to_string().contains("Live Grep"))
        .unwrap();

    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Type search query
    harness.type_text("MULTI_MATCH").unwrap();

    // Wait for results to appear
    harness
        .wait_until(|h| {
            let s = h.screen_to_string();
            s.contains("file1.txt") || s.contains("MULTI_MATCH")
        })
        .unwrap();

    // Navigate down through results
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    let screen_after_nav = harness.screen_to_string();
    println!("Screen after navigation:\n{}", screen_after_nav);

    // The prompt should still show "MULTI_MATCH" (input preserved)
    // This verifies our fix that plugin prompts don't overwrite input on navigation
    assert!(
        screen_after_nav.contains("MULTI_MATCH"),
        "Search input should be preserved when navigating results. Got:\n{}",
        screen_after_nav
    );

    // Clean up
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
}
