use common::models::{Bom, Part, PartWithCountAndStock, PartWithStock};

pub mod widget;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    Toggle,
    PendingQuery(String),
    SubmitQuery,
    PartSearchResult(Vec<PartWithStock>),
    BomSearchResult(Vec<Bom>),
    BomPartsSearchResult(Vec<PartWithCountAndStock>),
    FailedSearch(String),
    ChangeStock(PartWithStock),
    HoverPart(PartWithStock),
    ClearHover,
    DepleteBom(Bom),
    RestockBom(Bom),
    StockQuantity(String),
    OpenBom(Bom),
    CloseBom,
    RefreshBom(Bom),
    StockChangeFailed,
    StockChangeSuccess(i64),
    SelectPart(PartWithStock),
    CancelPartStock,
    PartStockQuantity(String),
    PartStockRow(String),
    PartStockColumn(String),
    PartStockZ(String),
    RestockPart,
    DepletePart,
    GridCellSelected(i64, i64), // row, column
    EnableGridSelection(bool),
    UpdateTargetBinHighlight,
}
