use std::{collections::HashMap, fmt::Display, fs, sync::Arc};

use anyhow::{Result, anyhow};
use reqwest::{Client, RequestBuilder};
use reqwest_cookie_store::CookieStoreMutex;
use serde::Serialize;
use tracing::{debug, info};
use url::Url;

use crate::models::{Bom, Part, PartWithCountAndStock, PartWithStock, Profile, StockRows, User};

#[derive(Debug)]
pub struct NetworkClient {
    client: Client,
    base_url: Url,
    cookie_store: Arc<CookieStoreMutex>,
}

#[derive(Serialize)]
struct CreateProfileBody {
    name: String,
}

#[derive(Serialize)]
struct UpdateProfileBody {
    id: i32,
    name: String,
}

impl NetworkClient {
    fn build_get(
        &mut self,
        route: &str,
        params: &[(impl Display, impl Display)],
    ) -> RequestBuilder {
        let mut query_string = String::new();
        for (k, v) in params {
            query_string.push_str(&format!("&{}={}", k, v));
        }
        if !query_string.is_empty() {
            query_string.remove(0);
        }
        let mut url = self.base_url.join(route).unwrap();
        url.set_query(Some(&query_string));
        self.client.get(url.as_str())
    }

    fn build_post<T>(&mut self, route: &str, body: &T) -> RequestBuilder
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(self.base_url.join(route).unwrap().as_str())
            .json(body)
    }

    pub fn local_client() -> Self {
        let cookie_store = {
            if let Ok(file) = std::fs::File::open(".cookies.json").map(std::io::BufReader::new) {
                #[allow(deprecated)]
                reqwest_cookie_store::CookieStore::load_json_all(file).unwrap()
            } else {
                reqwest_cookie_store::CookieStore::new(None)
            }
        };
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        Self {
            client: Client::builder()
                .cookie_provider(Arc::clone(&cookie_store))
                .build()
                .unwrap(),
            base_url: Url::parse("http://localhost:3000").unwrap(),
            cookie_store,
        }
    }

    pub async fn create_user(&mut self, pending: User) -> Result<()> {
        let resp_text = self
            .build_post("/api/user/create", &pending)
            .send()
            .await?
            .text()
            .await?;
        println!("{:?}", resp_text);

        Ok(())
    }

    pub async fn login(&mut self, user: User) -> Result<()> {
        let resp_text = self.build_post("/api/user/session", &user).send().await?;
        {
            let cs = self.cookie_store.lock().unwrap();
            if let Ok(mut file) =
                std::fs::File::create(".cookies.json").map(std::io::BufWriter::new)
            {
                println!("Writing");
                #[allow(deprecated)]
                cs.save_incl_expired_and_nonpersistent_json(&mut file)
                    .unwrap();
            } else {
                println!("Couldnt open file");
            }
        };
        Ok(())
    }

    pub async fn get_parts(
        &mut self,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Vec<Part>> {
        let mut params = vec![];
        if let Some(name) = name {
            params.push(("name", name));
        }
        if let Some(description) = description {
            params.push(("description", description));
        }
        let resp = self
            .build_get("/api/parts", &params)
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub async fn new_part(&mut self, part: Part) -> Result<()> {
        let resp_text = self
            .build_post("/api/parts", &part)
            .send()
            .await?
            .text()
            .await?;
        println!("{:?}", resp_text);
        Ok(())
    }

    pub async fn get_profiles(&mut self, name: Option<String>) -> Result<Vec<Profile>> {
        let mut params = vec![];
        if let Some(name) = name {
            params.push(("name", name));
        }
        let resp = self
            .build_get("/api/profile", &params)
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub async fn new_profile(&mut self, name: String) -> Result<()> {
        let body = CreateProfileBody { name };
        let _resp_text = self
            .build_post("/api/profile", &body)
            .send()
            .await?
            .text()
            .await?;
        Ok(())
    }

    pub async fn list_stock(&mut self, profile_id: i64) -> Result<Vec<StockRows>> {
        let resp_text = self
            .build_get("/api/stock", &[("profileId", profile_id)])
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp_text)?)
    }

    pub async fn stock_part(
        &mut self,
        profile_id: i64,
        part_id: i64,
        stock: i64,
        column: i64,
        row: i64,
        z: i64,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct StockBody {
            profile_id: i64,
            part_id: i64,
            stock: i64,
            column: i64,
            row: i64,
            z: i64,
        }
        let body = StockBody {
            profile_id,
            part_id,
            stock,
            column,
            row,
            z,
        };
        let _resp_text = self
            .build_post("/api/stock", &body)
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }

    pub async fn list_boms(&mut self, profile_id: i64, bom_id: Option<i64>) -> Result<Vec<Bom>> {
        let mut params = vec![("profileId", profile_id)];
        if let Some(bom_id) = bom_id {
            params.push(("bomId", bom_id));
        }
        let resp_text = self
            .build_get("/api/bom", &params)
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp_text)?)
    }

    pub async fn new_bom(
        &mut self,
        profile_id: i64,
        name: String,
        description: String,
        candidates: Vec<(i64, Part)>,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct BomRow {
            count: i64,
            part: Part,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct BomBody {
            profile_id: i64,
            name: String,
            description: String,
            parts: Vec<BomRow>,
        }

        let body = BomBody {
            profile_id,
            name,
            description,
            parts: candidates
                .into_iter()
                .map(|(count, part)| BomRow { count, part })
                .collect(),
        };

        let resp_text = self
            .build_post("/api/bom", &body)
            .send()
            .await?
            .text()
            .await?;
        println!("{}", resp_text);
        Ok(())
    }

    pub async fn parts_in_bom(
        &mut self,
        profile_id: i64,
        bom_id: i64,
    ) -> Result<Vec<PartWithCountAndStock>> {
        let resp = self
            .build_get("/api/bom/parts", &[
                ("profileId", profile_id),
                ("bomId", bom_id),
            ])
            .send()
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub async fn parts_with_stock(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        profile_id: i64,
    ) -> Result<Vec<PartWithStock>> {
        let mut params = vec![("profileId", format!("{}", profile_id))];
        if let Some(name) = name {
            params.push(("name", name));
        }
        if let Some(description) = description {
            params.push(("description", description));
        }
        let resp = self
            .build_get("/api/parts/stock", &params)
            .send()
            .await?
            .text()
            .await?;
        info!("{}", resp);
        Ok(serde_json::from_str(&resp)?)
    }
}
