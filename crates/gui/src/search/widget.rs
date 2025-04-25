use anyhow::Result;
use common::{models::Part, network::NetworkClient};
use iced::{Border, Length, alignment, widget};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

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
}

#[derive(Debug)]
pub struct PartSearch {
    matching: Vec<Part>,
    network: Arc<Mutex<NetworkClient>>,
}

#[derive(Debug)]
pub struct BomSearch {
    matching: Vec<Part>, // TODO: Change this datatype
    network: Arc<Mutex<NetworkClient>>,
}

impl Search {
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            mode: SearchMode::default(),
            part_searcher: PartSearch::new(network.clone()),
            bom_searcher: BomSearch::new(network.clone()),
            query: String::new(),
        }
    }

    pub fn update(&mut self, message: SearchMessage) -> iced::Task<SearchMessage> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, SearchMessage> {
        widget::container(
            widget::column!(
                widget::row!(
                    // TODO: Search icon
                    widget::text_input("Name or description", &self.query),
                    widget::horizontal_space().width(16.0),
                    widget::text("Parts"),
                    widget::horizontal_space().width(8.0),
                    widget::toggler(false),
                    widget::text("BOMs"),
                )
                .height(Length::Shrink)
                .align_y(alignment::Vertical::Center),
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
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            matching: vec![],
            network,
        }
    }
    async fn query(&mut self, query: &str) -> Result<()> {
        todo!()
    }

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        widget::text("Part search").into()
    }
}

impl BomSearch {
    pub fn new(network: Arc<Mutex<NetworkClient>>) -> Self {
        Self {
            matching: vec![],
            network,
        }
    }
    async fn query(&mut self, query: &str) -> Result<()> {
        todo!()
    }

    fn view(&self) -> iced::Element<'_, SearchMessage> {
        widget::text("BOM search").into()
    }
}
