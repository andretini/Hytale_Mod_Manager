use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock, RwLock};
use reqwest::Client;

const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const HYTALE_GAME_ID: u32 = 70216;

static CLIENT: OnceLock<RwLock<Arc<Client>>> = OnceLock::new();

fn client_store() -> &'static RwLock<Arc<Client>> {
    CLIENT.get_or_init(|| RwLock::new(Arc::new(build_client(None))))
}

fn build_client(api_key: Option<&str>) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("HytaleModManager/1.0"));

    if let Some(key) = api_key {
        headers.insert("x-api-key", HeaderValue::from_str(key).unwrap());
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
        println!("Global API key applied to all future requests on Curse Forge.");
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiFile {
    pub id: u32,
    pub display_name: String,
    pub file_name: String,
    pub file_date: String,
    pub file_length: u64,
    pub release_type: u8,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GetFilesResponse {
    pub data: Vec<ApiFile>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub data: T,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub index: u32,
    pub page_size: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMod {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub links: ModLinks,
    pub summary: String,
    pub status: i32,
    pub download_count: f64,
    pub is_featured: bool,
    pub primary_category_id: Option<u32>,
    pub categories: Vec<Category>,
    pub class_id: Option<u32>,
    pub authors: Vec<Author>,
    pub logo: Option<ModAsset>,
    pub screenshots: Vec<ModAsset>,
    pub main_file_id: u32,
    pub latest_files: Vec<ModFile>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub allow_mod_distribution: Option<bool>,
    pub game_popularity_rank: Option<u32>,
    pub is_available: Option<bool>,
    pub thumbs_up_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModLinks {
    pub website_url: String,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: String,
    pub is_class: Option<bool>,
    pub class_id: Option<u32>,
    pub parent_category_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModAsset {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    pub id: u32,
    pub game_id: u32,
    pub mod_id: u32,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: i32,
    pub file_status: i32,
    pub hashes: Vec<FileHash>,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u64,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    pub dependencies: Vec<FileDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHash {
    pub value: String,
    pub algo: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDependency {
    pub mod_id: u32,
    pub relation_type: i32,
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub id: u32,
    pub name: String,
}

pub async fn search_mods(query: String, search_sort: u32, offset: u32) -> Result<(Vec<CurseForgeMod>, Option<Pagination>), String> {
    let url = format!("{}/mods/search", CURSEFORGE_API);
    let search_filter = if query.trim().is_empty() { "" } else { &query };

    let params = [
        ("gameId", HYTALE_GAME_ID.to_string()),
        ("searchFilter", search_filter.to_string()),
        ("category", "0".to_string()),
        ("pageSize", "20".to_string()),
        ("sortField", (search_sort).to_string()),
        ("sortOrder", "desc".to_string()),
        ("index", offset.to_string()),
    ];

    let resp = client()
        .get(&url)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("Network Request Failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let json: ApiResponse<Vec<CurseForgeMod>> = resp.json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok((json.data, json.pagination))
}

pub async fn search_exact_mod(query: String) -> Option<CurseForgeMod> {
    if let Ok((mods, _)) = search_mods(query.clone(), 0, 0).await {
        return mods.into_iter().find(|m| {
            m.name.to_lowercase() == query.to_lowercase() ||
            m.slug.to_lowercase() == query.to_lowercase()
        });
    }
    None
}

pub async fn download_image(url: String) -> Result<Vec<u8>, String> {
    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Image download failed: {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

pub async fn find_hytale_id() -> Result<u32, String> {
    let url = format!("{}/games", CURSEFORGE_API);
    let params = [("index", "0"), ("pageSize", "50")];

    let resp = client()
        .get(&url)
        .query(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: ApiResponse<Vec<GameData>> = resp.json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(hytale) = json.data.iter().find(|g| g.name == "Hytale") {
        Ok(hytale.id)
    } else {
        Err("Hytale ID not found in CurseForge response".to_string())
    }
}

pub async fn get_mod_files(mod_id: u32) -> Result<Vec<ApiFile>, reqwest::Error> {
    let url = format!("{}/mods/{}/files?pageSize=50", CURSEFORGE_API, mod_id);

    let res = client().get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !res.status().is_success() {
        println!("API Error: {}", res.status());
        res.error_for_status_ref()?;
    }

    let body: GetFilesResponse = res.json().await?;
    Ok(body.data)
}
pub async fn download_url(url: &str) -> Result<Vec<u8>, String> {
    let resp = client()
        .get(url)
        .send()
        .await
        .map_err(|e| {
            let err = format!("Request Send Error: {}", e);
            println!("{}", err);
            err
        })?;

    let status = resp.status();

    if !status.is_success() {
        let err = format!("Download failed: {}", status);
        println!("{}", err);
        return Err(err);
    }

    let bytes = resp.bytes().await.map_err(|e| {
        let err = format!("Failed to read bytes: {}", e);
        println!("{}", err);
        err
    })?;

    Ok(bytes.to_vec())
}

pub async fn download_mod(mod_data: &CurseForgeMod) -> Result<(String, Vec<u8>), String> {

    let latest_file = mod_data.latest_files.first()
        .ok_or_else(|| {
            let err = "No files found for this mod".to_string();
            println!("ERROR: {}", err);
            err
        })?;

    download_mod_version(latest_file).await
}

pub async fn download_mod_version(file: &ModFile) -> Result<(String, Vec<u8>), String> {
    let url = file.download_url.as_ref()
        .ok_or("No download URL available for this file".to_string())?;

    let bytes = download_url(url).await?;

    Ok((file.file_name.clone(), bytes))
}

pub async fn get_mod(mod_id: u32) -> Result<CurseForgeMod, String> {
    let url = format!("{}/mods/{}", CURSEFORGE_API, mod_id);

    let resp = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network Request Failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Failed to get text: {}", e))?;

    if !status.is_success() {
        return Err(format!("API Error {}: {}", status, text));
    }

    let json: ApiResponse<CurseForgeMod> = serde_json::from_str(&text)
        .map_err(|e| format!("Serde Parsing Error: {} | Raw JSON: {}", e, text))?;

    Ok(json.data)
}