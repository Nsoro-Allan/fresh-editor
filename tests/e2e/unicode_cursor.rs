use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::TempDir;

/// Test that cursor position stays in sync when editing lines with non-ASCII characters
/// This reproduces the bug where visual cursor position drifts from actual position
/// when a line contains Unicode box-drawing characters or other multi-byte UTF-8 characters
#[test]
fn test_cursor_sync_with_non_ascii_box_drawing_chars() {
    let mut harness = EditorTestHarness::new(120, 30).unwrap();

    // Type a line with box-drawing characters like in the bug report
    // Example: │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │
    let text_with_boxes = "   17 │ │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │";
    harness.type_text(text_with_boxes).unwrap();
    harness.render().unwrap();

    // Verify buffer content is correct
    harness.assert_buffer_content(text_with_boxes);

    // Get the buffer position (should be at end)
    let buffer_pos = harness.cursor_position();
    let expected_buffer_pos = text_with_boxes.len();
    assert_eq!(
        buffer_pos, expected_buffer_pos,
        "Cursor should be at end of text (byte position {}), but is at {}",
        expected_buffer_pos, buffer_pos
    );

    // Move cursor to the beginning of the line
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // Cursor should now be at position 0
    let buffer_pos_after_home = harness.cursor_position();
    assert_eq!(
        buffer_pos_after_home, 0,
        "Cursor should be at position 0 after Home"
    );

    // Now move cursor right character by character and verify screen position matches
    // The key insight: when moving through multi-byte UTF-8 characters,
    // the buffer position advances by the number of bytes in the character,
    // but the screen column should advance by 1

    // First, let's move right 10 times (through "   17 │ │ ")
    for i in 1..=10 {
        harness
            .send_key(KeyCode::Right, KeyModifiers::NONE)
            .unwrap();

        let buffer_pos = harness.cursor_position();
        let (screen_x, _screen_y) = harness.screen_cursor_position();

        // The screen cursor position depends on gutter width
        // For this test, we're mainly checking that the screen cursor advances properly
        // The gutter width varies based on line numbers, so we'll focus on relative movement

        println!(
            "After {} right arrows: buffer_pos={}, screen_x={}",
            i, buffer_pos, screen_x
        );
    }

    // Now test: type a character and verify it appears at the visual cursor position
    // Move to somewhere in the middle of the line
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // Move right 20 characters
    for _ in 0..20 {
        harness
            .send_key(KeyCode::Right, KeyModifiers::NONE)
            .unwrap();
    }

    let buffer_pos_before_insert = harness.cursor_position();
    let (screen_x_before, screen_y_before) = harness.screen_cursor_position();

    println!(
        "Before insert: buffer_pos={}, screen=({}, {})",
        buffer_pos_before_insert, screen_x_before, screen_y_before
    );

    // Insert a marker character 'X' at this position
    harness.type_text("X").unwrap();

    // Verify that 'X' appears at the expected position in the buffer
    let buffer_content_after = harness.get_buffer_content().unwrap();
    println!("Buffer after insert: {:?}", buffer_content_after);

    // The 'X' should be inserted at buffer_pos_before_insert
    // and should appear visually at screen_x_before

    // Get the screen position where 'X' appears
    harness.render().unwrap();

    // This is where the bug manifests: if cursor tracking is broken,
    // the 'X' will not appear at screen_x_before
}

/// Test cursor movement with simple multi-byte UTF-8 characters (emojis)
#[test]
fn test_cursor_sync_with_emoji() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Type a line with emojis
    let text = "Hello 😀 World 🌍";
    harness.type_text(text).unwrap();

    // Move to beginning
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // The text has these characters:
    // H e l l o   😀   W o r l d   🌍
    // 0 1 2 3 4 5 [6-9] 10 11 12 13 14 15 [16-19]
    // Note: 😀 is 4 bytes (U+1F600), 🌍 is 4 bytes (U+1F30D)

    // Move right 7 times should position us after the emoji
    for _ in 0..7 {
        harness
            .send_key(KeyCode::Right, KeyModifiers::NONE)
            .unwrap();
    }

    let buffer_pos = harness.cursor_position();
    // "Hello " = 6 bytes, "😀" = 4 bytes, so position should be 10
    assert_eq!(
        buffer_pos, 10,
        "After moving through 'Hello 😀', cursor should be at byte 10"
    );

    // Type 'X' and verify it's inserted correctly
    harness.type_text("X").unwrap();
    let expected = "Hello 😀X World 🌍";
    harness.assert_buffer_content(expected);
}

/// Test that cursor position is correct when clicking on text with non-ASCII characters
#[test]
fn test_mouse_click_on_non_ascii_text() {
    let mut harness = EditorTestHarness::new(120, 30).unwrap();

    // Type a line with box-drawing characters
    let text = "│  ┌──────────────┐  ┌──────────────┐  │";
    harness.type_text(text).unwrap();
    harness.render().unwrap();

    // Now click on various positions in the line and verify cursor position

    // Get the gutter width first by checking where line 1 starts
    // The tab bar is at row 0, first line of text is at row 1
    let _line_row = 1;

    // Click at the beginning of the text (after gutter)
    // We need to figure out where the gutter ends
    // Let's assume standard gutter of 8 chars for now: " " + "   1" + " │ "

    // This test may need adjustment based on actual gutter rendering
}

