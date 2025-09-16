use anyhow::{Result, anyhow};
use common::{
    models::{Bom, BomWithParts, Part, PartWithCountAndStock, PartWithStock},
    network::NetworkClient,
};
use iced::{
    Alignment, Border, Font, Length, Padding, Pixels, Theme, alignment, clipboard, font::Weight,
    futures::future::join_all, widget,
};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

use super::SearchMessage;

#[derive(Debug, Default)]
enum SearchMode {
    #[default]
    Parts,
    Boms,
}

#[derive(Debug)]
pub struct Search {
    mode: SearchMode,
    pub part_searcher: PartSearch,
    bom_searcher: BomSearch,
    query: String,
    network: Arc<Mutex<NetworkClient>>,
}

#[derive(Debug)]
pub struct PartSearch {
    pub matching: Vec<PartWithStock>,
    pub hovered_part: Option<i64>,
    pub selected_part: Option<PartWithStock>,
    pub stock_quantity: String,
    pub stock_row: String,
    pub stock_column: String,
    pub stock_z: String,
}

#[derive(Debug)]
pub struct BomSearch {
    pub matching: Vec<Bom>,
    pub expanded: Option<Bom>,
    pub parts: Vec<PartWithCountAndStock>,
    pub stock_quantity: String,
}

impl Search {
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            mode: SearchMode::default(),
            part_searcher: PartSearch::new(),
            bom_searcher: BomSearch::new(),
            query: String::new(),
            network,
        }
    }

    pub fn update(&mut self, message: SearchMessage) -> iced::Task<SearchMessage> {
        match message {
            SearchMessage::PendingQuery(s) => {
                self.query = s;
                iced::Task::none()
            }
            SearchMessage::SubmitQuery => match self.mode {
                SearchMode::Parts => iced::Task::perform(
                    PartSearch::query(self.network.clone(), self.query.clone()),
                    |output| match output {
                        Ok(output) => SearchMessage::PartSearchResult(output),
                        Err(e) => SearchMessage::FailedSearch(format!("{}", e)),
                    },
                ),
                SearchMode::Boms => iced::Task::perform(
                    BomSearch::query(self.network.clone(), self.query.clone()),
                    |output| match output {
                        Ok(output) => SearchMessage::BomSearchResult(output),
                        Err(e) => SearchMessage::FailedSearch(format!("{}", e)),
                    },
                ),
            },
            SearchMessage::PartSearchResult(vec) => {
                self.part_searcher.matching = vec;
                self.part_searcher.hovered_part = None;
                iced::Task::none()
            }
            SearchMessage::BomSearchResult(vec) => {
                self.bom_searcher.matching = vec;
                iced::Task::none()
            }
            SearchMessage::FailedSearch(msg) => {
                error!("Error searching {}", msg);
                iced::Task::none()
            }
            SearchMessage::ChangeStock(_) => {
                error!("ChangeStock should be consumed by parent");
                iced::Task::none()
            }
            SearchMessage::HoverPart(part) => {
                self.part_searcher.hovered_part = Some(part.id);
                iced::Task::none()
            }
            SearchMessage::ClearHover => {
                self.part_searcher.hovered_part = None;
                iced::Task::none()
            }
            SearchMessage::OpenBom(bom) => {
                self.bom_searcher.expanded = Some(bom.clone());
                self.bom_searcher.parts.clear();
                iced::Task::perform(
                    BomSearch::fetch_bom_parts(self.network.clone(), bom),
                    |output| match output {
                        Ok(output) => SearchMessage::BomPartsSearchResult(output),
                        Err(e) => SearchMessage::FailedSearch(format!("{}", e)),
                    },
                )
            }
            SearchMessage::RefreshBom(bom) => iced::Task::perform(
                BomSearch::fetch_bom_parts(self.network.clone(), bom),
                |output| match output {
                    Ok(output) => SearchMessage::BomPartsSearchResult(output),
                    Err(e) => SearchMessage::FailedSearch(format!("{}", e)),
                },
            ),
            SearchMessage::Toggle => {
                self.mode = match self.mode {
                    SearchMode::Parts => SearchMode::Boms,
                    SearchMode::Boms => SearchMode::Parts,
                };
                iced::Task::done(SearchMessage::SubmitQuery)
            }
            SearchMessage::BomPartsSearchResult(vec) => {
                self.bom_searcher.parts = vec;
                iced::Task::none()
            }
            SearchMessage::RestockBom(bom) => {
                let diff = match self.bom_searcher.stock_quantity.parse() {
                    Ok(x) => x,
                    Err(_) => return iced::Task::none(),
                };
                let old_parts = self.bom_searcher.parts.clone();
                for p in self.bom_searcher.parts.iter_mut() {
                    p.stock += diff * p.count;
                }
                iced::Task::perform(
                    BomSearch::change_bom_stock(self.network.clone(), bom, old_parts, diff),
                    move |output| match output {
                        Ok(_) => SearchMessage::StockChangeSuccess(diff),
                        Err(_) => SearchMessage::StockChangeFailed,
                    },
                )
            }
            SearchMessage::DepleteBom(bom) => {
                let diff: i64 = match self.bom_searcher.stock_quantity.parse() {
                    Ok(x) => x,
                    Err(_) => return iced::Task::none(),
                };
                let old_parts = self.bom_searcher.parts.clone();
                for p in self.bom_searcher.parts.iter_mut() {
                    p.stock -= diff * p.count;
                }
                iced::Task::perform(
                    BomSearch::change_bom_stock(self.network.clone(), bom, old_parts, -diff),
                    move |output| match output {
                        Ok(_) => SearchMessage::StockChangeSuccess(diff),
                        Err(_) => SearchMessage::StockChangeFailed,
                    },
                )
            }
            SearchMessage::StockQuantity(s) => {
                self.bom_searcher.stock_quantity = s;
                iced::Task::none()
            }
            SearchMessage::StockChangeFailed => iced::Task::none(),
            SearchMessage::StockChangeSuccess(_) => {
                if let Some(bom) = &self.bom_searcher.expanded {
                    let bom = bom.clone();
                    iced::Task::done(SearchMessage::RefreshBom(bom))
                } else if self.part_searcher.selected_part.is_some() {
                    // Clear the selected part and refresh the parts search
                    self.part_searcher.selected_part = None;
                    self.part_searcher.stock_quantity.clear();
                    self.part_searcher.stock_row.clear();
                    self.part_searcher.stock_column.clear();
                    self.part_searcher.stock_z.clear();
                    iced::Task::done(SearchMessage::SubmitQuery)
                        .chain(iced::Task::done(SearchMessage::EnableGridSelection(false)))
                } else {
                    iced::Task::none()
                }
            }
            SearchMessage::CloseBom => {
                self.bom_searcher.expanded = None;
                self.bom_searcher.parts.clear();
                self.bom_searcher.stock_quantity.clear();
                iced::Task::none()
            }
            SearchMessage::SelectPart(part) => {
                self.part_searcher.selected_part = Some(part.clone());
                self.part_searcher.stock_row = part.row.to_string();
                self.part_searcher.stock_column = part.column.to_string();
                self.part_searcher.stock_z = part.z.to_string();
                self.part_searcher.stock_quantity.clear();
                iced::Task::done(SearchMessage::EnableGridSelection(true))
                    .chain(iced::Task::done(SearchMessage::UpdateTargetBinHighlight))
            }
            SearchMessage::CancelPartStock => {
                self.part_searcher.selected_part = None;
                self.part_searcher.stock_quantity.clear();
                self.part_searcher.stock_row.clear();
                self.part_searcher.stock_column.clear();
                self.part_searcher.stock_z.clear();
                iced::Task::done(SearchMessage::EnableGridSelection(false))
                    .chain(iced::Task::done(SearchMessage::UpdateTargetBinHighlight))
            }
            SearchMessage::PartStockQuantity(s) => {
                self.part_searcher.stock_quantity = s;
                iced::Task::none()
            }
            SearchMessage::PartStockRow(s) => {
                self.part_searcher.stock_row = s;
                iced::Task::done(SearchMessage::UpdateTargetBinHighlight)
            }
            SearchMessage::PartStockColumn(s) => {
                self.part_searcher.stock_column = s;
                iced::Task::done(SearchMessage::UpdateTargetBinHighlight)
            }
            SearchMessage::PartStockZ(s) => {
                self.part_searcher.stock_z = s;
                iced::Task::done(SearchMessage::UpdateTargetBinHighlight)
            }
            SearchMessage::RestockPart => {
                if let Some(part) = &self.part_searcher.selected_part {
                    let diff: i64 = match self.part_searcher.stock_quantity.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let row: i64 = match self.part_searcher.stock_row.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let column: i64 = match self.part_searcher.stock_column.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let z: i64 = match self.part_searcher.stock_z.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };

                    iced::Task::perform(
                        PartSearch::change_part_stock(
                            self.network.clone(),
                            part.id,
                            part.stock,
                            diff,
                            row,
                            column,
                            z,
                        ),
                        move |output| match output {
                            Ok(_) => SearchMessage::StockChangeSuccess(diff),
                            Err(_) => SearchMessage::StockChangeFailed,
                        },
                    )
                } else {
                    iced::Task::none()
                }
            }
            SearchMessage::DepletePart => {
                if let Some(part) = &self.part_searcher.selected_part {
                    let diff: i64 = match self.part_searcher.stock_quantity.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let row: i64 = match self.part_searcher.stock_row.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let column: i64 = match self.part_searcher.stock_column.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };
                    let z: i64 = match self.part_searcher.stock_z.parse() {
                        Ok(x) => x,
                        Err(_) => return iced::Task::none(),
                    };

                    iced::Task::perform(
                        PartSearch::change_part_stock(
                            self.network.clone(),
                            part.id,
                            part.stock,
                            -diff,
                            row,
                            column,
                            z,
                        ),
                        move |output| match output {
                            Ok(_) => SearchMessage::StockChangeSuccess(diff),
                            Err(_) => SearchMessage::StockChangeFailed,
                        },
                    )
                } else {
                    iced::Task::none()
                }
            }
            SearchMessage::GridCellSelected(row, column) => {
                if self.part_searcher.selected_part.is_some() {
                    self.part_searcher.stock_row = row.to_string();
                    self.part_searcher.stock_column = column.to_string();
                    iced::Task::done(SearchMessage::UpdateTargetBinHighlight)
                } else {
                    iced::Task::none()
                }
            }
            SearchMessage::EnableGridSelection(_) => {
                // This message is handled by the app to enable/disable grid selection mode
                iced::Task::none()
            }
            SearchMessage::UpdateTargetBinHighlight => {
                // This message is handled by the app to update grid target bin highlighting
                iced::Task::none()
            }
            SearchMessage::CopyToClipboard(to_copy) => iced::clipboard::write(to_copy),
        }
    }

    pub fn view(&self, focused: bool) -> iced::Element<'_, SearchMessage> {
        let search_bar: iced::Element<'_, SearchMessage> = if focused {
            widget::text_input("Name or description", &self.query)
                .on_input(SearchMessage::PendingQuery)
                .on_submit(SearchMessage::SubmitQuery)
                .into()
        } else {
            widget::container(widget::text("Name or description").style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::text::Style {
                    color: palette.secondary.strong.color.into(),
                }
            }))
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::container::Style {
                    background: Some(palette.background.base.color.into()),
                    border: Border {
                        color: palette.secondary.strong.color.into(),
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    ..Default::default()
                }
            })
            .padding(4.0)
            .width(Length::Fill)
            .into()
        };
        widget::container(
            widget::column!(
                widget::row!(
                    // TODO: Search icon
                    search_bar,
                    widget::horizontal_space().width(16.0),
                    widget::text("Parts"),
                    widget::horizontal_space().width(8.0),
                    widget::toggler(match self.mode {
                        SearchMode::Parts => false,
                        SearchMode::Boms => true,
                    })
                    .on_toggle(|_| SearchMessage::Toggle)
                    .style(|theme: &Theme, _status| {
                        let palette = theme.extended_palette();
                        let mut style = widget::toggler::default(theme, _status);
                        style.background = palette.background.base.color;
                        style.foreground = palette.primary.base.color;
                        style
                    }),
                    widget::text("BOMs"),
                )
                .height(Length::Shrink)
                .align_y(alignment::Vertical::Center),
                widget::horizontal_rule(4.0),
                match self.mode {
                    SearchMode::Parts => self.part_searcher.view(),
                    SearchMode::Boms => self.bom_searcher.view(),
                },
            )
            .spacing(8.0),
        )
        .height(Length::Fill)
        .style(|theme| {
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
}

impl PartSearch {
    pub fn new() -> Self {
        Self {
            matching: vec![],
            hovered_part: None,
            selected_part: None,
            stock_quantity: String::new(),
            stock_row: String::new(),
            stock_column: String::new(),
            stock_z: String::new(),
        }
    }
    async fn query(
        network: Arc<Mutex<NetworkClient>>,
        query: String,
    ) -> Result<Vec<PartWithStock>> {
        let mut network = network.lock().await;
        let out = network
            .parts_with_stock(Some(query.clone()), Some(query), 1)
            .await?;
        Ok(out)
    }

    async fn change_part_stock(
        network: Arc<Mutex<NetworkClient>>,
        part_id: i64,
        current_stock: i64,
        diff: i64,
        row: i64,
        column: i64,
        z: i64,
    ) -> Result<()> {
        let mut network = network.lock().await;
        let profile_id = match &network.user_data.profile {
            Some(p) => p.id,
            None => return Err(anyhow!("No profile selected")),
        };
        let new_stock = current_stock + diff;
        network
            .stock_part(profile_id, part_id, new_stock, column, row, z)
            .await?;
        Ok(())
    }

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        if let Some(selected_part) = &self.selected_part {
            widget::column![
                widget::row![
                    widget::column![
                        widget::text(&selected_part.name).size(18.0).font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        }),
                        widget::text(&selected_part.description).size(14.0),
                        widget::text(format!("Current stock: {}", selected_part.stock)).size(12.0),
                    ]
                    .width(Length::Fill),
                    widget::button("Cancel").on_press(SearchMessage::CancelPartStock),
                ]
                .spacing(16.0)
                .align_y(Alignment::Center),
                widget::horizontal_rule(2.0),
                widget::column![
                    widget::text("Quantity:"),
                    widget::text_input("Enter amount", &self.stock_quantity)
                        .on_input(SearchMessage::PartStockQuantity),
                    widget::text("Bin Location:"),
                    widget::text(
                        "Click on the grid to select a bin location, or enter coordinates manually:"
                    )
                    .size(12.0)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        widget::text::Style {
                            color: palette.background.weak.text.into(),
                        }
                    }),
                    widget::row![
                        widget::column![
                            widget::text("Row:"),
                            widget::text_input("", &self.stock_row)
                                .on_input(SearchMessage::PartStockRow)
                                .width(80.0),
                        ],
                        widget::column![
                            widget::text("Column:"),
                            widget::text_input("", &self.stock_column)
                                .on_input(SearchMessage::PartStockColumn)
                                .width(80.0),
                        ],
                        widget::column![
                            widget::text("Z:"),
                            widget::text_input("", &self.stock_z)
                                .on_input(SearchMessage::PartStockZ)
                                .width(80.0),
                        ],
                    ]
                    .spacing(16.0),
                    widget::row![
                        widget::button("Restock")
                            .style(|theme: &Theme, _status| {
                                let palette = theme.extended_palette();
                                widget::button::Style {
                                    background: Some(palette.success.base.color.into()),
                                    text_color: palette.success.base.text,
                                    ..widget::button::primary(theme, _status)
                                }
                            })
                            .on_press(SearchMessage::RestockPart),
                        widget::button("Deplete")
                            .style(|theme: &Theme, _status| {
                                let palette = theme.extended_palette();
                                widget::button::Style {
                                    background: Some(palette.danger.base.color.into()),
                                    text_color: palette.danger.base.text,
                                    ..widget::button::primary(theme, _status)
                                }
                            })
                            .on_press(SearchMessage::DepletePart),
                    ]
                    .spacing(16.0),
                ]
                .spacing(8.0),
            ]
            .spacing(16.0)
            .into()
        } else {
            let mut rows = vec![
                widget::row![
                    table_header("Name").width(Length::Fill),
                    table_header("Description").width(Length::Fill),
                    table_header("Stock").width(60.0).align_x(Alignment::End),
                    table_header("").width(140.0),
                ]
                .spacing(16.0)
                .into(),
            ];
            rows.extend(self.matching.iter().map(|p| {
                let is_hovered = self.hovered_part == Some(p.id);
                widget::mouse_area(
                    widget::container(
                        widget::row![
                            widget::text(&p.name).width(Length::Fill),
                            widget::text(&p.description).width(Length::Fill),
                            widget::text(&p.stock).width(60.0).align_x(Alignment::End),
                            widget::button("Change stock")
                                .width(140.0)
                                .on_press(SearchMessage::SelectPart(p.clone())),
                        ]
                        .spacing(16.0)
                        .align_y(Alignment::Center),
                    )
                    .padding(4.0)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        if is_hovered {
                            widget::container::Style {
                                background: Some(palette.background.strong.color.into()),
                                border: Border {
                                    color: palette.background.base.text,
                                    width: 1.0,
                                    radius: iced::border::Radius::from(4.0),
                                },
                                ..Default::default()
                            }
                        } else {
                            widget::container::Style::default()
                        }
                    }),
                )
                .on_enter(SearchMessage::HoverPart(p.clone()))
                .on_exit(SearchMessage::ClearHover)
                .into()
            }));
            widget::scrollable(widget::column(rows).spacing(8.0)).into()
        }
    }
}

