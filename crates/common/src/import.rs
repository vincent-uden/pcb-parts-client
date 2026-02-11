use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use altium_format::{
    RecordTree,
    io::SchDoc,
    ops::{
        output::BomItem,
        util::{alphanumeric_sort, get_component_designator},
    },
    records::sch::{SchPrimitive, SchRecord},
};
use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder};
use tracing::info;

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
        let desc_bytes = r
            .get(desc_idx)
            .ok_or(anyhow!("Non-homogeneous csv file"))?
            .to_owned();
        let description = match String::from_utf8(desc_bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                let (s, _, _) = encoding_rs::WINDOWS_1251.decode(&desc_bytes);
                s.to_string()
            }
        };
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
    out.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
    Ok(out)
}

pub fn csv_to_headers(path: &Path) -> Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let headers = rdr.headers()?;
    Ok(headers.iter().map(String::from).collect())
}

pub fn altium_schematic_file_to_bom(path: &Path) -> Result<Vec<(i64, Part)>> {
    let file = fs::File::open(path)?;
    let rdr = BufReader::new(file);
    altium_schematic_reader_to_bom(rdr)
}

pub fn altium_schematic_reader_to_bom<T>(rdr: BufReader<T>) -> Result<Vec<(i64, Part)>>
where
    T: std::io::Read + std::io::Seek,
{
    let doc = SchDoc::open(rdr)?;
    let tree = RecordTree::from_records(doc.primitives.clone());

    let mut bom: HashMap<String, (i64, Part)> = HashMap::new();
    for (id, record) in tree.iter() {
        if let SchRecord::Component(c) = record {
            let mut comment = None;
            let mut part_number = None;
            let mut manufacturer_part_number = None;
            for (_id, child) in tree.children(id) {
                match child {
                    SchRecord::Parameter(p) => {
                        // The part name is stored in the comment for some reason
                        match p.name.as_str() {
                            "Comment" => {
                                comment = Some(p.label.text.clone());
                            }
                            "Part Number" => {
                                part_number = Some(p.label.text.clone());
                            }
                            "Manufacturer Part Number" => {
                                manufacturer_part_number = Some(p.label.text.clone());
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            let name_prio = vec![manufacturer_part_number, part_number, comment];
            let name = name_prio
                .into_iter()
                .find(|p| p.is_some())
                .unwrap()
                .unwrap();
            if !bom.contains_key(&name) {
                bom.insert(
                    name.clone(),
                    (
                        1,
                        Part {
                            id: 0,
                            name: name.clone(),
                            description: c.component_description.clone(),
                        },
                    ),
                );
            } else {
                bom.get_mut(&name).unwrap().0 += 1;
            }
        }
    }

    let mut out: Vec<(i64, Part)> = bom.into_values().collect();
    out.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufReader};

    use csv::ReaderBuilder;

    use crate::import::altium_schematic_reader_to_bom;

    use super::reader_to_bom;

    #[test]
    fn can_parse_altium_bom() {
        let bytes = include_bytes!("../assets/Magnet Harvesting 1.2.csv");
        let rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&bytes[..]);

        let bom = reader_to_bom(rdr, "Name", "Description", "Quantity").unwrap();
        assert_eq!(bom.len(), 13, "BOM should have 10 entires");
    }

    #[test]
    fn can_parse_altium_schdoc() {
        let bytes = include_bytes!("../assets/MagnetV1_2.SchDoc");
        let cursor = io::Cursor::new(bytes);
        let rdr = BufReader::new(cursor);

        let bom = altium_schematic_reader_to_bom(rdr).unwrap();
        assert_eq!(bom.len(), 13, "BOM should have 10 entries");
    }

    #[test]
    fn csv_and_schdoc_create_identical_boms() {
        let csv_bytes = include_bytes!("../assets/Magnet Harvesting 1.2.csv");
        let sch_bytes = include_bytes!("../assets/MagnetV1_2.SchDoc");

        let csv_rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&csv_bytes[..]);

        let csv_bom = reader_to_bom(csv_rdr, "Name", "Description", "Quantity").unwrap();

        let cursor = io::Cursor::new(sch_bytes);
        let sch_rdr = BufReader::new(cursor);

        let sch_bom = altium_schematic_reader_to_bom(sch_rdr).unwrap();
        pretty_assertions::assert_eq!(csv_bom, sch_bom);
    }
}
