use crate::gpui_app::model::AppState;
use crate::source::SourceEvent;
use gpui::TestAppContext;
use std::sync::mpsc;

#[gpui::test]
async fn test_app_state_filtering(cx: &mut TestAppContext) {
    let (tx, _rx) = mpsc::channel::<SourceEvent>();
    
    // Create the model using AppState::new inside cx.update
    let state = cx.update(|cx| {
        AppState::new(cx, Some(tx))
    });

    // Add some logs
    state.update(cx, |state, cx| {
        state.add_line("INFO: System started".to_string(), cx);
        state.add_line("ERROR: Connection failed".to_string(), cx);
        state.add_line("INFO: Retrying...".to_string(), cx);
        state.add_line("DEBUG: details".to_string(), cx);
    });

    // Verify initial state (all lines shown)
    state.update(cx, |state, _| {
        assert_eq!(state.lines.len(), 4);
        assert_eq!(state.filtered_indices.len(), 4);
    });

    // Apply Filter "ERROR"
    state.update(cx, |state, cx| {
        state.filter_text = "ERROR".to_string();
        state.apply_filter(cx);
    });

    state.update(cx, |state, _| {
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.filtered_indices[0], 1); // Index of ERROR line
        assert_eq!(state.lines[state.filtered_indices[0]].content, "ERROR: Connection failed");
    });

    // Apply Hide "INFO" with NO filter
    state.update(cx, |state, cx| {
        state.filter_text = "".to_string(); // Clear filter
        state.apply_filter(cx);
        state.hide_text = "INFO".to_string();
        state.apply_hide(cx);
    });

    state.update(cx, |state, _| {
        // "Hide" redacts text, but doesn't remove line if filter is empty
        assert_eq!(state.filtered_indices.len(), 4); 
        
        let line = &state.lines[0]; // "INFO: System started"
        let display = state.get_display_content(line);
        assert_eq!(display, ": System started"); // "INFO" removed
    });
    
    // Apply Hide "INFO" AND Filter "INFO"
    state.update(cx, |state, cx| {
        state.filter_text = "INFO".to_string();
        state.apply_filter(cx);
    });
    
    state.update(cx, |state, _| {
        // Should find 0 matches because "INFO" is hidden/removed before filtering
        assert_eq!(state.filtered_indices.len(), 0);
    });
    
    // Test Highlight (doesn't filter, just highlights)
    state.update(cx, |state, cx| {
            state.filter_text = "".to_string();
            state.apply_filter(cx);
            state.hide_text = "".to_string();
            state.apply_hide(cx);
            state.highlight_text = "started".to_string();
            state.apply_highlight(cx);
    });
    
    state.update(cx, |state, _| {
        assert_eq!(state.filtered_indices.len(), 4);
        assert!(state.filter_state.highlight_expr.is_some());
    });
}

#[gpui::test]
async fn test_follow_tail(cx: &mut TestAppContext) {
    let (tx, _rx) = mpsc::channel::<SourceEvent>();
    let state = cx.update(|cx| {
        AppState::new(cx, Some(tx))
    });

    state.update(cx, |state, _| {
        assert!(state.follow_tail);
    });

    // Toggle follow tail
    state.update(cx, |state, cx| {
        state.toggle_follow_tail(cx);
    });

    state.update(cx, |state, _| {
        assert!(!state.follow_tail);
    });
}

#[gpui::test]
async fn test_clear(cx: &mut TestAppContext) {
    let (tx, _rx) = mpsc::channel::<SourceEvent>();
    let state = cx.update(|cx| {
        AppState::new(cx, Some(tx))
    });

    state.update(cx, |state, cx| {
        state.add_line("Log 1".to_string(), cx);
        state.add_line("Log 2".to_string(), cx);
    });

    state.update(cx, |state, _| {
        assert_eq!(state.lines.len(), 2);
    });

    state.update(cx, |state, cx| {
        state.clear(cx);
    });

    state.update(cx, |state, _| {
        assert_eq!(state.lines.len(), 0);
        assert_eq!(state.filtered_indices.len(), 0);
    });
}
