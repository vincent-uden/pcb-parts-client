use common::models::{PartWithCountAndStock, PartWithStock};

pub mod widget;

#[derive(Debug, Clone)]
pub enum GridMessage {
    HighlightParts(Vec<PartWithCountAndStock>),
    LayerUp,
    LayerDown,
    CellClicked(i64, i64), // row, column
    SetSelectionMode(bool),
    HighlightTargetBin(Option<(i64, i64)>), // row, column for target bin
}
