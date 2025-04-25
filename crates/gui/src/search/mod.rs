use common::models::Part;

pub mod widget;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    PendingQuery(String),
    SubmitQuery,
    PartSearchResult(Vec<Part>),
    FailedSearch(String),
}
