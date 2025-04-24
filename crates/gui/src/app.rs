use common::network::NetworkClient;
use iced::{Subscription, event::listen_with, widget};

use crate::CONFIG;

#[derive(Debug)]
pub struct App {
    pub dark_mode: bool,
    network: NetworkClient,
}

impl App {
    pub fn new() -> Self {
        Self {
            dark_mode: true,
            // TODO: Cli flag for this
            network: NetworkClient::local_client(),
        }
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Quit => iced::exit(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        widget::text("PCB Part Manager").into()
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

#[derive(Debug, Clone)]
pub enum AppMessage {
    Quit,
}
