use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub stock: i64,
    pub bin_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bin {
    pub id: i64,
    pub row: i64,
    pub column: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartBinRelation {
    pub id: i64,
    pub part_id: i64,
    pub bin_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bom {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomPartRelation {
    pub id: i64,
    pub bom_id: i64,
    pub part_id: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileBomRelation {
    pub id: i64,
    pub profile_id: i64,
    pub bom_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub profile_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub expires: Option<i64>, // nullable timestamp
}
