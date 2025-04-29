use common::models::{Bom, Part, PartWithStock};

pub mod widget;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    Toggle,
    PendingQuery(String),
    SubmitQuery,
    PartSearchResult(Vec<PartWithStock>),
    BomSearchResult(Vec<Bom>),
    FailedSearch(String),
    ChangeStock(PartWithStock),
    OpenBom(Bom),
}
