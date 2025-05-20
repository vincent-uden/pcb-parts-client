use std::path::PathBuf;

use common::models::PartWithStock;
use serde::{Deserialize, Serialize};

pub mod widget;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartCandidate {
    pub name: String,
    pub description: String,
    pub count: i64,
    pub linked_part: Option<PartWithStock>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    PendingPath(String),
    OpenFile,
    OpenSuccess(Vec<String>),
    OpenFailed,
    BomName(String),
    SelectNameColumn(String),
    SelectDescriptionColumn(String),
    SelectCountColumn(String),
}
