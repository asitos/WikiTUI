use crate::app::pane::Pane;
use crate::layout::LayoutNode;

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
            panes: vec![Pane::new(initial_pane_id)],
            active_pane_idx: 0,
            layout_root: LayoutNode::Leaf(0),
        }
    }
}