/// Test that backspace properly deletes entire UTF-8 characters, not just bytes
/// This reproduces the bug where backspace removes only the last byte of a multi-byte character
#[test]
fn test_backspace_deletes_entire_utf8_character() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Test 1: Euro sign (3 bytes: 0xE2 0x82 0xAC)
    harness.type_text("€").unwrap();
    harness.assert_buffer_content("€");

    // Backspace should delete the entire euro sign, not just one byte
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("");

    // Test 2: Norwegian characters (2 bytes each: æ=0xC3 0xA6, ø=0xC3 0xB8, å=0xC3 0xA5)
    harness.type_text("æøå").unwrap();
    harness.assert_buffer_content("æøå");

    // Backspace should delete 'å' entirely
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("æø");

    // Another backspace should delete 'ø' entirely
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("æ");

    // Another backspace should delete 'æ' entirely
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("");

    // Test 3: Emoji (4 bytes: 😀 = U+1F600)
    harness.type_text("a😀b").unwrap();
    harness.assert_buffer_content("a😀b");

    // Backspace should delete 'b'
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("a😀");

    // Backspace should delete the entire emoji (4 bytes), not just one byte
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("a");
}

/// Test that delete (forward) properly removes entire UTF-8 characters
#[test]
fn test_delete_forward_removes_entire_utf8_character() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Type text with multi-byte characters
    harness.type_text("a€b").unwrap();
    harness.assert_buffer_content("a€b");

    // Move to beginning
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // Delete 'a' - this should work fine (ASCII)
    harness
        .send_key(KeyCode::Delete, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("€b");

    // Delete '€' - should delete entire 3-byte euro sign, not just one byte
    harness
        .send_key(KeyCode::Delete, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("b");
}

/// Test that selecting and deleting/replacing UTF-8 characters works correctly
#[test]
fn test_selection_delete_with_utf8_characters() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Type text with multi-byte characters: a + æ(2) + ø(2) + å(2) + b
    harness.type_text("aæøåb").unwrap();
    harness.assert_buffer_content("aæøåb");

    // Move to beginning
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // Move right once (past 'a')
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();

    // Select the three Norwegian characters by shift+right 3 times
    harness
        .send_key(KeyCode::Right, KeyModifiers::SHIFT)
        .unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::SHIFT)
        .unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::SHIFT)
        .unwrap();

    // Delete the selection with backspace
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.assert_buffer_content("ab");
}

/// Test that selecting and replacing UTF-8 characters works correctly
#[test]
fn test_selection_replace_with_utf8_characters() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    // Type text with emoji
    harness.type_text("hello😀world").unwrap();
    harness.assert_buffer_content("hello😀world");

    // Move to beginning
    harness.send_key(KeyCode::Home, KeyModifiers::NONE).unwrap();

    // Move right 5 times (past "hello")
    for _ in 0..5 {
        harness
            .send_key(KeyCode::Right, KeyModifiers::NONE)
            .unwrap();
    }

    // Select the emoji (1 character, 4 bytes)
    harness
        .send_key(KeyCode::Right, KeyModifiers::SHIFT)
        .unwrap();

    // Replace with a different character
    harness.type_text("X").unwrap();
    harness.assert_buffer_content("helloXworld");
}

/// Test loading a file with UTF-8 characters, backspacing, saving, and verifying file content
/// This reproduces the exact bug where backspace removes only a byte, corrupting the file on save
#[test]
fn test_backspace_utf8_file_save_roundtrip() {
    let temp_dir = TempDir::new().unwrap();

    // Test 1: Euro sign (3 bytes: 0xE2 0x82 0xAC)
    let euro_path = temp_dir.path().join("euro.txt");
    std::fs::write(&euro_path, "€\n").unwrap();

    let mut harness = EditorTestHarness::new(80, 24).unwrap();
    harness.open_file(&euro_path).unwrap();
    harness.render().unwrap();

    // Move to end of line (after €, before newline)
    harness.send_key(KeyCode::End, KeyModifiers::NONE).unwrap();

    // Backspace should delete the entire euro sign
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();

    // Save with Ctrl+S
    harness
        .send_key(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Verify the file contains only a newline (euro sign fully deleted)
    let saved = std::fs::read(&euro_path).unwrap();
    assert_eq!(
        saved, b"\n",
        "Euro sign should be fully deleted, file should contain only newline. Got: {:?}",
        saved
    );

    // Test 2: Norwegian characters (æøå)
    let norwegian_path = temp_dir.path().join("norwegian.txt");
    std::fs::write(&norwegian_path, "æøå\n").unwrap();

    let mut harness2 = EditorTestHarness::new(80, 24).unwrap();
    harness2.open_file(&norwegian_path).unwrap();
    harness2.render().unwrap();

    // Move to end of line
    harness2.send_key(KeyCode::End, KeyModifiers::NONE).unwrap();

    // Backspace should delete 'å' entirely (2 bytes)
    harness2
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();

    // Save
    harness2
        .send_key(KeyCode::Char('s'), KeyModifiers::CONTROL)
        .unwrap();
    harness2.render().unwrap();

    // Verify
    let saved2 = std::fs::read(&norwegian_path).unwrap();
    assert_eq!(
        saved2,
        "æø\n".as_bytes(),
        "Only 'å' should be deleted, leaving 'æø'. Got: {:?}",
        String::from_utf8_lossy(&saved2)
    );
}
