use crate::api::{curse_forge_api, mod_tale_api};
use crate::api::curse_forge_api::{ApiFile, CurseForgeMod, ModFile};
use crate::api::mod_tale_api::{ModTaleFile, ModTaleMod};
use crate::api::settings::{ApiProvider, AppSettings};

#[derive(Debug, Clone, PartialEq)]
pub struct UiModVersion {
    pub file_id: String,
    pub display_name: String,
    pub file_name: String,
    pub download_url: Option<String>,
    pub release_type: u8,
    pub game_versions: Vec<String>,
    pub upload_date: String,
}

impl UiModVersion {
    pub fn from_curseforge_mod_file(file: &ModFile) -> Self {
        Self {
            file_id: file.id.to_string(),
            display_name: file.display_name.clone(),
            file_name: file.file_name.clone(),
            download_url: file.download_url.clone(),
            release_type: file.release_type as u8,
            game_versions: file.game_versions.clone(),
            upload_date: file.file_date.clone(),
        }
    }

    pub fn from_curseforge_api_file(file: &ApiFile) -> Self {
        Self {
            file_id: file.id.to_string(),
            display_name: file.display_name.clone(),
            file_name: file.file_name.clone(),
            download_url: file.download_url.clone(),
            release_type: file.release_type as u8,
            game_versions: file.game_versions.clone(),
            upload_date: file.file_date.clone(),
        }
    }

