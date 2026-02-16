// End-to-end tests for folding behavior and interactions

use crate::common::fixtures::TestFixture;
use crate::common::harness::{layout, EditorTestHarness};
use crossterm::event::{KeyCode, KeyModifiers};
use lsp_types::FoldingRange;

fn set_fold_range(harness: &mut EditorTestHarness, start_line: usize, end_line: usize) {
    let state = harness.editor_mut().active_state_mut();
    state.folding_ranges = vec![FoldingRange {
        start_line: start_line as u32,
        end_line: end_line as u32,
        start_character: None,
        end_character: None,
        kind: None,
        collapsed_text: None,
    }];
}

fn set_top_line(harness: &mut EditorTestHarness, line: usize) {
    let top_byte = {
        let buffer = &mut harness.editor_mut().active_state_mut().buffer;
        buffer
            .line_start_offset(line)
            .unwrap_or_else(|| buffer.len())
    };
    let viewport = harness.editor_mut().active_viewport_mut();
    viewport.top_byte = top_byte;
    viewport.top_view_line_offset = 0;

    let cursors = harness.editor_mut().active_cursors_mut();
    cursors.primary_mut().position = top_byte;
    cursors.primary_mut().anchor = None;
    cursors.primary_mut().sticky_column = 0;
}

fn set_cursor_line(harness: &mut EditorTestHarness, line: usize) {
    let pos = {
        let buffer = &mut harness.editor_mut().active_state_mut().buffer;
        buffer
            .line_start_offset(line)
            .unwrap_or_else(|| buffer.len())
    };
    let cursors = harness.editor_mut().active_cursors_mut();
    cursors.primary_mut().position = pos;
    cursors.primary_mut().anchor = None;
    cursors.primary_mut().sticky_column = 0;
}

fn find_text_position(harness: &EditorTestHarness, needle: &str) -> (u16, u16) {
    let (start_row, end_row) = harness.content_area_rows();
    for row in start_row..=end_row {
        let text = harness.get_row_text(row as u16);
        if let Some(col) = text.find(needle) {
            return (row as u16, col as u16);
        }
    }
    panic!(
        "Expected to find '{}' on screen.\nScreen:\n{}",
        needle,
        harness.screen_to_string()
    );
}

#[test]
fn test_fold_gutter_double_click_toggles_like_single() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let content: String = (0..30).map(|i| format!("line {i}\n")).collect();
    let fixture = TestFixture::new("fold_double_click.py", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();

    set_fold_range(&mut harness, 2, 6);
    harness.render().unwrap();

    let row = (layout::CONTENT_START_ROW + 2) as u16;
    let col = 0;

    // Two rapid clicks at the same gutter position should act like two single clicks
    // (fold then unfold), not trigger word selection.
    harness.mouse_click(col, row).unwrap();
    harness.mouse_click(col, row).unwrap();

    // After two toggles, the folded lines should be visible again.
    let row_text = harness.get_row_text(row + 1);
    assert!(
        row_text.contains("line 3"),
        "Expected folded lines to be visible after double click. Row text: '{row_text}'"
    );
}

#[test]
fn test_fold_click_moves_cursor_out_of_fold() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let content: String = (0..30).map(|i| format!("line {i}\n")).collect();
    let fixture = TestFixture::new("fold_cursor_inside.py", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();

    set_fold_range(&mut harness, 2, 6);
    harness.render().unwrap();

    // Move cursor into the fold body (line 4).
    harness
        .send_key_repeat(KeyCode::Down, KeyModifiers::NONE, 4)
        .unwrap();

    let cursor_line_before = harness
        .editor()
        .active_state()
        .buffer
        .get_line_number(harness.editor().active_cursors().primary().position);
    assert_eq!(
        cursor_line_before, 4,
        "Precondition failed: cursor not inside fold body."
    );

    let row = (layout::CONTENT_START_ROW + 2) as u16;
    harness.mouse_click(0, row).unwrap();

    let cursor_line_after = harness
        .editor()
        .active_state()
        .buffer
        .get_line_number(harness.editor().active_cursors().primary().position);
    assert_eq!(
        cursor_line_after, 2,
        "Cursor should move to fold header when collapsing."
    );

    let row_text = harness.get_row_text(row + 1);
    assert!(
        row_text.contains("line 7"),
        "Expected fold to collapse even when cursor was inside. Row text: '{row_text}'"
    );
}

