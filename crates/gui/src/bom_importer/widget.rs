use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Result;
use common::{
    import::{csv_to_bom, csv_to_headers},
    network::NetworkClient,
};
use iced::{Alignment, Border, Length, Theme, widget};
use tokio::sync::Mutex;
use tracing::debug;

use crate::search::widget::table_header;

use super::{Msg, PartCandidate, PendingBom};

// TODO: File picker for importing
#[derive(Debug, Clone)]
pub struct BomImporter {
    path: String,
    pending: Option<PendingBom>,
    bom_name: String,
    bom_description: String,
    column_names: Vec<String>,
    name_column: Option<String>,
    description_column: Option<String>,
    count_column: Option<String>,
    network: Arc<Mutex<NetworkClient>>,
}

impl BomImporter {
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            path: String::new(),
            bom_name: String::new(),
            bom_description: String::new(),
            pending: None,
            column_names: vec![],
            name_column: None,
            description_column: None,
            count_column: None,
            network,
        }
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
                self.bom_name = s;
                iced::Task::none()
            }
            Msg::BomDescription(s) => {
                self.bom_description = s;
                iced::Task::none()
            }
            Msg::SelectNameColumn(s) => {
                self.name_column = Some(s);
                iced::Task::done(Msg::TryLoadPending)
            }
            Msg::SelectDescriptionColumn(s) => {
                self.description_column = Some(s);
                iced::Task::done(Msg::TryLoadPending)
            }
            Msg::SelectCountColumn(s) => {
                self.count_column = Some(s);
                iced::Task::done(Msg::TryLoadPending)
            }
            Msg::PendingPath(s) => {
                self.path = s;
                iced::Task::none()
            }
            Msg::TryLoadPending => {
                if let (Some(name), Some(desc), Some(count)) = (
                    &self.name_column,
                    &self.description_column,
                    &self.count_column,
                ) {
                    match csv_to_bom(
                        &PathBuf::from_str(&self.path).unwrap_or_default(),
                        name,
                        desc,
                        count,
                    ) {
                        Ok(parts) => iced::Task::perform(
                            Self::fetch_pending_bom(self.network.clone(), parts),
                            |output| match output {
                                Ok(pending) => Msg::PendingFetched(pending),
                                Err(_) => Msg::PendingFailed,
                            },
                        ),
                        Err(_) => iced::Task::none(),
                    }
                } else {
                    iced::Task::none()
                }
            }
            Msg::PendingFetched(pending_bom) => {
                self.pending = Some(pending_bom);
                iced::Task::none()
            }
            Msg::PendingFailed => todo!(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Msg> {
        let mut bom_view = widget::Column::new();
        bom_view = bom_view.extend(vec![
            widget::text("Name").into(),
            widget::text_input("", &self.bom_name)
                .on_input(Msg::BomName)
                .into(),
            widget::text("Description").into(),
            widget::text_input("", &self.bom_description)
                .on_input(Msg::BomDescription)
                .into(),
            self.view_bom_contents(),
        ]);
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
            widget::text("BOM Importer").size(36.0),
            widget::vertical_space().height(8.0),
            widget::row![
                widget::text_input("Path", &self.path)
                    .on_input(Msg::PendingPath)
                    .on_submit(Msg::OpenFile)
            ],
            widget::vertical_space().height(8.0),
            column_pickers,
            widget::vertical_space().height(8.0),
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
        .padding(16.0)
        .into()
    }

    fn view_bom_contents(&self) -> iced::Element<'_, Msg> {
        if let Some(pending) = &self.pending {
            let mut rows = vec![
                widget::vertical_space().height(12.0).into(),
                widget::horizontal_rule(2.0).into(),
                widget::vertical_space().height(4.0).into(),
                widget::row![
                    table_header("Name").width(Length::Fill),
                    table_header("Description").width(Length::Fill),
                    table_header("Count").width(60.0).align_x(Alignment::End),
                    table_header("Linked").width(60.0).align_x(Alignment::End),
                ]
                .spacing(16.0)
                .into(),
            ];

            let mut parts = widget::column(vec![]);
            parts = parts.extend(pending.candidates.iter().map(|p| {
                widget::row![
                    widget::text(&p.name).width(Length::Fill),
                    widget::text(&p.description).width(Length::Fill),
                    widget::text(&p.count).width(60.0).align_x(Alignment::End),
                    widget::text(if p.linked_part.is_some() { "Yes" } else { "No" }).width(60.0).align_x(Alignment::End),
                ]
                .align_y(Alignment::Center)
                .spacing(16.0)
                .into()
            }));

            rows.push(widget::scrollable(parts).height(Length::Fill).into());
            rows.push(
                widget::container(widget::button("Add"))
                    .center_x(Length::Fill)
                    .into(),
            );

            widget::column(rows).into()
        } else {
            widget::vertical_space().into()
        }
    }

    async fn fetch_pending_bom(
        network: Arc<Mutex<NetworkClient>>,
        parts: Vec<(i64, common::models::Part)>,
    ) -> Result<PendingBom> {
        let mut out = PendingBom { candidates: vec![] };
        let mut n = network.lock().await;

        for (count, p) in parts {
            let linked = n.get_parts(Some(p.name.clone()), None).await?;
            out.candidates.push(PartCandidate {
                name: p.name,
                description: p.description,
                count,
                linked_part: linked.first().cloned(),
            });
        }

        Ok(out)
    }
}
