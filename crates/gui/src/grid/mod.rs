use common::models::{PartWithCountAndStock, PartWithStock};

pub mod widget;

#[derive(Debug, Clone)]
pub enum GridMessage {
    HighlightParts(Vec<PartWithCountAndStock>),
    LayerUp,
    LayerDown,
}
