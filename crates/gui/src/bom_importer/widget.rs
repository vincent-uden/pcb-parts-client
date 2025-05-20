use std::{path::PathBuf, str::FromStr};

use common::import::csv_to_headers;
use iced::{Border, Length, Theme, widget};

use super::{Msg, PartCandidate};

#[derive(Debug, Clone)]
pub struct PendingBom {
    name: String,
    candidates: Vec<PartCandidate>,
}

// TODO: File picker for importing
#[derive(Debug, Clone, Default)]
pub struct BomImporter {
    path: String,
    pending: Option<PendingBom>,
    column_names: Vec<String>,
    name_column: Option<String>,
    description_column: Option<String>,
    count_column: Option<String>,
}

impl BomImporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: Msg) -> iced::Task<Msg> {
        match msg {
            Msg::OpenFile => {
                match csv_to_headers(&PathBuf::from_str(&self.path).unwrap_or_default()) {
                    Ok(headers) => iced::Task::done(Msg::OpenSuccess(headers)),
                    Err(_) => iced::Task::done(Msg::OpenFailed),
                }
            }
            Msg::OpenSuccess(column_names) => {
                self.column_names = column_names;
                iced::Task::none()
            }
            Msg::OpenFailed => todo!(),
            Msg::BomName(s) => {
                if let Some(pending) = &mut self.pending {
                    pending.name = s;
                }
                iced::Task::none()
            }
            Msg::SelectNameColumn(s) => {
                self.name_column = Some(s);
                iced::Task::none()
            }
            Msg::SelectDescriptionColumn(s) => {
                self.description_column = Some(s);
                iced::Task::none()
            }
            Msg::SelectCountColumn(s) => {
                self.count_column = Some(s);
                iced::Task::none()
            }
            Msg::PendingPath(s) => {
                self.path = s;
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Msg> {
        let mut bom_view = widget::Column::new();
        if let Some(pending) = &self.pending {
            bom_view = bom_view.extend(vec![
                widget::text("Name").into(),
                widget::text_input("", &pending.name)
                    .on_input(Msg::BomName)
                    .into(),
            ]);
        }
        let mut column_pickers = widget::Column::new().spacing(4.0);
        if !self.path.is_empty() && !self.column_names.is_empty() {
            column_pickers = column_pickers.push(
                widget::row![
                    widget::text("Name Column").width(Length::Fill),
                    widget::text("Description Column").width(Length::Fill),
                    widget::text("Count Column").width(Length::Fill),
                ]
                .spacing(4.0),
            );
            column_pickers = column_pickers.push(
                widget::row![
                    widget::pick_list(
                        self.column_names.clone(),
                        self.name_column.clone(),
                        Msg::SelectNameColumn,
                    )
                    .width(Length::Fill),
                    widget::pick_list(
                        self.column_names.clone(),
                        self.description_column.clone(),
                        Msg::SelectDescriptionColumn,
                    )
                    .width(Length::Fill),
                    widget::pick_list(
                        self.column_names.clone(),
                        self.count_column.clone(),
                        Msg::SelectCountColumn,
                    )
                    .width(Length::Fill),
                ]
                .spacing(4.0),
            );
        }

        widget::container(widget::column![
            widget::text("BOM Importer"),
            widget::row![
                widget::text_input("Path", &self.path)
                    .on_input(Msg::PendingPath)
                    .on_submit(Msg::OpenFile)
            ],
            widget::vertical_space().height(8.0),
            column_pickers,
            bom_view,
        ])
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            widget::container::Style {
                text_color: Some(palette.background.weak.text),
                background: Some(palette.background.weak.color.into()),
                border: Border::default().rounded(8.0),
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .padding(8.0)
        .into()
    }
}
