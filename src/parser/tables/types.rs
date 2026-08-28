use crate::parser::types::StyledToken;

pub type CellLinkInfo = (String, String, Vec<(usize, usize)>);

#[derive(Debug, Clone)]
pub(crate) enum CellEntry {
    Origin {
        tokens: Vec<StyledToken>,
        colspan: usize,
        rowspan: usize,
        is_header: bool,
    },
    Covered {
        origin_r: usize,
        origin_c: usize,
    },
}

pub(crate) struct TableGrid {
    pub num_rows: usize,
    pub num_cols: usize,
    pub cells: Vec<Vec<CellEntry>>,
    pub caption: Option<String>,
}
