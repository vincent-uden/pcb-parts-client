use std::path::Path;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;

use crate::models::Part;

pub fn csv_to_bom(
    path: &Path,
    name_col: &str,
    desc_col: &str,
    count_col: &str,
) -> Result<Vec<(i64, Part)>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let mut out = vec![];

    let headers = rdr.headers()?;
    let name_idx = headers
        .iter()
        .enumerate()
        .find(|(i, s)| *s == name_col)
        .ok_or(anyhow!("Name column not found"))?
        .0;
    let desc_idx = headers
        .iter()
        .enumerate()
        .find(|(i, s)| *s == desc_col)
        .ok_or(anyhow!("Description column not found"))?
        .0;
    let count_idx = headers
        .iter()
        .enumerate()
        .find(|(i, s)| *s == count_col)
        .ok_or(anyhow!("Count column not found"))?
        .0;

    for record in rdr.records() {
        let r = record?;
        let name = r
            .get(name_idx)
            .ok_or(anyhow!("Non-homogeneous csv file"))?
            .to_string();
        let description = r
            .get(desc_idx)
            .ok_or(anyhow!("Non-homogeneous csv file"))?
            .to_string();
        let count = r
            .get(count_idx)
            .ok_or(anyhow!("Non-homogeneous csv file"))?
            .parse()?;

        out.push((count, Part {
            id: 0,
            name,
            description,
        }));
    }

    Ok(out)
}

pub fn csv_to_headers(path: &Path) -> Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let headers = rdr.headers()?;
    Ok(headers.iter().map(String::from).collect())
}
