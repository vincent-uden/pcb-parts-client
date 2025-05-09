use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use common::{
    models::{Part, PartWithCountAndStock, PartWithStock, Profile, User},
    network::{self, NetworkClient},
};
use iced::{
    Border, Color, Element, Length, Subscription, Theme, alignment,
    event::listen_with,
    theme::palette,
    widget::{self, svg},
};

use crate::{
    CONFIG,
    grid::{GridMessage, widget::GridWidget},
    icons,
    search::{SearchMessage, widget::Search},
    settings::Grid,
};

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Tells the grid to highlight some parts
    HighlightParts(Vec<PartWithCountAndStock>),
    SearchMessage(SearchMessage),
    GridMessage(GridMessage),
    Modal(OpenModal),
    StockModalAmount(String),
    StockModalRow(String),
    StockModalColumn(String),
    StockModalZ(String),
    ChangeStock(i64),
    StockModalSuccess,
    LoginModalEmail(String),
    LoginModalPassword(String),
    ConfirmLogin,
    LoginModalNewEmail(String),
    LoginModalNewPassword(String),
    ConfirmUserCreation,
    LoginSuccess,
    ProfilesFetched(Vec<Profile>),
    ProfilesFetchFail,
    SelectProfile(i64),
    ConfirmProfile,
    NewProfile,
    NewProfilePending(String),
    NewProfileFailed,
    Quit,
    StockModalFail,
    LoginFail,
    UserCreationFailed,
    UserCreationSuccess,
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
    Login,
    SelectProfile,
}

#[derive(Debug, Clone, Default)]
pub struct StockModalData {
    pub stock_diff: String,
    pub column: String,
    pub row: String,
    pub z: String,
}