    pub fn from_modtale_file(file: &ModTaleFile) -> Self {
        let release_type = match file.channel.as_deref() {
            Some("BETA") => 2,
            Some("ALPHA") => 3,
            _ => 1,
        };

        let file_name = file.download_url.as_ref()
            .and_then(|url| url.split('/').last())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.jar", file.version_number));

        Self {
            file_id: file.id.clone(),
            display_name: file.version_number.clone(),
            file_name,
            download_url: file.download_url.clone(),
            release_type,
            game_versions: file.supported_versions.clone(),
            upload_date: file.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiMod {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub authors: String,
    pub download_count: u64,
    pub categories: Vec<String>,
    pub icon: String,
    pub banner: String,
    pub gallery_urls: Vec<String>,
    pub website_url: String,
    pub version: UiModVersion,
}

impl UiMod {
    pub fn from_curseforge_api(api_mod: &CurseForgeMod) -> Self {
        let authors = api_mod.authors.iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let categories = api_mod.categories.iter()
            .map(|c| c.name.clone())
            .collect();

        let icon = api_mod.logo.as_ref()
            .map(|l| l.thumbnail_url.clone())
            .unwrap_or_default();

        let banner = api_mod.screenshots.first()
            .map(|s| s.url.clone())
            .unwrap_or_else(|| icon.clone());

        let gallery_urls = api_mod.screenshots.iter()
            .skip(1)
            .map(|s| s.thumbnail_url.clone())
            .take(3)
            .collect();

        let version = if let Some(latest) = api_mod
            .latest_files
            .iter()
            .find(|v| v.release_type == 1)
            .or(api_mod.latest_files.first())
        {
            UiModVersion::from_curseforge_mod_file(latest)
        } else {
            UiModVersion_dummy()
        };

        Self {
            id: api_mod.id.to_string(),
            name: api_mod.name.clone(),
            summary: api_mod.summary.clone(),
            authors,
            download_count: api_mod.download_count as u64,
            categories,
            icon,
            banner,
            gallery_urls,
            website_url: api_mod.links.website_url.clone(),
            version
        }
    }

    pub fn from_modtale_api(modtale_mod: &ModTaleMod) -> Self {
        let icon = modtale_mod.icon_url.clone().unwrap_or_default();
        let banner = modtale_mod.banner_url.clone().unwrap_or(icon.clone());

        let version = if let Some(versions) = &modtale_mod.versions {
            if let Some(first) = versions.first() {
                UiModVersion::from_modtale_file(first)
            } else {
                UiModVersion_dummy()
            }
        } else {
            UiModVersion_dummy()
        };

        let slug = modtale_mod.slug.clone().unwrap_or_else(|| modtale_mod.id.clone());

        Self {
            id: modtale_mod.id.clone(),
            name: modtale_mod.name.clone(),
            summary: modtale_mod.summary.clone().unwrap_or_default(),
            authors: modtale_mod.author.clone(),
            download_count: modtale_mod.download_count,
            categories: modtale_mod.categories.clone().unwrap_or_default(),
            icon,
            banner,
            gallery_urls: vec![],
            website_url: format!("https://modtale.net/project/{}", slug),
            version,
        }
    }
}

fn UiModVersion_dummy() -> UiModVersion {
    UiModVersion {
        file_id: "".to_string(),
        display_name: "No files".to_string(),
        file_name: "".to_string(),
        download_url: None,
        release_type: 0,
        game_versions: vec![],
        upload_date: "".to_string(),
    }
}

pub async fn search_mods_unified(
    settings: &AppSettings,
    sort: u32,
    query: String,
    offset: u32,
) -> Result<(Vec<UiMod>, u32), String> {
    match settings.api_provider {
        ApiProvider::CurseForge => {
            match curse_forge_api::search_mods(query, sort, offset).await {
                Ok((api_mods, pagination)) => {
                    let ui_mods: Vec<UiMod> = api_mods.iter()
                        .map(|m| UiMod::from_curseforge_api(m))
                        .collect();

                    let total_pages = if let Some(p) = pagination {
                        (p.total_count as f64 / p.page_size as f64).ceil() as u32
                    } else {
                        0
                    };

                    Ok((ui_mods, total_pages))
                }
                Err(e) => Err(e),
            }
        }
        ApiProvider::Modtale => {
            let sort = match sort {
                1 => "relevance",
                2 => "downloads",
                3 => "updated",
                _ => "downloads",
            };
            match mod_tale_api::search_mods(query, sort, offset).await {
                Ok((api_mods, meta)) => {
                    let ui_mods: Vec<UiMod> = api_mods.iter()
                        .map(|m| UiMod::from_modtale_api(m))
                        .collect();

                    let total_pages = if let Some(m) = meta {
                        m.last_page
                    } else {
                        0
                    };

                    Ok((ui_mods, total_pages))
                }
                Err(e) => Err(e),
            }
        }
    }
}

pub async fn get_mod_versions_unified(
    settings: &AppSettings,
    mod_id: &str,
) -> Result<Vec<UiModVersion>, String> {
    match settings.api_provider {
        ApiProvider::CurseForge => {
            let cf_id = mod_id.parse::<u32>()
                .map_err(|_| "Invalid ID format for CurseForge (expected number)".to_string())?;

            match curse_forge_api::get_mod_files(cf_id).await {
                Ok(files) => {
                    let versions = files.iter()
                        .map(|f| UiModVersion::from_curseforge_api_file(f))
                        .collect();
                    Ok(versions)
                }
                Err(e) => {
                    println!("CurseForge Versions fetch FAILED: {}", e);
                    Err(e.to_string())
                }
            }
        }
        ApiProvider::Modtale => {
            match mod_tale_api::get_mod_files(mod_id).await {
                Ok(files) => {
                    let versions = files.iter()
                        .map(|f| UiModVersion::from_modtale_file(f))
                        .collect();
                    Ok(versions)
                }
                Err(e) => {
                    println!("ModTale Versions fetch FAILED: {}", e);
                    Err(e)
                }
            }
        }
    }
}

pub async fn download_version_unified(
    settings: &AppSettings,
    version: &UiModVersion,
) -> Result<(String, Vec<u8>), String> {
    let url = version.download_url.as_ref()
        .ok_or("No download URL available")?;

    match settings.api_provider {
        ApiProvider::CurseForge => curse_forge_api::download_url(url).await.map(|bytes| (version.file_name.clone(), bytes)),
        ApiProvider::Modtale => {
            match mod_tale_api::download_url(url).await {
                Ok(bytes) => {
                    Ok((version.file_name.clone(), bytes))
                }
                Err(e) => {
                    println!("ModTale Download FAILED: {}", e);
                    Err(e)
                }
            }
        }
    }
}

pub async fn get_mod_details_unified(
    provider: &ApiProvider,
    mod_id: &str,
) -> Option<UiMod> {
    match provider {
        ApiProvider::CurseForge => {
            if let Ok(id_num) = mod_id.parse::<u32>() {
                match curse_forge_api::get_mod(id_num).await {
                    Ok(m) => Some(UiMod::from_curseforge_api(&m)),
                    Err(e) => {
                        println!("CurseForge fetch failed for {}: {}", mod_id, e);
                        None
                    }
                }
            } else {
                println!("Invalid CurseForge ID: {}", mod_id);
                None
            }
        }
        ApiProvider::Modtale => {
            match mod_tale_api::get_mod(mod_id).await {
                Ok(m) => {
                    Some(UiMod::from_modtale_api(&m))
                }
                Err(e) => {
                    println!("ModTale fetch FAILED for {}: {}", mod_id, e);
                    None
                }
            }
        }
    }
}