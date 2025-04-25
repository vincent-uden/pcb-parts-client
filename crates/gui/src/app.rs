use std::sync::Arc;
use tokio::sync::Mutex;

use common::{models::Part, network::NetworkClient};
use iced::{Border, Color, Element, Length, Subscription, Theme, event::listen_with, widget};

use crate::{
    CONFIG,
    search::{SearchMessage, widget::Search},
};

// TODO: Figure out where to store login and auth information.
//       Some kind of context system?
//       Maybe just a global like the config?
//       Must be persisted in an encrypted manner, cant use the config file

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Tells the grid to highlight some parts
    HighlightParts(Vec<Part>),
    SearchMessage(SearchMessage),
    Modal(OpenModal),
    Quit,
}

#[derive(Debug, Clone, Copy, Default)]
enum AppTab {
    #[default]
    Search,
    Settings,
    BomImport,
}

#[derive(Debug, Clone, Copy, Default)]
enum OpenModal {
    #[default]
    None,
    ChangeStock,
}

#[derive(Debug)]
pub struct App {
    pub dark_mode: bool,
    tab: AppTab,
    network: Arc<Mutex<NetworkClient>>,
    search: Search,
    modal: OpenModal,
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
            modal: OpenModal::default(),
        }
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::HighlightParts(vec) => todo!(),
            AppMessage::Quit => iced::exit(),
            AppMessage::SearchMessage(SearchMessage::ChangeStock(part)) => {
                iced::Task::done(AppMessage::Modal(OpenModal::ChangeStock))
            }
            AppMessage::SearchMessage(search_message) => self
                .search
                .update(search_message)
                .map(AppMessage::SearchMessage),
            AppMessage::Modal(open_modal) => {
                self.modal = open_modal;
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        let root = widget::container(widget::row!(
            match self.tab {
                AppTab::Search => self.draw_search_tab(),
                _ => todo!(),
            },
            widget::horizontal_space().width(Length::Fill),
        ))
        .center(Length::Fill)
        .padding(16.0);
        match self.modal {
            OpenModal::None => root.into(),
            OpenModal::ChangeStock => modal(
                root,
                self.draw_change_stock_modal(),
                AppMessage::Modal(OpenModal::None),
            ),
        }
    }

    fn draw_search_tab(&self) -> iced::Element<'_, AppMessage> {
        widget::row(vec![self.search.view().map(AppMessage::SearchMessage)])
            .spacing(16.0)
            .into()
    }

    fn draw_change_stock_modal(&self) -> iced::Element<'_, AppMessage> {
        // TODO: Show bin controls
        widget::container(
            widget::column![
                widget::text("Change Stock"),
                widget::vertical_space().height(8.0),
                widget::text_input("Amount", ""),
                widget::row![widget::button("Add Stock").width(Length::Fill), widget::button("Remove Stock").width(Length::Fill)].spacing(4.0),
                widget::horizontal_rule(4.0),
                widget::row![
                    widget::text("Row").width(Length::Fill),
                    widget::text("Column").width(Length::Fill),
                    widget::text("Z").width(Length::Fill),
                ].spacing(4.0),
                widget::row![
                    widget::text_input("", "").width(Length::Fill),
                    widget::text_input("", "").width(Length::Fill),
                    widget::text_input("", "").width(Length::Fill),
                ].spacing(4.0),
            ]
            .spacing(4.0),
        )
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
        .width(300.0)
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

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    widget::stack![
        base.into(),
        widget::opaque(
            widget::mouse_area(widget::center(widget::opaque(content)).style(|_theme| {
                widget::container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..Default::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