#[derive(Debug, Clone, Default)]
pub struct LoginModalData {
    pub email: String,
    pub password: String,
    pub new_email: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileModalData {
    pub new_name: String,
    pub profiles: Vec<Profile>,
    pub selected_prof: Option<i64>,
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
    login_modal_data: LoginModalData,
    profile_modal_data: ProfileModalData,
}

impl App {
    pub fn new() -> Self {
        // TODO: Error states
        // TODO: Cli flag for network client
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
            login_modal_data: LoginModalData::default(),
            profile_modal_data: ProfileModalData::default(),
        }
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::HighlightParts(vec) => self
                .grid
                .update(GridMessage::HighlightParts(vec))
                .map(AppMessage::GridMessage),
            AppMessage::Quit => match self.modal {
                OpenModal::None => iced::exit(),
                _ => iced::Task::none(),
            },
            AppMessage::SearchMessage(SearchMessage::ChangeStock(part)) => {
                iced::Task::done(AppMessage::Modal(OpenModal::ChangeStock(part)))
            }
            AppMessage::SearchMessage(ref msg @ SearchMessage::BomPartsSearchResult(ref part)) => {
                self.search
                    .update(msg.clone())
                    .map(AppMessage::SearchMessage)
                    .chain(iced::Task::done(AppMessage::HighlightParts(part.clone())))
            }
            AppMessage::SearchMessage(ref msg @ SearchMessage::CloseBom) => self
                .search
                .update(msg.clone())
                .map(AppMessage::SearchMessage)
                .chain(iced::Task::done(AppMessage::HighlightParts(vec![]))),
            AppMessage::SearchMessage(search_message) => self
                .search
                .update(search_message)
                .map(AppMessage::SearchMessage),
            AppMessage::Modal(open_modal) => {
                self.modal = open_modal;
                match &self.modal {
                    OpenModal::None => iced::Task::none(),
                    OpenModal::ChangeStock(part_with_stock) => {
                        self.stock_modal_data.column = part_with_stock.column.to_string();
                        self.stock_modal_data.row = part_with_stock.row.to_string();
                        self.stock_modal_data.z = part_with_stock.z.to_string();
                        iced::Task::none()
                    }
                    OpenModal::Login => {
                        self.login_modal_data = LoginModalData::default();
                        iced::Task::none()
                    }
                    OpenModal::SelectProfile => {
                        self.profile_modal_data.profiles.clear();
                        if let Some(id) = self
                            .network
                            .blocking_lock()
                            .user_data
                            .profile
                            .as_ref()
                            .map(|p| p.id)
                        {
                            self.profile_modal_data.selected_prof = Some(id);
                        }
                        iced::Task::perform(Self::fetch_profiles(self.network.clone()), |output| {
                            match output {
                                Ok(p) => AppMessage::ProfilesFetched(p),
                                Err(_) => AppMessage::ProfilesFetchFail,
                            }
                        })
                    }
                }
            }
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
            AppMessage::LoginModalEmail(s) => {
                self.login_modal_data.email = s;
                iced::Task::none()
            }
            AppMessage::LoginModalPassword(s) => {
                self.login_modal_data.password = s;
                iced::Task::none()
            }
            AppMessage::ConfirmLogin => iced::Task::perform(
                App::login(self.network.clone(), self.login_modal_data.clone()),
                |output| match output {
                    Ok(_) => AppMessage::LoginSuccess,
                    Err(_) => AppMessage::LoginFail,
                },
            ),
            AppMessage::LoginSuccess => {
                self.login_modal_data = LoginModalData::default();
                self.modal = OpenModal::None;
                iced::Task::none()
            }
            AppMessage::LoginFail => iced::Task::none(),
            AppMessage::ProfilesFetched(vec) => {
                self.profile_modal_data.profiles = vec;
                iced::Task::none()
            }
            AppMessage::ProfilesFetchFail => iced::Task::none(),
            AppMessage::SelectProfile(id) => {
                self.profile_modal_data.selected_prof = Some(id);
                iced::Task::none()
            }
            AppMessage::ConfirmProfile => iced::Task::perform(
                Self::confirm_login(
                    self.network.clone(),
                    self.profile_modal_data
                        .profiles
                        .iter()
                        .find(|p| p.id == self.profile_modal_data.selected_prof.unwrap_or(-1))
                        .unwrap()
                        .clone(),
                ),
                |_| AppMessage::Modal(OpenModal::None),
            ),
            AppMessage::NewProfile => iced::Task::perform(
                Self::create_new_profile(
                    self.network.clone(),
                    self.profile_modal_data.new_name.clone(),
                ),
                |output| match output {
                    Ok(_) => AppMessage::Modal(OpenModal::SelectProfile),
                    Err(_) => AppMessage::NewProfileFailed,
                },
            ),
            AppMessage::NewProfileFailed => iced::Task::none(),
            AppMessage::NewProfilePending(s) => {
                self.profile_modal_data.new_name = s;
                iced::Task::none()
            }
            AppMessage::LoginModalNewEmail(s) => {
                self.login_modal_data.new_email = s;
                iced::Task::none()
            }
            AppMessage::LoginModalNewPassword(s) => {
                self.login_modal_data.new_password = s;
                iced::Task::none()
            }
            AppMessage::ConfirmUserCreation => iced::Task::perform(
                Self::create_user(
                    self.network.clone(),
                    self.login_modal_data.new_email.clone(),
                    self.login_modal_data.new_password.clone(),
                ),
                |output| match output {
                    Ok(_) => AppMessage::UserCreationSuccess,
                    Err(_) => AppMessage::UserCreationFailed,
                },
            ),
            AppMessage::UserCreationFailed => iced::Task::none(),
            AppMessage::UserCreationSuccess => {
                self.login_modal_data.email = self.login_modal_data.new_email.clone();
                self.login_modal_data.password = self.login_modal_data.new_password.clone();
                iced::Task::done(AppMessage::ConfirmLogin)
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        let root = widget::container(widget::column![
            self.draw_status_bar(),
            widget::row!(
                match self.tab {
                    AppTab::Search => self.draw_search_tab(),
                    _ => todo!(),
                },
                self.grid.view().map(AppMessage::GridMessage),
            )
            .spacing(16.0),
        ])
        .center(Length::Fill)
        .padding(16.0);
        match &self.modal {
            OpenModal::None => root.into(),
            OpenModal::ChangeStock(part) => modal(
                root,
                self.draw_change_stock_modal(part),
                AppMessage::Modal(OpenModal::None),
            ),
            OpenModal::Login => modal(
                root,
                self.draw_login_modal(),
                AppMessage::Modal(OpenModal::None),
            ),
            OpenModal::SelectProfile => modal(
                root,
                self.draw_profile_modal(),
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

    fn draw_status_bar(&self) -> iced::Element<'_, AppMessage> {
        let n = self.network.blocking_lock();
        let user_data = n.user_data.clone();
        widget::row![
            widget::horizontal_space().width(Length::Fill),
            widget::text(user_data.user.unwrap_or_default().email),
            widget::vertical_rule(2.0),
            widget::svg(icons::user())
                .width(18.0)
                .height(18.0)
                .style(|theme: &Theme, _| {
                    let palette = theme.palette();
                    svg::Style {
                        color: Some(palette.text),
                    }
                }),
            widget::text(user_data.profile.unwrap_or_default().name),
            widget::vertical_rule(2.0),
            widget::svg(icons::server())
                .width(18.0)
                .height(18.0)
                .style(|theme: &Theme, _| {
                    let palette = theme.palette();
                    svg::Style {
                        color: Some(palette.text),
                    }
                }),
            widget::text(n.host_name()),
        ]
        .spacing(4.0)
        .align_y(alignment::Vertical::Center)
        .height(Length::Shrink)
        .into()
    }

    pub fn subscription(&self) -> Subscription<AppMessage> {
        // TODO: HERE --- Ignore keybinds if in certain, special states such as in text inputs
        let keys = listen_with(|event, _, _| match event {
            iced::Event::Keyboard(event) => {
                let mut config = CONFIG.write().unwrap();
                config.keyboard.dispatch(event).map(|x| (*x).into())
            }
            _ => None,
        });

        Subscription::batch(vec![keys])
    }

    fn draw_login_modal(&self) -> iced::Element<'_, AppMessage> {
        widget::container(
            widget::column![
                widget::text("Login"),
                widget::vertical_space().height(8.0),
                widget::text_input("Email", &self.login_modal_data.email)
                    .on_input(AppMessage::LoginModalEmail),
                widget::text_input("Password", &self.login_modal_data.password)
                    .secure(true)
                    .on_input(AppMessage::LoginModalPassword),
                widget::button("Login").on_press(AppMessage::ConfirmLogin),
                widget::vertical_space().height(8.0),
                widget::horizontal_rule(4.0),
                widget::vertical_space().height(8.0),
                widget::text("Create new account"),
                widget::vertical_space().height(8.0),
                widget::text_input("Email", &self.login_modal_data.new_email)
                    .on_input(AppMessage::LoginModalNewEmail),
                widget::text_input("Password", &self.login_modal_data.new_password)
                    .secure(true)
                    .on_input(AppMessage::LoginModalNewPassword),
                widget::button("Create").on_press(AppMessage::ConfirmUserCreation),
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

    fn draw_profile_modal(&self) -> iced::Element<'_, AppMessage> {
        let mut col = widget::column![
            widget::text("Select Profile"),
            widget::vertical_space().height(8.0),
        ]
        .spacing(4.0);
        for prof in &self.profile_modal_data.profiles {
            let selected = prof.id == self.profile_modal_data.selected_prof.unwrap_or(-1);
            col = col.push(
                widget::button(widget::text(&prof.name))
                    .style(move |theme: &Theme, _status| {
                        // This needs to
                        // be a button styled to look like text
                        let palette = theme.extended_palette();
                        widget::button::Style {
                            text_color: if selected {
                                palette.success.base.color.into()
                            } else {
                                palette.background.base.text.into()
                            },
                            background: None,
                            ..Default::default()
                        }
                    })
                    .on_press(AppMessage::SelectProfile(prof.id)),
            );
        }
        col = col.push(widget::button("Select").on_press(AppMessage::ConfirmProfile));
        col = col.extend(vec![
            widget::horizontal_rule(4.0).into(),
            widget::text_input("New profile name", &self.profile_modal_data.new_name)
                .on_input(AppMessage::NewProfilePending)
                .into(),
            widget::button("Create new profile")
                .on_press(AppMessage::NewProfile)
                .into(),
        ]);
        widget::container(col)
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

    async fn login(network: Arc<Mutex<NetworkClient>>, data: LoginModalData) -> Result<()> {
        let mut n = network.lock().await;
        n.login(User {
            id: 0,
            email: data.email,
            password: data.password,
        })
        .await
    }

    async fn fetch_profiles(network: Arc<Mutex<NetworkClient>>) -> Result<Vec<Profile>> {
        let mut n = network.lock().await;
        n.get_profiles(None).await
    }

    async fn confirm_login(network: Arc<Mutex<NetworkClient>>, profile: Profile) -> Result<()> {
        let mut n = network.lock().await;
        n.user_data.profile = Some(profile);
        Ok(())
    }

    async fn create_new_profile(network: Arc<Mutex<NetworkClient>>, name: String) -> Result<()> {
        let mut n = network.lock().await;
        n.new_profile(name).await
    }

    async fn create_user(
        network: Arc<Mutex<NetworkClient>>,
        email: String,
        password: String,
    ) -> Result<()> {
        let mut n = network.lock().await;
        n.create_user(User {
            email,
            password,
            ..Default::default()
        })
        .await
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
