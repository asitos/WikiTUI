use crate::app::pane::Pane;
use crate::app::tab::Tab;
use crate::app::App;
use crate::layout::SplitDirection;

impl App {
    pub(crate) fn find_pane_mut(&mut self, target_id: usize) -> Option<&mut Pane> {
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                if pane.id == target_id {
                    return Some(pane);
                }
            }
        }
        None
    }

    pub fn new_tab(&mut self) {
        let name = "new tab".to_string();
        self.tabs.push(Tab::new(name, self.next_pane_id));
        self.next_pane_id += 1;
        self.active_tab_idx = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_idx = (self.active_tab_idx + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_tab_idx == 0 {
                self.active_tab_idx = self.tabs.len() - 1;
            } else {
                self.active_tab_idx -= 1;
            }
        }
    }

    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_idx);
            if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            }
        } else {
            let new_pane_id = self.next_pane_id;
            self.next_pane_id += 1;
            self.tabs[0] = Tab::new("home".to_string(), new_pane_id);
            self.active_tab_idx = 0;
        }
    }

    pub fn split_active_pane(&mut self, direction: SplitDirection) {
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let tab = self.active_tab_mut();
        let current_pane_idx = tab.active_pane_idx;

        tab.panes.push(Pane::new(new_pane_id));
        let new_pane_idx = tab.panes.len() - 1;

        tab.layout_root
            .split_pane(current_pane_idx, new_pane_idx, direction);
        tab.active_pane_idx = new_pane_idx;
    }

    pub fn close_active_pane(&mut self) {
        if self.active_tab().panes.len() <= 1 {
            self.close_current_tab();
            return;
        }

        let tab = self.active_tab_mut();
        let target_idx = tab.active_pane_idx;

        if let Some(new_root) = tab.layout_root.remove_pane(target_idx) {
            tab.layout_root = new_root;
            tab.layout_root.decrement_indices_above(target_idx);

            tab.panes.remove(target_idx);

            if tab.active_pane_idx >= tab.panes.len() {
                tab.active_pane_idx = tab.panes.len() - 1;
            }
        }
    }

    pub fn navigate_panes(&mut self, dir: char, term_width: u16, term_height: u16) {
        use crate::layout::find_pane_in_direction;
        use ratatui::layout::Rect;

        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = self.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        if let Some(next_idx) = find_pane_in_direction(&rects, tab.active_pane_idx, dir) {
            tab.active_pane_idx = next_idx;
        }
    }
}
