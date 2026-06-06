use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub(crate) fn read_yaml_file<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let text = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}

pub(crate) fn write_yaml_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let text = serde_yaml::to_string(value)
        .map_err(|error| format!("Could not serialize {}: {error}", path.display()))?;
    fs::write(path, text).map_err(|error| format!("Could not write {}: {error}", path.display()))
}
