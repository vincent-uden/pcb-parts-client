use common::models::PartWithStock;

pub mod widget;

#[derive(Debug, Clone)]
pub enum GridMessage {
    HighlightParts(Vec<PartWithStock>),
    LayerUp,
    LayerDown,
}
