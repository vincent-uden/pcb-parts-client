use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Part {
    pub id: i64,
    pub name: String,
    pub description: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
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
pub struct BomProfileRow {
    boms: Bom,
    profile_boms: ProfileBomRelation,
    profile: Profile,
}
