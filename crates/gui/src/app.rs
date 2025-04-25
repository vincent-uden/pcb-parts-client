use std::sync::Arc;
use tokio::sync::Mutex;

use common::{models::Part, network::NetworkClient};
use iced::{Length, Subscription, event::listen_with, widget};

use crate::{
    CONFIG,
    search::{SearchMessage, widget::Search},
};

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Tells the grid to highlight some parts
    HighlightParts(Vec<Part>),
    SearchMessage(SearchMessage),
    Quit,
}

#[derive(Debug, Clone, Copy, Default)]
enum AppTab {
    #[default]
    Search,
    Settings,
    BomImport,
}

#[derive(Debug)]
pub struct App {
    pub dark_mode: bool,
    tab: AppTab,
    network: Arc<Mutex<NetworkClient>>,
    search: Search,
}

impl App {
    pub fn new() -> Self {
        // TODO: Cli flag for this
        let network = Arc::new(Mutex::new(NetworkClient::local_client()));

        Self {
            dark_mode: true,
            tab: AppTab::default(),
            search: Search::new(network.clone()),
            network,
        }
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::HighlightParts(vec) => todo!(),
            AppMessage::Quit => iced::exit(),
            AppMessage::SearchMessage(search_message) => self
                .search
                .update(search_message)
                .map(AppMessage::SearchMessage),
        }
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        let root = widget::container(widget::row!(
            match self.tab {
                AppTab::Search => self.draw_search_tab(),
                _ => todo!(),
            },
            widget::horizontal_space().width(Length::Fill)
        ))
        .center(Length::Fill)
        .padding(16.0);
        root.into()
    }

    fn draw_search_tab(&self) -> iced::Element<'_, AppMessage> {
        widget::row(vec![self.search.view().map(AppMessage::SearchMessage)])
            .spacing(16.0)
            .into()
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        let keys = listen_with(|event, _, _| match event {
            iced::Event::Keyboard(event) => {
                let mut config = CONFIG.write().unwrap();
                config.keyboard.dispatch(event).map(|x| (*x).into())
            }
            _ => None,
        });

        Subscription::batch(vec![keys])
    }
}
