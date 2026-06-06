use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::yaml_store::{read_yaml_file, write_yaml_file};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct CookieSiteIndex {
    #[serde(default)]
    pub(crate) sites: Vec<CookieSiteIndexEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct CookieSiteIndexEntry {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) login_url: String,
    #[serde(default)]
    pub(crate) match_domains: Vec<String>,
    #[serde(default)]
    pub(crate) cookie_domains: Vec<String>,
    #[serde(default)]
    pub(crate) cookie_file: String,
    #[serde(default)]
    pub(crate) builtin: bool,
    #[serde(default)]
    pub(crate) updated_unix: u64,
}

pub(crate) fn cookie_site_index_path(cookie_dir: &Path) -> PathBuf {
    cookie_dir.join("sites.yaml")
}

pub(crate) fn read_cookie_site_index(cookie_dir: &Path) -> Result<Option<CookieSiteIndex>, String> {
    let path = cookie_site_index_path(cookie_dir);
    if !path.is_file() {
        return Ok(None);
    }
    read_yaml_file(&path)
        .ok_or_else(|| format!("Could not parse cookie site index: {}", path.display()))
        .map(Some)
}

pub(crate) fn read_cookie_site_index_or_default(cookie_dir: &Path) -> CookieSiteIndex {
    read_yaml_file(&cookie_site_index_path(cookie_dir)).unwrap_or_default()
}

pub(crate) fn write_cookie_site_index(
    cookie_dir: &Path,
    index: &CookieSiteIndex,
) -> Result<(), String> {
    write_yaml_file(&cookie_site_index_path(cookie_dir), index)
}
