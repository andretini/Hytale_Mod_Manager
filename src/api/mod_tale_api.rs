use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock, RwLock};
use reqwest::Client;
use crate::api::curse_forge_api::CurseForgeMod;

const MODTALE_API: &str = "https://api.modtale.net/api/v1";
const MODTALE_CDN: &str = "https://cdn.modtale.net";

static CLIENT: OnceLock<RwLock<Arc<Client>>> = OnceLock::new();

fn client_store() -> &'static RwLock<Arc<Client>> {
    CLIENT.get_or_init(|| RwLock::new(Arc::new(build_client(None))))
}

fn build_client(api_key: Option<&str>) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("HytaleModManager/1.0"));

    if let Some(key) = api_key {
        let auth_val = format!("{}", key);
        if let Ok(val) = HeaderValue::from_str(&auth_val) {
            headers.insert("X-MODTALE-KEY", val);
        }
    }

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build HTTP client")
}

fn client() -> Arc<Client> {
    client_store().read().unwrap().clone()
}

pub fn set_global_api_key(key: &str) {
    let new_client = Arc::new(build_client(Some(key)));
    if let Ok(mut lock) = client_store().write() {
        *lock = new_client;
        println!("Global API key applied to all future requests on ModTale.");
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub content: Vec<T>,
    pub total_pages: u32,
    pub total_elements: u64,
    pub last: bool,
    pub first: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModTaleMod {
    pub id: String,

    #[serde(alias = "title")]
    pub name: String,

    pub slug: Option<String>,

    #[serde(alias = "description")]
    pub summary: Option<String>,

    pub author: String,

    #[serde(alias = "imageUrl")]
    pub icon_url: Option<String>,

    #[serde(alias = "bannerUrl")]
    pub banner_url: Option<String>,

    pub download_count: u64,

    pub categories: Option<Vec<String>>,

    pub created_at: String,
    pub updated_at: String,

    pub versions: Option<Vec<ModTaleFile>>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModTaleFile {
    pub id: String,
    pub version_number: String,
    #[serde(alias = "gameVersions")]
    pub supported_versions: Vec<String>,
    #[serde(alias = "fileUrl")]
    pub download_url: Option<String>,
    #[serde(alias = "releaseDate")]
    pub created_at: String,
    pub download_count: u64,
    pub channel: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub current_page: u32,
    pub last_page: u32,
    pub total: u32,
    pub per_page: u32,
}

pub async fn search_mods(query: String, sort: &str, offset: u32) -> Result<(Vec<ModTaleMod>, Option<Meta>), String> {
    let url = format!("{}/projects", MODTALE_API);

    let limit: u32 = 20;
    let page_index = offset / limit;

    let params = [
        ("q", query.as_str()),
        ("sort", sort),
        ("page", &page_index.to_string()),
        ("size", &limit.to_string()),
    ];

    println!("[ModTale DEBUG] Searching Page: {} (Limit: {})", page_index, limit);

    let resp = client()
        .get(&url)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("Network Request Failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("ModTale API Error: {}", resp.status()));
    }

    let json: PageResponse<ModTaleMod> = resp.json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let meta = Meta {
        current_page: page_index,
        last_page: json.total_pages,
        total: json.total_elements as u32,
        per_page: limit,
    };

    Ok((json.content, Some(meta)))
}

pub async fn search_exact_mod(query: String) -> Option<ModTaleMod> {
    if let Ok((mods, _)) = search_mods(query.clone(), "desc", 0).await {
        return mods.into_iter().find(|m| {
            m.name.to_lowercase() == query.to_lowercase() ||
            m.slug.to_owned().unwrap_or_default().to_lowercase() == query.to_lowercase()
        });
    }
    None
}


pub async fn get_mod(mod_id: &str) -> Result<ModTaleMod, String> {
    let url = format!("{}/projects/{}", MODTALE_API, mod_id);

    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network Request Failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API Error {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let mod_data: ModTaleMod = resp.json()
        .await
        .map_err(|e| format!("Serde Parsing Error: {}", e))?;

    Ok(mod_data)
}

pub async fn get_mod_files(mod_id: &str) -> Result<Vec<ModTaleFile>, String> {
    let mod_data = get_mod(mod_id).await?;
    Ok(mod_data.versions.unwrap_or_default())
}

pub async fn download_url(url: &str) -> Result<Vec<u8>, String> {
    let full_url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{}/{}", MODTALE_CDN, url)
    };

    println!("[ModTale] Downloading from: {}", full_url);

    let resp = client()
        .get(&full_url)
        .send()
        .await
        .map_err(|e| format!("Request Send Error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| format!("Failed to read bytes: {}", e))?;
    Ok(bytes.to_vec())
}