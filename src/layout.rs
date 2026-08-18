use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutNode {
    Leaf(usize),
    Split {
        direction: SplitDirection,
        left: Box<LayoutNode>,
        right: Box<LayoutNode>,
    },
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::Leaf(0)
    }
}

impl LayoutNode {
    pub fn compute_rects(&self, rect: Rect) -> Vec<(usize, Rect)> {
        match self {
            LayoutNode::Leaf(idx) => vec![(*idx, rect)],
            LayoutNode::Split {
                direction,
                left,
                right,
            } => {
                let dir = match direction {
                    SplitDirection::Horizontal => Direction::Vertical, // splitting horizontally yields rows
                    SplitDirection::Vertical => Direction::Horizontal, // splitting vertically yields columns
                };
                let chunks = Layout::default()
                    .direction(dir)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rect);
                let mut rects = left.compute_rects(chunks[0]);
                rects.extend(right.compute_rects(chunks[1]));
                rects
            }
        }
    }

    pub fn split_pane(
        &mut self,
        target_idx: usize,
        new_idx: usize,
        direction: SplitDirection,
    ) -> bool {
        match self {
            LayoutNode::Leaf(idx) => {
                if *idx == target_idx {
                    *self = LayoutNode::Split {
                        direction,
                        left: Box::new(LayoutNode::Leaf(target_idx)),
                        right: Box::new(LayoutNode::Leaf(new_idx)),
                    };
                    true
                } else {
                    false
                }
            }
            LayoutNode::Split { left, right, .. } => {
                left.split_pane(target_idx, new_idx, direction)
                    || right.split_pane(target_idx, new_idx, direction)
            }
        }
    }

    pub fn remove_pane(&self, target_idx: usize) -> Option<LayoutNode> {
        match self {
            LayoutNode::Leaf(idx) => {
                if *idx == target_idx {
                    None
                } else {
                    Some(self.clone())
                }
            }
            LayoutNode::Split {
                direction,
                left,
                right,
            } => {
                if matches!(**left, LayoutNode::Leaf(idx) if idx == target_idx) {
                    return Some(*right.clone());
                }
                if matches!(**right, LayoutNode::Leaf(idx) if idx == target_idx) {
                    return Some(*left.clone());
                }

                let new_left = left.remove_pane(target_idx);
                let new_right = right.remove_pane(target_idx);

                match (new_left, new_right) {
                    (Some(l), Some(r)) => Some(LayoutNode::Split {
                        direction: *direction,
                        left: Box::new(l),
                        right: Box::new(r),
                    }),
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None,
                }
            }
        }
    }

    pub fn decrement_indices_above(&mut self, threshold: usize) {
        match self {
            LayoutNode::Leaf(idx) => {
                if *idx > threshold {
                    *idx -= 1;
                }
            }
            LayoutNode::Split { left, right, .. } => {
                left.decrement_indices_above(threshold);
                right.decrement_indices_above(threshold);
            }
        }
    }
}

pub fn find_pane_in_direction(
    rects: &[(usize, Rect)],
    active_idx: usize,
    dir: char,
) -> Option<usize> {
    let (_, active_rect) = rects.iter().find(|(idx, _)| *idx == active_idx)?;
    let active_center_x = active_rect.x as i32 + (active_rect.width as i32 / 2);
    let active_center_y = active_rect.y as i32 + (active_rect.height as i32 / 2);

    let mut best_idx = None;
    let mut min_distance = i32::MAX;

    for &(idx, r) in rects {
        if idx == active_idx {
            continue;
        }
        let cx = r.x as i32 + (r.width as i32 / 2);
        let cy = r.y as i32 + (r.height as i32 / 2);

        let dx = cx - active_center_x;
        let dy = cy - active_center_y;

        let is_valid = match dir {
            'h' => dx < 0 && dy.abs() < active_rect.height as i32,
            'l' => dx > 0 && dy.abs() < active_rect.height as i32,
            'k' => dy < 0 && dx.abs() < active_rect.width as i32,
            'j' => dy > 0 && dx.abs() < active_rect.width as i32,
            _ => false,
        };

        if is_valid {
            let distance = dx.pow(2) + dy.pow(2);
            if distance < min_distance {
                min_distance = distance;
                best_idx = Some(idx);
            }
        }
    }

    best_idx
}
