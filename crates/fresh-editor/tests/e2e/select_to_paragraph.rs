//! E2E tests for Ctrl+Shift+Up/Down to select to empty line
//!
//! This feature extends selection to the next/previous empty (whitespace-only) line

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use fresh::config::Config;
use tempfile::TempDir;

/// Test that Ctrl+Shift+Up selects to previous empty line
#[test]
fn test_select_to_paragraph_up() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    // Create a file with paragraphs separated by empty lines
    let content =
        "paragraph 1 line 1\nparagraph 1 line 2\n\nparagraph 2 line 1\nparagraph 2 line 2\n";
    std::fs::write(&file_path, content).unwrap();

    let config = Config::default();
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&file_path).unwrap();

    // Move to line 4 (paragraph 2 line 1)
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Press Ctrl+Shift+Up to select to previous empty line
    harness
        .send_key(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    // Check that selection exists
    let selection = harness.get_selection_range();
    assert!(
        selection.is_some(),
        "Selection should exist after Ctrl+Shift+Up"
    );

    // The cursor should now be at the empty line (line 3, position after "paragraph 1 line 2\n")
    let pos = harness.cursor_position();
    println!("Cursor position after Ctrl+Shift+Up: {}", pos);

    // Selection should include from empty line to where we started
    let range = selection.unwrap();
    println!("Selection range: {:?}", range);

    // The selection should end at the empty line start
    assert!(
        range.start < range.end,
        "Selection should have positive range"
    );
}

/// Test that Ctrl+Shift+Down selects to next empty line
#[test]
fn test_select_to_paragraph_down() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    // Create a file with paragraphs separated by empty lines
    let content =
        "paragraph 1 line 1\nparagraph 1 line 2\n\nparagraph 2 line 1\nparagraph 2 line 2\n";
    std::fs::write(&file_path, content).unwrap();

    let config = Config::default();
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&file_path).unwrap();

    // Start at line 1
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Press Ctrl+Shift+Down to select to next empty line
    harness
        .send_key(KeyCode::Down, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    // Check that selection exists
    let selection = harness.get_selection_range();
    assert!(
        selection.is_some(),
        "Selection should exist after Ctrl+Shift+Down"
    );

    let range = selection.unwrap();
    println!("Selection range: {:?}", range);

    // Selection should have positive range (anchor at start, cursor moved forward)
    assert!(
        range.start < range.end,
        "Selection should have positive range"
    );
}

/// Test multiple Ctrl+Shift+Up presses extend selection further
#[test]
fn test_multiple_select_to_paragraph_up() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    // Create a file with multiple paragraphs
    let content = "para 1\n\npara 2\n\npara 3\npara 3 continued\n";
    std::fs::write(&file_path, content).unwrap();

    let config = Config::default();
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&file_path).unwrap();

    // Move to last line (para 3 continued)
    harness
        .send_key(KeyCode::End, KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // First Ctrl+Shift+Up - should select to empty line before para 3
    harness
        .send_key(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    let selection1 = harness.get_selection_range();
    assert!(selection1.is_some(), "First selection should exist");

    // Second Ctrl+Shift+Up - should extend selection to empty line before para 2
    harness
        .send_key(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    let selection2 = harness.get_selection_range();
    assert!(selection2.is_some(), "Second selection should exist");

    // Selection should be larger after second press
    let range1 = selection1.unwrap();
    let range2 = selection2.unwrap();
    assert!(
        range2.end - range2.start > range1.end - range1.start,
        "Selection should grow with multiple presses"
    );
}

/// Test Ctrl+Shift+Up at document start
#[test]
fn test_select_to_paragraph_up_at_start() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let content = "line 1\nline 2\n";
    std::fs::write(&file_path, content).unwrap();

    let config = Config::default();
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&file_path).unwrap();

    // Move to line 2
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Press Ctrl+Shift+Up - should select to start of document (no empty line found)
    harness
        .send_key(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    // Should have selection to start of document
    let selection = harness.get_selection_range();
    assert!(selection.is_some(), "Selection should exist");

    let range = selection.unwrap();
    assert_eq!(range.start, 0, "Selection should extend to document start");
}

/// Test Ctrl+Shift+Down at document end
#[test]
fn test_select_to_paragraph_down_at_end() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let content = "line 1\nline 2\n";
    std::fs::write(&file_path, content).unwrap();

    let config = Config::default();
    let mut harness = EditorTestHarness::with_config(80, 24, config).unwrap();
    harness.open_file(&file_path).unwrap();

    // Start at beginning
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Press Ctrl+Shift+Down - should select to end of document (no empty line found)
    harness
        .send_key(KeyCode::Down, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        .unwrap();
    harness.render().unwrap();

    // Should have selection to end of document
    let selection = harness.get_selection_range();
    assert!(selection.is_some(), "Selection should exist");

    let range = selection.unwrap();
    let content_len = content.len();
    assert_eq!(
        range.end, content_len,
        "Selection should extend to document end"
    );
}
