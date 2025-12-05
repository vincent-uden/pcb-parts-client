use common::models::{Bom, PartWithStock, PurchaseRequirement};

pub mod widget;

#[derive(Debug, Clone)]
pub enum Msg {
    // BOM Selection
    SearchBom(String),
    SubmitSearch,
    SearchResults(Vec<Bom>),
    SearchFailed(String),
    SelectBom(Bom),
    RemoveBom(i64),
    UpdateQuantity(i64, String),

    // Planning calculation
    CalculatePlan,
    PlanCalculated(Vec<PurchaseRequirement>),
    PlanFailed(String),

    // Part interaction
    HoverPart(PartWithStock),
    ClearHover,

    // Export
    ExportPath(String),
    ExportCsv,
    ExportSuccess,
    ExportFailed(String),
}

#[derive(Debug, Clone)]
pub struct SelectedBom {
    pub bom: Bom,
    pub quantity: i64,
}
