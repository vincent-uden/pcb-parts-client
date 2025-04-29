use anyhow::{Result, anyhow};
use common::{
    models::{Bom, BomWithParts, Part, PartWithStock},
    network::NetworkClient,
};
use iced::{Border, Length, Pixels, Theme, alignment, widget};
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
    part_searcher: PartSearch,
    bom_searcher: BomSearch,
    query: String,
    network: Arc<Mutex<NetworkClient>>,
}

#[derive(Debug)]
pub struct PartSearch {
    pub matching: Vec<PartWithStock>,
}

#[derive(Debug)]
pub struct BomSearch {
    pub matching: Vec<Bom>, // TODO: Fetch from network
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
            SearchMessage::OpenBom(bom) => todo!(),
            SearchMessage::Toggle => {
                self.mode = match self.mode {
                    SearchMode::Parts => SearchMode::Boms,
                    SearchMode::Boms => SearchMode::Parts,
                };
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, SearchMessage> {
        widget::container(
            widget::column!(
                widget::row!(
                    // TODO: Search icon
                    widget::text_input("Name or description", &self.query)
                        .on_input(SearchMessage::PendingQuery)
                        .on_submit(SearchMessage::SubmitQuery),
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
        .padding(8.0)
        .into()
    }
}

impl PartSearch {
    pub fn new() -> Self {
        Self { matching: vec![] }
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

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        let mut rows = vec![
            widget::row![
                widget::text("Name").width(Length::Fill),
                widget::text("Description").width(Length::Fill),
                widget::text("Stock").width(60.0),
                widget::text("").width(140.0),
            ]
            .spacing(16.0)
            .into(),
            widget::vertical_space().height(4.0).into(),
        ];
        rows.extend(self.matching.iter().map(|p| {
            widget::row![
                widget::text(&p.name).width(Length::Fill),
                widget::text(&p.description).width(Length::Fill),
                widget::text(&p.stock).width(60.0),
                widget::button("Change stock")
                    .width(140.0)
                    .on_press(SearchMessage::ChangeStock(p.clone())),
            ]
            .spacing(16.0)
            .into()
        }));
        widget::scrollable(widget::column(rows).spacing(8.0)).into()
    }
}

impl BomSearch {
    pub fn new() -> Self {
        Self { matching: vec![] }
    }
    async fn query(network: Arc<Mutex<NetworkClient>>, query: String) -> Result<Vec<Bom>> {
        let mut network = network.lock().await;
        let profile_id = match &network.user_data.profile {
            Some(p) => p.id,
            None => return Err(anyhow!("No profile selected")),
        };
        network.list_boms(profile_id, None, Some(query)).await
    }

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        let mut rows = vec![
            widget::row![
                widget::text("Name").width(Length::Fill),
                widget::text("Description").width(Length::Fill),
                widget::text("").width(140.0),
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
            .spacing(16.0)
            .into()
        }));
        widget::scrollable(widget::column(rows).spacing(8.0)).into()
    }
}
