use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Part {
    pub id: i64,
    pub name: String,
    pub description: String,
}

pub fn default_bin_placement() -> i64 {
    -1
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct PartWithCountAndStock {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub count: i64,
    #[serde(default)]
    pub stock: i64,
    #[serde(default = "default_bin_placement")]
    pub column: i64,
    #[serde(default = "default_bin_placement")]
    pub row: i64,
    #[serde(default = "default_bin_placement")]
    pub z: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct PartWithStock {
    pub id: i64,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub stock: i64,
    #[serde(default = "default_bin_placement")]
    pub column: i64,
    #[serde(default = "default_bin_placement")]
    pub row: i64,
    #[serde(default = "default_bin_placement")]
    pub z: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct Bin {
    pub id: i64,
    pub profile_id: i64,
    pub row: i64,
    pub column: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct PartBinRelation {
    pub id: i64,
    pub part_id: i64,
    pub bin_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Bom {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct BomPartRelation {
    pub id: i64,
    pub bom_id: i64,
    pub part_id: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Tabled)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct ProfileBomRelation {
    pub id: i64,
    pub profile_id: i64,
    pub bom_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub profile_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub expires: Option<i64>, // nullable timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
pub struct StockRows {
    id: i64,
    bin_id: i64,
    row: i64,
    column: i64,
    z: i64,
    part_id: i64,
    name: String,
    description: String,
    stock: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomWithParts {
    pub bom: Bom,
    pub parts: PartWithStock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequirement {
    pub part: PartWithStock,
    pub required: i64,
    pub shortfall: i64,
    pub bom_sources: Vec<BomSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomSource {
    pub bom_name: String,
    pub bom_id: i64,
    pub quantity_needed: i64,
    pub builds: i64,
}
