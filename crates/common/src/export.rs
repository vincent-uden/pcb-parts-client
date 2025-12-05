use std::path::Path;

use anyhow::Result;
use csv::Writer;

use crate::models::PurchaseRequirement;

pub fn export_purchase_plan_to_csv(
    path: &Path,
    requirements: &[PurchaseRequirement],
) -> Result<()> {
    let mut wtr = Writer::from_path(path)?;

    // Write header
    wtr.write_record(&[
        "Part Name",
        "Description",
        "Current Stock",
        "Total Required",
        "Need to Purchase",
        "Required By",
    ])?;

    // Write each requirement
    for req in requirements {
        let required_by = req
            .bom_sources
            .iter()
            .map(|source| {
                if source.quantity_needed == source.builds {
                    // If quantity needed equals builds, just show BOM name
                    source.bom_name.clone()
                } else {
                    // Otherwise show "BOM(qty)"
                    format!("{}({})", source.bom_name, source.quantity_needed)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        wtr.write_record(&[
            &req.part.name,
            &req.part.description,
            &req.part.stock.to_string(),
            &req.required.to_string(),
            &req.shortfall.to_string(),
            &required_by,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
