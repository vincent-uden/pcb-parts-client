use std::sync::Arc;
use tokio::sync::Mutex;

use common::{
    models::{Part, PartWithStock},
    network::NetworkClient,
};
use iced::{Border, Color, Element, Length, Subscription, Theme, event::listen_with, widget};

use crate::{
    CONFIG,
    grid::{GridMessage, widget::GridWidget},
    search::{SearchMessage, widget::Search},
};

// TODO: Figure out where to store login and auth information.
//       Some kind of context system?
//       Maybe just a global like the config?
//       Must be persisted in an encrypted manner, cant use the config file

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Tells the grid to highlight some parts
    HighlightParts(Vec<PartWithStock>),
    SearchMessage(SearchMessage),
    GridMessage(GridMessage),
    Modal(OpenModal),
    StockModalAmount(String),
    StockModalRow(String),
    StockModalColumn(String),
    StockModalZ(String),
    ChangeStock(i64),
    StockModalSuccess,
    Quit,
    StockModalFail,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AppTab {
    #[default]
    Search,
    Settings,
    BomImport,
}

#[derive(Debug, Clone, Default)]
pub enum OpenModal {
    #[default]
    None,
    ChangeStock(PartWithStock),
}

#[derive(Debug, Clone, Default)]
pub struct StockModalData {
    pub stock_diff: String,
    pub column: String,
    pub row: String,
    pub z: String,
}

#[derive(Debug)]
pub struct App {
    pub dark_mode: bool,
    tab: AppTab,
    network: Arc<Mutex<NetworkClient>>,
    search: Search,
    grid: GridWidget,
    modal: OpenModal,
    stock_modal_data: StockModalData,
}

impl App {
    pub fn new() -> Self {
        // TODO: Cli flag for this
        let network = Arc::new(Mutex::new(NetworkClient::local_client()));
        let config = CONFIG.read().unwrap();

        Self {
            dark_mode: true,
            tab: AppTab::default(),
            search: Search::new(network.clone()),
            grid: GridWidget::new(config.grid),
            network,
            modal: OpenModal::default(),
            stock_modal_data: StockModalData::default(),
        }
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::HighlightParts(vec) => todo!(),
            AppMessage::Quit => match self.modal {
                OpenModal::None => iced::exit(),
                OpenModal::ChangeStock(_) => iced::Task::none(),
            },
            AppMessage::SearchMessage(SearchMessage::ChangeStock(part)) => {
                iced::Task::done(AppMessage::Modal(OpenModal::ChangeStock(part)))
            }
            AppMessage::SearchMessage(search_message) => self
                .search
                .update(search_message)
                .map(AppMessage::SearchMessage),
            AppMessage::Modal(open_modal) => {
                match &open_modal {
                    OpenModal::None => {}
                    OpenModal::ChangeStock(part_with_stock) => {
                        self.stock_modal_data.column = part_with_stock.column.to_string();
                        self.stock_modal_data.row = part_with_stock.row.to_string();
                        self.stock_modal_data.z = part_with_stock.z.to_string();
                    }
                }
                self.modal = open_modal;
                iced::Task::none()
            }
            // TODO: Data validation? Numerical fields
            AppMessage::StockModalAmount(s) => {
                self.stock_modal_data.stock_diff = s;
                iced::Task::none()
            }
            AppMessage::StockModalRow(s) => {
                self.stock_modal_data.row = s;
                iced::Task::none()
            }
            AppMessage::StockModalColumn(s) => {
                self.stock_modal_data.column = s;
                iced::Task::none()
            }
            AppMessage::StockModalZ(s) => {
                self.stock_modal_data.z = s;
                iced::Task::none()
            }
            AppMessage::ChangeStock(diff) => match &self.modal {
                OpenModal::ChangeStock(part) => {
                    let network = self.network.clone();
                    let row = self.stock_modal_data.row.parse().unwrap();
                    let column = self.stock_modal_data.column.parse().unwrap();
                    let z = self.stock_modal_data.z.parse().unwrap();
                    let id = part.id;
                    let stock = part.stock;
                    iced::Task::perform(
                        async move {
                            let mut network = network.lock().await;
                            network
                                .stock_part(1, id, stock + diff, column, row, z)
                                .await
                        },
                        |result| match result {
                            Ok(_) => AppMessage::StockModalSuccess,
                            Err(_) => AppMessage::StockModalFail,
                        },
                    )
                }
                _ => iced::Task::none(),
            },
            AppMessage::StockModalSuccess => {
                self.stock_modal_data = StockModalData::default();
                self.modal = OpenModal::None;
                iced::Task::done(AppMessage::SearchMessage(SearchMessage::SubmitQuery))
            }
            AppMessage::StockModalFail => iced::Task::none(),
            AppMessage::GridMessage(_) => todo!(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        let root = widget::container(
            widget::row!(
                match self.tab {
                    AppTab::Search => self.draw_search_tab(),
                    _ => todo!(),
                },
                self.grid.view().map(AppMessage::GridMessage),
            )
            .spacing(16.0),
        )
        .center(Length::Fill)
        .padding(16.0);
        match &self.modal {
            OpenModal::None => root.into(),
            OpenModal::ChangeStock(part) => modal(
                root,
                self.draw_change_stock_modal(part),
                AppMessage::Modal(OpenModal::None),
            ),
        }
    }

    fn draw_search_tab(&self) -> iced::Element<'_, AppMessage> {
        widget::row(vec![self.search.view().map(AppMessage::SearchMessage)])
            .spacing(16.0)
            .into()
    }

    fn draw_change_stock_modal(&self, part: &PartWithStock) -> iced::Element<'_, AppMessage> {
        widget::container(
            widget::column![
                widget::text(format!("Change Stock - {}", part.name)),
                widget::vertical_space().height(8.0),
                widget::text_input("Amount", &self.stock_modal_data.stock_diff)
                    .on_input(AppMessage::StockModalAmount),
                widget::row![
                    widget::button("Restock").width(Length::Fill).on_press(
                        AppMessage::ChangeStock(
                            self.stock_modal_data.stock_diff.parse().unwrap_or(0)
                        )
                    ),
                    widget::button("Deplete").width(Length::Fill).on_press(
                        AppMessage::ChangeStock(
                            -self.stock_modal_data.stock_diff.parse::<i64>().unwrap_or(0)
                        )
                    ),
                ]
                .spacing(4.0),
                widget::horizontal_rule(4.0),
                widget::row![
                    widget::text("Row").width(Length::Fill),
                    widget::text("Column").width(Length::Fill),
                    widget::text("Z").width(Length::Fill),
                ]
                .spacing(4.0),
                widget::row![
                    widget::text_input("", &self.stock_modal_data.row)
                        .width(Length::Fill)
                        .on_input(AppMessage::StockModalRow),
                    widget::text_input("", &self.stock_modal_data.column)
                        .width(Length::Fill)
                        .on_input(AppMessage::StockModalColumn),
                    widget::text_input("", &self.stock_modal_data.z)
                        .width(Length::Fill)
                        .on_input(AppMessage::StockModalZ),
                ]
                .spacing(4.0),
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
