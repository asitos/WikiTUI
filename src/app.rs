use crate::layout::{LayoutNode, SplitDirection};

#[derive(Clone, Debug)]
pub struct Pane {
    pub id: usize,
}

pub struct Tab {
    pub name: String,
    pub panes: Vec<Pane>,
    pub active_pane_idx: usize,
    pub layout_root: LayoutNode,
}

impl Tab {
    pub fn new(name: String, initial_pane_id: usize) -> Self {
        Self {
            name,
            panes: vec![Pane {
                id: initial_pane_id,
            }],
            active_pane_idx: 0,
            layout_root: LayoutNode::Leaf(0),
        }
    }
}

pub struct App {
    pub running: bool,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub waiting_for_split_cmd: bool,
    next_pane_id: usize,
}

impl App {
    // default launch screen
    pub fn new() -> Self {
        let mut app = Self {
            running: true,
            tabs: Vec::new(),
            active_tab_idx: 0,
            waiting_for_split_cmd: false,
            next_pane_id: 1,
        };
        // start with a default tab
        app.tabs.push(Tab::new("home".to_string(), 0));
        app
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_idx]
    }

    // tab management
    pub fn new_tab(&mut self) {
        let name = format!("tab {}", self.tabs.len() + 1);
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
        }
    }

    // split management
    pub fn split_active_pane(&mut self, direction: SplitDirection) {
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let tab = self.active_tab_mut();
        let current_pane_idx = tab.active_pane_idx;

        tab.panes.push(Pane { id: new_pane_id });
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
