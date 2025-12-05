use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use common::{
    export::export_purchase_plan_to_csv,
    models::{Bom, BomSource, PartWithStock, PurchaseRequirement},
    network::NetworkClient,
};
use iced::{Alignment, Border, Length, Padding, Theme, widget};
use tokio::sync::Mutex;
use tracing::error;

use super::{Msg, SelectedBom};

#[derive(Debug)]
pub struct PurchasePlanner {
    network: Arc<Mutex<NetworkClient>>,
    bom_search_query: String,
    bom_search_results: Vec<Bom>,
    selected_boms: Vec<SelectedBom>,
    purchase_requirements: Vec<PurchaseRequirement>,
    export_path: String,
}

impl PurchasePlanner {
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            network,
            bom_search_query: String::new(),
            bom_search_results: vec![],
            selected_boms: vec![],
            purchase_requirements: vec![],
            export_path: String::from("./purchase_plan.csv"),
        }
    }

    pub fn update(&mut self, msg: Msg) -> iced::Task<Msg> {
        match msg {
            Msg::SearchBom(query) => {
                self.bom_search_query = query;
                iced::Task::none()
            }
            Msg::SubmitSearch => {
                let network = self.network.clone();
                let query = if self.bom_search_query.is_empty() {
                    None
                } else {
                    Some(self.bom_search_query.clone())
                };

                iced::Task::perform(
                    Self::fetch_boms(network, query),
                    |result| match result {
                        Ok(boms) => Msg::SearchResults(boms),
                        Err(e) => Msg::SearchFailed(e.to_string()),
                    },
                )
            }
            Msg::SearchResults(boms) => {
                self.bom_search_results = boms;
                iced::Task::none()
            }
            Msg::SearchFailed(e) => {
                error!("BOM search failed: {}", e);
                iced::Task::none()
            }
            Msg::SelectBom(bom) => {
                // Check if BOM is already selected
                if !self.selected_boms.iter().any(|sb| sb.bom.id == bom.id) {
                    self.selected_boms.push(SelectedBom { bom, quantity: 1 });
                    // Clear search results after selection
                    self.bom_search_results.clear();
                    self.bom_search_query.clear();
                    // Trigger recalculation
                    iced::Task::done(Msg::CalculatePlan)
                } else {
                    iced::Task::none()
                }
            }
            Msg::RemoveBom(bom_id) => {
                self.selected_boms.retain(|sb| sb.bom.id != bom_id);
                // Trigger recalculation
                if self.selected_boms.is_empty() {
                    // Clear requirements if no BOMs selected
                    self.purchase_requirements.clear();
                    iced::Task::none()
                } else {
                    iced::Task::done(Msg::CalculatePlan)
                }
            }
            Msg::UpdateQuantity(bom_id, quantity_str) => {
                if let Some(selected_bom) = self.selected_boms.iter_mut().find(|sb| sb.bom.id == bom_id) {
                    if let Ok(quantity) = quantity_str.parse::<i64>() {
                        if quantity > 0 {
                            selected_bom.quantity = quantity;
                            // Trigger recalculation
                            return iced::Task::done(Msg::CalculatePlan);
                        }
                    }
                }
                iced::Task::none()
            }
            Msg::CalculatePlan => {
                let network = self.network.clone();
                let selected_boms = self.selected_boms.clone();

                iced::Task::perform(
                    Self::calculate_purchase_plan(network, selected_boms),
                    |result| match result {
                        Ok(requirements) => Msg::PlanCalculated(requirements),
                        Err(e) => Msg::PlanFailed(e.to_string()),
                    },
                )
            }
            Msg::PlanCalculated(mut requirements) => {
                // Sort alphabetically by part name
                requirements.sort_by(|a, b| a.part.name.cmp(&b.part.name));
                self.purchase_requirements = requirements;
                iced::Task::none()
            }
            Msg::PlanFailed(e) => {
                error!("Purchase plan calculation failed: {}", e);
                iced::Task::none()
            }
            Msg::HoverPart(_) => {
                // Handled in app.rs for grid integration
                iced::Task::none()
            }
            Msg::ClearHover => {
                // Handled in app.rs for grid integration
                iced::Task::none()
            }
            Msg::ExportPath(path) => {
                self.export_path = path;
                iced::Task::none()
            }
            Msg::ExportCsv => {
                let path = PathBuf::from(&self.export_path);
                let requirements = self.purchase_requirements.clone();

                match export_purchase_plan_to_csv(&path, &requirements) {
                    Ok(_) => iced::Task::done(Msg::ExportSuccess),
                    Err(e) => iced::Task::done(Msg::ExportFailed(e.to_string())),
                }
            }
            Msg::ExportSuccess => {
                // Could show a success message in the future
                iced::Task::none()
            }
            Msg::ExportFailed(e) => {
                error!("Export failed: {}", e);
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Msg> {
        let mut content = widget::column![
            widget::text("Purchase Planner").size(36.0),
            widget::vertical_space().height(16.0),
        ]
        .spacing(8.0);

        // BOM Search Section
        content = content.push(widget::text("BOM Selection").size(20.0));
        content = content.push(
            widget::text_input("Search BOMs...", &self.bom_search_query)
                .on_input(Msg::SearchBom)
                .on_submit(Msg::SubmitSearch)
                .width(Length::Fill),
        );

        // Search Results (only show if we have results)
        if !self.bom_search_query.is_empty() && !self.bom_search_results.is_empty() {
            content = content.push(widget::text("Search Results:").size(14.0));

            let buttons: Vec<_> = self.bom_search_results
                .iter()
                .map(|bom| {
                    widget::button(widget::text(&bom.name))
                        .on_press(Msg::SelectBom(bom.clone()))
                        .into()
                })
                .collect();
            
            content = content.push(widget::row(buttons).spacing(8.0).wrap());
        }

        content = content.push(widget::vertical_space().height(8.0));
        content = content.push(widget::horizontal_rule(2.0));
        content = content.push(widget::vertical_space().height(8.0));

        // Selected BOMs Section (only show if we have selected BOMs)
        if !self.selected_boms.is_empty() {
            content = content.push(widget::text("Selected BOMs:").size(20.0));
            content = content.push(
                widget::row![
                    widget::text("Name").width(Length::FillPortion(3)),
                    widget::text("Description").width(Length::FillPortion(3)),
                    widget::text("Quantity").width(Length::Fixed(100.0)),
                    widget::text("").width(Length::Fixed(80.0)), // Remove button column
                ]
                .spacing(8.0)
                .padding(Padding::default().bottom(4.0)),
            );

            for selected_bom in &self.selected_boms {
                content = content.push(
                    widget::row![
                        widget::text(&selected_bom.bom.name).width(Length::FillPortion(3)),
                        widget::text(&selected_bom.bom.description).width(Length::FillPortion(3)),
                        widget::text_input("", &selected_bom.quantity.to_string())
                            .on_input(move |s| Msg::UpdateQuantity(selected_bom.bom.id, s))
                            .width(Length::Fixed(100.0)),
                        widget::button("Remove")
                            .on_press(Msg::RemoveBom(selected_bom.bom.id))
                            .width(Length::Fixed(80.0)),
                    ]
                    .spacing(8.0)
                    .align_y(Alignment::Center),
                );
            }

            content = content.push(widget::vertical_space().height(8.0));
            content = content.push(widget::horizontal_rule(2.0));
            content = content.push(widget::vertical_space().height(8.0));
        }

        // Purchase Requirements Section (only show if we have requirements)
        if !self.purchase_requirements.is_empty() {
            content = content.push(widget::text("Purchase Requirements:").size(20.0));

            // Table header
            content = content.push(
                widget::row![
                    widget::text("Part Name").width(Length::FillPortion(2)),
                    widget::text("Description").width(Length::FillPortion(2)),
                    widget::text("Stock").width(Length::Fixed(80.0)).align_x(Alignment::End),
                    widget::text("Required").width(Length::Fixed(80.0)).align_x(Alignment::End),
                    widget::text("Purchase").width(Length::Fixed(80.0)).align_x(Alignment::End),
                    widget::text("Required By").width(Length::FillPortion(2)),
                ]
                .spacing(8.0)
                .padding(Padding::default().bottom(4.0)),
            );

            // Table rows
            let mut rows = widget::column![].spacing(4.0);
            for req in &self.purchase_requirements {
                let required_by = req
                    .bom_sources
                    .iter()
                    .map(|source| {
                        if source.quantity_needed == source.builds {
                            source.bom_name.clone()
                        } else {
                            format!("{}({})", source.bom_name, source.quantity_needed)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                // Determine color based on stock status
                let stock_color = if req.part.stock >= req.required {
                    // Green: sufficient stock
                    |theme: &Theme| {
                        let palette = theme.extended_palette();
                        widget::text::Style {
                            color: Some(palette.success.base.color),
                        }
                    }
                } else if req.part.stock > 0 {
                    // Yellow: partial stock
                    |theme: &Theme| {
                        let palette = theme.extended_palette();
                        widget::text::Style {
                            color: Some(palette.warning.base.color),
                        }
                    }
                } else {
                    // Red: no stock
                    |theme: &Theme| {
                        let palette = theme.extended_palette();
                        widget::text::Style {
                            color: Some(palette.danger.base.color),
                        }
                    }
                };

                let row = widget::mouse_area(
                    widget::row![
                        widget::text(&req.part.name).width(Length::FillPortion(2)),
                        widget::text(&req.part.description).width(Length::FillPortion(2)),
                        widget::text(req.part.stock.to_string())
                            .width(Length::Fixed(80.0))
                            .align_x(Alignment::End)
                            .style(stock_color),
                        widget::text(req.required.to_string())
                            .width(Length::Fixed(80.0))
                            .align_x(Alignment::End),
                        widget::text(req.shortfall.to_string())
                            .width(Length::Fixed(80.0))
                            .align_x(Alignment::End)
                            .style(stock_color),
                        widget::text(required_by).width(Length::FillPortion(2)),
                    ]
                    .spacing(8.0)
                    .align_y(Alignment::Center),
                )
                .on_enter(Msg::HoverPart(req.part.clone()))
                .on_exit(Msg::ClearHover);

                rows = rows.push(row);
            }

            content = content.push(widget::scrollable(rows).height(Length::Fill));

            content = content.push(widget::vertical_space().height(8.0));
            content = content.push(widget::horizontal_rule(2.0));
            content = content.push(widget::vertical_space().height(8.0));

            // Export Section
            content = content.push(
                widget::row![
                    widget::text("Export Path:"),
                    widget::text_input("", &self.export_path)
                        .on_input(Msg::ExportPath)
                        .width(Length::Fill),
                    widget::button("Export to CSV").on_press(Msg::ExportCsv),
                ]
                .spacing(8.0)
                .align_y(Alignment::Center),
            );
        }

        widget::container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::container::Style {
                    text_color: Some(palette.background.weak.text),
                    background: Some(palette.background.weak.color.into()),
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                }
            })
            .padding(16.0)
            .into()
    }

    async fn fetch_boms(network: Arc<Mutex<NetworkClient>>, query: Option<String>) -> Result<Vec<Bom>> {
        let mut n = network.lock().await;
        let profile_id = n
            .user_data
            .profile
            .as_ref()
            .ok_or(anyhow!("No profile selected"))?
            .id;

        n.list_boms(profile_id, None, query).await
    }

    async fn calculate_purchase_plan(
        network: Arc<Mutex<NetworkClient>>,
        selected_boms: Vec<SelectedBom>,
    ) -> Result<Vec<PurchaseRequirement>> {
        let mut n = network.lock().await;
        let profile_id = n
            .user_data
            .profile
            .as_ref()
            .ok_or(anyhow!("No profile selected"))?
            .id;

        // Step 1: Aggregate parts from all selected BOMs
        // Store (required_count, stock_info, bom_sources)
        let mut part_map: HashMap<i64, (i64, (String, String, i64, i64, i64, i64), Vec<BomSource>)> = HashMap::new();

        for selected_bom in &selected_boms {
            let parts = n.parts_in_bom(profile_id, selected_bom.bom.id).await?;

            for part in parts {
                let total_needed = part.count * selected_bom.quantity;
                let part_info = (
                    part.name.clone(),
                    part.description.clone(),
                    part.stock,
                    part.column,
                    part.row,
                    part.z,
                );

                part_map
                    .entry(part.id)
                    .and_modify(|(total, _info, sources)| {
                        *total += total_needed;
                        sources.push(BomSource {
                            bom_name: selected_bom.bom.name.clone(),
                            bom_id: selected_bom.bom.id,
                            quantity_needed: total_needed,
                            builds: selected_bom.quantity,
                        });
                    })
                    .or_insert_with(|| {
                        (
                            total_needed,
                            part_info,
                            vec![BomSource {
                                bom_name: selected_bom.bom.name.clone(),
                                bom_id: selected_bom.bom.id,
                                quantity_needed: total_needed,
                                builds: selected_bom.quantity,
                            }],
                        )
                    });
            }
        }

        // Step 2: Build purchase requirements
        let mut requirements = vec![];

        for (part_id, (required, (name, description, stock, column, row, z), bom_sources)) in part_map {
            let shortfall = (required - stock).max(0);

            requirements.push(PurchaseRequirement {
                part: common::models::PartWithStock {
                    id: part_id,
                    name,
                    description,
                    stock,
                    column,
                    row,
                    z,
                },
                required,
                shortfall,
                bom_sources,
            });
        }

        Ok(requirements)
    }
}
