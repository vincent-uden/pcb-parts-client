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
    DepleteBom(Bom),
    RestockBom(Bom),
    StockQuantity(String),
    OpenBom(Bom),
    CloseBom,
    RefreshBom(Bom),
    StockChangeFailed,
    StockChangeSuccess(i64),
}
