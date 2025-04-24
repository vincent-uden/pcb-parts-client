use common::network::NetworkClient;

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
}

#[derive(Debug, Clone)]
pub enum AppMessage {}
