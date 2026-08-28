#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDragTarget {
    Pane(usize),
    Toc,
    SavedLists(bool),
}
