use anyhow::Result;
use reqwest::Client;
use url::Url;

use crate::models::User;

#[derive(Debug)]
pub struct NetworkClient {
    client: Client,
    base_url: Url,
}

impl NetworkClient {
    pub fn local_client() -> Self {
        Self {
            client: Client::builder().cookie_store(true).build().unwrap(),
            base_url: Url::parse("http://localhost:3000").unwrap(),
        }
    }

    pub async fn create_user(&mut self, pending: User) -> Result<bool> {
        let resp_text = self
            .client
            .post(self.base_url.join("/api/user/create").unwrap().as_str())
            .json(&pending)
            .send()
            .await?
            .text()
            .await?;
        println!("{:?}", resp_text);

        Ok(true)
    }
}
