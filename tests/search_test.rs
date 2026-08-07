use tokio::sync::mpsc;
use wiki_tui::api::SearchResultItem;
use wiki_tui::app::{App, PaneContent};

fn result(title: &str, snippet: &str) -> SearchResultItem {
    SearchResultItem {
        title: title.to_string(),
        snippet: snippet.to_string(),
    }
}

#[test]
fn search_selection_scrolls_by_rendered_lines() {
    let (tx, _) = mpsc::unbounded_channel();
    let mut app = App::new(tx);
    let pane = app.active_pane_mut();
    pane.viewport_height = 4;
    pane.content = PaneContent::SearchResults {
        query: "test".to_string(),
        items: vec![
            result("one", ""),
            result("two", "snippet"),
            result("three", ""),
        ],
    };

    app.select_next_item(24);
    assert_eq!(
        (
            app.active_pane().selected_idx,
            app.active_pane().scroll_offset
        ),
        (1, 1)
    );

    app.select_next_item(24);
    assert_eq!(
        (
            app.active_pane().selected_idx,
            app.active_pane().scroll_offset
        ),
        (2, 3)
    );

    app.select_prev_item();
    assert_eq!(
        (
            app.active_pane().selected_idx,
            app.active_pane().scroll_offset
        ),
        (1, 2)
    );
}
