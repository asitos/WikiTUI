use crate::api::NetworkEvent;
use crate::app::pane::PaneContent;
use crate::app::App;
use crate::parser::parse_wikipedia_html;

impl App {
    pub fn handle_network_event(&mut self, ev: NetworkEvent) {
        match ev {
            NetworkEvent::SearchResult {
                request_id,
                pane_id,
                query,
                results,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    if request_id >= pane.current_request_id {
                        pane.is_loading = false;
                        pane.loading_title = None;
                        pane.selected_idx = 0;
                        pane.scroll_offset = 0;
                        pane.toc_focused = false;
                        pane.content = PaneContent::SearchResults {
                            query,
                            items: results,
                        };
                    }
                }
            }
            NetworkEvent::ArticleResult {
                request_id,
                pane_id,
                title,
                content,
            } => {
                let is_current = self
                    .find_pane_mut(pane_id)
                    .is_some_and(|p| request_id >= p.current_request_id);

                if is_current {
                    self.record_recent_article(&title);
                    let show_footnotes = self.config.reader.show_footnotes;
                    let show_external_links = self.config.reader.show_external_links;
                    let heading_marker = self.config.reader.heading_marker;
                    let code_line_numbers = self.config.reader.code_line_numbers;
                    let show_icons = self.config.ui.icons;
                    if let Some(pane) = self.find_pane_mut(pane_id) {
                        pane.is_loading = false;
                        pane.loading_title = None;
                        pane.toc_focused = false;
                        let initial_width = 80;
                        let parsed_doc = parse_wikipedia_html(
                            &content,
                            initial_width,
                            show_footnotes,
                            show_external_links,
                            heading_marker,
                            code_line_numbers,
                            show_icons,
                        );
                        pane.scroll_offset = pane
                            .scroll_offset
                            .min(parsed_doc.lines.len().saturating_sub(1));
                        let initial_link_idx = if !parsed_doc.links.is_empty() {
                            Some(0)
                        } else {
                            None
                        };
                        pane.content = PaneContent::ArticleText {
                            title,
                            raw_html: content,
                            parsed_doc: Box::new(parsed_doc),
                            last_width: initial_width,
                            last_show_footnotes: show_footnotes,
                            last_show_external_links: show_external_links,
                            last_heading_marker: heading_marker,
                            last_code_line_numbers: code_line_numbers,
                            last_show_icons: show_icons,
                        };
                        pane.selected_link_idx = initial_link_idx;
                    }
                }
            }
            NetworkEvent::Error {
                request_id,
                pane_id,
                message,
            } => {
                if let Some(pane) = self.find_pane_mut(pane_id) {
                    if request_id >= pane.current_request_id {
                        pane.is_loading = false;
                        pane.loading_title = None;
                        pane.content = PaneContent::Error(message);
                    }
                }
            }
            NetworkEvent::FeedBatchLoaded { items } => {
                self.feed.is_fetching = false;
                for mut item in items {
                    item.is_liked = self.feed.profile.liked_articles.contains(&item.title)
                        || self.saved_lists.is_article_in_list("liked", &item.title);
                    self.feed.add_item(item);
                }
            }
            NetworkEvent::DailyFeedLoaded(feed) => {
                self.daily_feed = Some(*feed);
            }
            NetworkEvent::StatsLoaded(stats) => {
                self.wiki_stats = stats;
            }
        }
    }
}