impl BomSearch {
    pub fn new() -> Self {
        Self {
            matching: vec![],
            expanded: None,
            parts: vec![],
            stock_quantity: String::new(),
        }
    }
    async fn query(network: Arc<Mutex<NetworkClient>>, query: String) -> Result<Vec<Bom>> {
        let mut network = network.lock().await;
        let profile_id = match &network.user_data.profile {
            Some(p) => p.id,
            None => return Err(anyhow!("No profile selected")),
        };
        network.list_boms(profile_id, None, Some(query)).await
    }

    async fn fetch_bom_parts(
        network: Arc<Mutex<NetworkClient>>,
        bom: Bom,
    ) -> Result<Vec<PartWithCountAndStock>> {
        let mut network = network.lock().await;
        let profile_id = match &network.user_data.profile {
            Some(p) => p.id,
            None => return Err(anyhow!("No profile selected")),
        };
        network.parts_in_bom(profile_id, bom.id).await
    }

    async fn change_bom_stock(
        network: Arc<Mutex<NetworkClient>>,
        bom: Bom,
        parts: Vec<PartWithCountAndStock>,
        diff: i64,
    ) -> Result<()> {
        let mut network = network.lock().await;
        let profile_id = network.user_data.profile.as_ref().unwrap().id;
        network.stock_parts(profile_id, &parts, diff).await
    }

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        match &self.expanded {
            Some(bom) => self.view_bom_contents(bom),
            None => self.view_list(),
        }
    }

    fn view_list(&self) -> iced::Element<'_, SearchMessage> {
        let mut rows = vec![
            widget::row![
                table_header("Name").width(Length::Fill),
                table_header("Description").width(Length::Fill),
                table_header("").width(140.0),
            ]
            .spacing(16.0)
            .into(),
            widget::vertical_space().height(4.0).into(),
        ];
        rows.extend(self.matching.iter().map(|p| {
            widget::row![
                widget::text(&p.name).width(Length::Fill),
                widget::text(&p.description).width(Length::Fill),
                widget::button("Open")
                    .width(140.0)
                    .on_press(SearchMessage::OpenBom(p.clone())),
            ]
            .align_y(Alignment::Center)
            .spacing(16.0)
            .into()
        }));
        widget::scrollable(widget::column(rows).spacing(8.0)).into()
    }

    fn view_bom_contents(&self, bom: &Bom) -> iced::Element<'_, SearchMessage> {
        let mut rows = vec![
            widget::row![
                widget::text(format!("{}", bom.name))
                    .width(Length::Fill)
                    .size(36.0),
                widget::text_input("", &self.stock_quantity)
                    .width(60.0)
                    .on_input(SearchMessage::StockQuantity),
                widget::button("Restock")
                    .width(80.0)
                    .on_press(SearchMessage::RestockBom(self.expanded.clone().unwrap())),
                widget::button("Deplete")
                    .width(80.0)
                    .on_press(SearchMessage::DepleteBom(self.expanded.clone().unwrap())),
            ]
            .spacing(4.0)
            .align_y(Alignment::Center)
            .into(),
            widget::vertical_space().height(12.0).into(),
            widget::text(format!("{}", bom.description)).into(),
            widget::vertical_space().height(8.0).into(),
            widget::horizontal_rule(8.0).into(),
            widget::vertical_space().height(8.0).into(),
            widget::row![
                table_header("Name").width(Length::Fill),
                table_header("Description").width(Length::Fill),
                table_header("Count").width(60.0).align_x(Alignment::End),
                table_header("Stock").width(60.0).align_x(Alignment::End),
            ]
            .spacing(16.0)
            .padding(Padding::default().right(16.0))
            .into(),
        ];

        let mut parts = widget::column(vec![]);
        parts = parts.extend(self.parts.iter().map(|p| {
            widget::row![
                widget::button(widget::text(&p.name))
                    .width(Length::Fill)
                    .on_press(SearchMessage::CopyToClipboard(p.name.clone()))
                    .style(|theme: &Theme, status| {
                        let palette = theme.extended_palette();
                        widget::button::Style {
                            background: None,
                            text_color: palette.background.base.text,
                            ..Default::default()
                        }
                    }),
                widget::text(&p.description).width(Length::Fill),
                widget::text(&p.count).width(60.0).align_x(Alignment::End),
                widget::text(&p.stock).width(60.0).align_x(Alignment::End),
            ]
            .align_y(Alignment::Center)
            .spacing(16.0)
            .padding(Padding::default().right(16.0))
            .into()
        }));

        rows.push(widget::scrollable(parts).height(Length::Fill).into());
        rows.push(
            widget::container(widget::button("Close").on_press(SearchMessage::CloseBom))
                .center_x(Length::Fill)
                .into(),
        );

        widget::column(rows).into()
    }
}

pub fn table_header(label: &str) -> widget::Text {
    let mut bold = Font::DEFAULT;
    bold.weight = Weight::Bold;
    widget::text(label).font(bold).style(|theme: &Theme| {
        let palette = theme.extended_palette();
        widget::text::Style {
            color: palette.primary.strong.color.into(),
        }
    })
}
