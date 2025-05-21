use std::{fs::File, path::Path};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder};

use crate::models::Part;

pub fn csv_to_bom(
    path: &Path,
    name_col: &str,
    desc_col: &str,
    count_col: &str,
) -> Result<Vec<(i64, Part)>> {
    let rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    reader_to_bom(rdr, name_col, desc_col, count_col)
}

pub fn reader_to_bom<T>(
    mut rdr: Reader<T>,
    name_col: &str,
    desc_col: &str,
    count_col: &str,
) -> Result<Vec<(i64, Part)>>
where
    T: std::io::Read,
{
    let mut out = vec![];

    let headers = rdr.headers()?;
    let name_idx = headers
        .iter()
        .enumerate()
        .find(|(_, s)| *s == name_col)
        .ok_or(anyhow!("Name column not found"))?
        .0;
    let desc_idx = headers
        .iter()
        .enumerate()
        .find(|(_, s)| *s == desc_col)
        .ok_or(anyhow!("Description column not found"))?
        .0;
    let count_idx = headers
        .iter()
        .enumerate()
        .find(|(_, s)| *s == count_col)
        .ok_or(anyhow!("Count column not found"))?
        .0;

    for record in rdr.byte_records() {
        let r = record?;
        let name =
            String::from_utf8_lossy(r.get(name_idx).ok_or(anyhow!("Non-homogeneous csv file"))?)
                .to_string();
        let description =
            String::from_utf8_lossy(r.get(desc_idx).ok_or(anyhow!("Non-homogeneous csv file"))?)
                .to_string();
        let count = String::from_utf8_lossy(
            r.get(count_idx)
                .ok_or(anyhow!("Non-homogeneous csv file"))?,
        )
        .parse()?;

        out.push((
            count,
            Part {
                id: 0,
                name,
                description,
            },
        ));
    }

    Ok(out)
}

pub fn csv_to_headers(path: &Path) -> Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let headers = rdr.headers()?;
    Ok(headers.iter().map(String::from).collect())
}

#[cfg(test)]
mod tests {
    use csv::ReaderBuilder;

    use super::reader_to_bom;

    #[test]
    fn can_parse_altium_bom() {
        let bytes = include_bytes!("../assets/Magnet Harvesting 1.2.csv");
        let rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&bytes[..]);

        let bom = reader_to_bom(rdr, "Name", "Description", "Quantity").unwrap();
        assert_eq!(bom.len(), 10, "BOM should have 10 entires");
    }
}
