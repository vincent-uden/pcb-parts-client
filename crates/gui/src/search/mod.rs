use common::models::{Part, PartWithStock};

pub mod widget;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    PendingQuery(String),
    SubmitQuery,
    PartSearchResult(Vec<PartWithStock>),
    FailedSearch(String),
    ChangeStock(PartWithStock),
}
