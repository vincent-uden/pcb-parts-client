use anyhow::Result;
use common::{models::Part, network::NetworkClient};
use iced::widget;
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
        }
    }

    pub fn update(&mut self, message: SearchMessage) -> iced::Task<SearchMessage> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, SearchMessage> {
        match self.mode {
            SearchMode::Parts => self.part_searcher.view(),
            SearchMode::Boms => self.bom_searcher.view(),
        }
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