#[test]
fn test_mouse_scroll_skips_folded_lines() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let content: String = (0..120).map(|i| format!("line {i}\n")).collect();
    let fixture = TestFixture::new("fold_scroll.py", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();

    let header_line = 10usize;
    let end_line = 20usize;
    set_fold_range(&mut harness, header_line, end_line);
    harness.render().unwrap();
    let header_row = (layout::CONTENT_START_ROW + header_line) as u16;
    harness.mouse_click(0, header_row).unwrap();

    set_top_line(&mut harness, header_line);
    harness.render().unwrap();

    // Scroll down once; top line should not land inside the folded range.
    harness
        .mouse_scroll_down(0, layout::CONTENT_START_ROW as u16)
        .unwrap();

    let top_line = harness.top_line_number();
    assert!(
        top_line <= header_line || top_line > end_line,
        "Top line should skip folded region. top_line={top_line}, folded=[{}..{}]",
        header_line + 1,
        end_line
    );
}

#[test]
fn test_cursor_down_skips_folded_lines() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let content: String = (0..40).map(|i| format!("line {i}\n")).collect();
    let fixture = TestFixture::new("fold_cursor_down.py", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();

    let header_line = 2usize;
    let end_line = 6usize;
    set_fold_range(&mut harness, header_line, end_line);
    harness.render().unwrap();

    // Collapse the fold without moving the cursor into it.
    let buffer_id = harness.editor().active_buffer();
    harness
        .editor_mut()
        .toggle_fold_at_line(buffer_id, header_line);
    harness.render().unwrap();

    // Move cursor to line before header (line 1).
    let line1_byte = harness
        .editor_mut()
        .active_state_mut()
        .buffer
        .line_start_offset(1)
        .unwrap();
    harness
        .editor_mut()
        .active_cursors_mut()
        .primary_mut()
        .position = line1_byte;
    harness.render().unwrap();

    // Move down into the fold; it should skip to the first visible line after the fold.
    harness
        .send_key_repeat(KeyCode::Down, KeyModifiers::NONE, 2)
        .unwrap();

    let cursor_line_after = harness
        .editor()
        .active_state()
        .buffer
        .get_line_number(harness.editor().active_cursors().primary().position);
    assert_eq!(
        cursor_line_after,
        end_line + 1,
        "Cursor should skip folded lines when moving down."
    );

    let row = (layout::CONTENT_START_ROW + header_line) as u16;
    let row_text = harness.get_row_text(row + 1);
    assert!(
        row_text.contains("line 7"),
        "Fold should remain collapsed while moving down. Row text: '{row_text}'"
    );
}

#[test]
fn test_folding_preserves_syntax_highlighting_after_skip() {
    let mut harness = EditorTestHarness::new(80, 24).unwrap();

    let mut lines: Vec<String> = (0..120).map(|i| format!("line {i}\n")).collect();
    lines[80] = "def highlighted_function():\n".to_string();
    lines[81] = "    return 1\n".to_string();
    let content = lines.concat();

    let fixture = TestFixture::new("fold_highlight.py", &content).unwrap();
    harness.open_file(&fixture.path).unwrap();

    // Baseline: capture highlight style for "def" without folding.
    set_cursor_line(&mut harness, 80);
    harness.render().unwrap();

    let (def_row, def_col) = find_text_position(&harness, "def highlighted_function");
    let (plain_row, plain_col) = find_text_position(&harness, "line 79");

    let def_style = harness
        .get_cell_style(def_col, def_row)
        .expect("Expected style for 'def'");
    let plain_style = harness
        .get_cell_style(plain_col, plain_row)
        .expect("Expected style for plain text");

    assert_ne!(
        def_style, plain_style,
        "Precondition failed: keyword highlight should differ from plain text."
    );

    // Fold a large range above the highlighted line.
    set_fold_range(&mut harness, 10, 70);
    harness.render().unwrap();
    let header_row = (layout::CONTENT_START_ROW + 10) as u16;
    harness.mouse_click(0, header_row).unwrap();
    set_cursor_line(&mut harness, 80);
    harness.render().unwrap();

    let (def_row_after, def_col_after) = find_text_position(&harness, "def highlighted_function");
    let def_style_after = harness
        .get_cell_style(def_col_after, def_row_after)
        .expect("Expected style for 'def' after folding");

    assert_eq!(
        def_style_after, def_style,
        "Syntax highlighting should remain stable after folding."
    );
}
