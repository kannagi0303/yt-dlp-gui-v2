use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const DEFAULT_WINDOW_TITLE: &str = "yt-dlp-gui";
const MAGIC: &[u8] = b"YTDLPGUI_V2_EDITION_V1";
const FOOTER_FIELDS_LEN: u64 = 8 + 8 + 8 + 4;
const FOOTER_LEN: u64 = FOOTER_FIELDS_LEN + MAGIC.len() as u64;
const SUPPORTED_SCHEMA_VERSION: u32 = 1;

static RUNTIME_EDITION: OnceLock<RuntimeEdition> = OnceLock::new();

#[derive(Clone, Debug, Default)]
pub struct RuntimeEdition {
    pub manifest: Option<EditionManifest>,
}

impl RuntimeEdition {
    pub fn is_custom_edition(&self) -> bool {
        self.manifest.is_some()
    }

    pub fn window_title(&self) -> String {
        self.manifest
            .as_ref()
            .and_then(EditionManifest::window_title_caption)
            .unwrap_or(DEFAULT_WINDOW_TITLE)
            .to_owned()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EditionManifest {
    pub schema_version: Option<u32>,
    pub edition: EditionSection,
    pub window_title: Option<String>,
    pub titlebar_caption: Option<String>,
}

impl EditionManifest {
    pub fn window_title_caption(&self) -> Option<&str> {
        first_non_empty([
            self.edition.window_title.as_deref(),
            self.edition.titlebar_caption.as_deref(),
            self.window_title.as_deref(),
            self.titlebar_caption.as_deref(),
        ])
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EditionSection {
    pub id: Option<String>,
    pub name: Option<String>,
    pub window_title: Option<String>,
    pub titlebar_caption: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedEdition {
    pub manifest: EditionManifest,
    pub payload: Vec<u8>,
    pub base_len: u64,
}

#[derive(Clone, Copy, Debug)]
struct EditionFooter {
    payload_len: u64,
    payload_hash: u64,
    base_len: u64,
    schema_version: u32,
}

#[derive(Clone, Debug)]
pub enum EditionCommand {
    Pack {
        edition_yaml: PathBuf,
        target_exe: PathBuf,
    },
    Restore {
        target_exe: PathBuf,
    },
    Export {
        edition_yaml: PathBuf,
    },
}

pub fn parse_edition_command() -> Result<Option<EditionCommand>, String> {
    let mut args = std::env::args_os().skip(1);
    let Some(command) = args.next() else {
        return Ok(None);
    };
    let command = command.to_string_lossy();
    match command.as_ref() {
        "-p" | "--pack-edition" => {
            let edition_yaml = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| "Usage: v2.exe -p <edition.yaml> <target.exe>".to_owned())?;
            let target_exe = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| "Usage: v2.exe -p <edition.yaml> <target.exe>".to_owned())?;
            if args.next().is_some() {
                return Err("Usage: v2.exe -p <edition.yaml> <target.exe>".to_owned());
            }
            Ok(Some(EditionCommand::Pack {
                edition_yaml,
                target_exe,
            }))
        }
        "-r" | "--restore-edition" => {
            let target_exe = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| "Usage: v2ce.exe -r <target.exe>".to_owned())?;
            if args.next().is_some() {
                return Err("Usage: v2ce.exe -r <target.exe>".to_owned());
            }
            Ok(Some(EditionCommand::Restore { target_exe }))
        }
        "-x" | "--export-edition" => {
            let edition_yaml = args
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| "Usage: v2ce.exe -x <edition.yaml>".to_owned())?;
            if args.next().is_some() {
                return Err("Usage: v2ce.exe -x <edition.yaml>".to_owned());
            }
            Ok(Some(EditionCommand::Export { edition_yaml }))
        }
        _ => Ok(None),
    }
}

pub fn run_edition_command(command: EditionCommand) -> Result<(), String> {
    match command {
        EditionCommand::Pack {
            edition_yaml,
            target_exe,
        } => pack_edition(&edition_yaml, &target_exe),
        EditionCommand::Restore { target_exe } => restore_current_exe_without_edition(&target_exe),
        EditionCommand::Export { edition_yaml } => export_current_edition(&edition_yaml),
    }
}

pub fn load_current_runtime_edition() -> RuntimeEdition {
    let manifest = std::env::current_exe()
        .ok()
        .and_then(|path| read_embedded_edition(&path).ok().flatten())
        .map(|embedded| embedded.manifest);
    RuntimeEdition { manifest }
}

pub fn set_runtime_edition(edition: RuntimeEdition) {
    let _ = RUNTIME_EDITION.set(edition);
}

pub fn runtime_window_title() -> String {
    RUNTIME_EDITION
        .get()
        .map(RuntimeEdition::window_title)
        .unwrap_or_else(|| DEFAULT_WINDOW_TITLE.to_owned())
}

pub fn current_exe_has_edition_manifest() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| read_embedded_footer_only(&path).ok().flatten())
        .is_some()
}

fn pack_edition(edition_yaml: &Path, target_exe: &Path) -> Result<(), String> {
    ensure_exe_target(target_exe)?;
    ensure_output_path_is_new(target_exe)?;
    ensure_target_is_not_current_exe(target_exe)?;

    let payload = fs::read(edition_yaml)
        .map_err(|error| format!("Could not read {}: {error}", edition_yaml.display()))?;
    let manifest = parse_manifest_payload(&payload)?;
    validate_manifest(&manifest)?;

    let current_exe = std::env::current_exe()
        .map_err(|error| format!("Could not resolve current executable: {error}"))?;
    let clean_base = clean_executable_bytes(&current_exe)?;
    let bytes = append_edition_payload(clean_base, payload, SUPPORTED_SCHEMA_VERSION)?;
    write_new_file_atomically(target_exe, &bytes)
}

fn restore_current_exe_without_edition(target_exe: &Path) -> Result<(), String> {
    ensure_exe_target(target_exe)?;
    ensure_output_path_is_new(target_exe)?;
    ensure_target_is_not_current_exe(target_exe)?;

    let current_exe = std::env::current_exe()
        .map_err(|error| format!("Could not resolve current executable: {error}"))?;
    let embedded = read_embedded_edition(&current_exe)?.ok_or_else(|| {
        "This executable is not a Custom Edition; there is no EditionManifest tail to remove."
            .to_owned()
    })?;
    let clean_base = read_prefix_bytes(&current_exe, embedded.base_len)?;
    write_new_file_atomically(target_exe, &clean_base)
}

fn export_current_edition(edition_yaml: &Path) -> Result<(), String> {
    ensure_output_path_is_new(edition_yaml)?;
    let current_exe = std::env::current_exe()
        .map_err(|error| format!("Could not resolve current executable: {error}"))?;
    let embedded = read_embedded_edition(&current_exe)?.ok_or_else(|| {
        "This executable is not a Custom Edition; there is no EditionManifest tail to export."
            .to_owned()
    })?;
    write_new_file_atomically(edition_yaml, &embedded.payload)
}

pub fn read_embedded_edition(path: &Path) -> Result<Option<EmbeddedEdition>, String> {
    let Some(footer) = read_embedded_footer_only(path)? else {
        return Ok(None);
    };
    if footer.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported EditionManifest schema version: {}",
            footer.schema_version
        ));
    }
    let file_len = fs::metadata(path)
        .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?
        .len();
    let payload_start = footer.base_len;
    let payload_end = payload_start
        .checked_add(footer.payload_len)
        .ok_or_else(|| "Invalid EditionManifest footer length.".to_owned())?;
    if payload_end.checked_add(FOOTER_LEN) != Some(file_len) {
        return Err("Invalid EditionManifest footer layout.".to_owned());
    }
    let payload = read_range_bytes(path, payload_start, footer.payload_len)?;
    let payload_hash = fnv1a64(&payload);
    if payload_hash != footer.payload_hash {
        return Err("EditionManifest checksum mismatch.".to_owned());
    }
    let manifest = parse_manifest_payload(&payload)?;
    validate_manifest(&manifest)?;
    Ok(Some(EmbeddedEdition {
        manifest,
        payload,
        base_len: footer.base_len,
    }))
}

fn read_embedded_footer_only(path: &Path) -> Result<Option<EditionFooter>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;
    let file_len = metadata.len();
    if file_len < FOOTER_LEN {
        return Ok(None);
    }

    let mut file = fs::File::open(path)
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(file_len - MAGIC.len() as u64))
        .map_err(|error| format!("Could not seek {}: {error}", path.display()))?;
    let mut magic = vec![0_u8; MAGIC.len()];
    file.read_exact(&mut magic)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    if magic != MAGIC {
        return Ok(None);
    }

    file.seek(SeekFrom::Start(file_len - FOOTER_LEN))
        .map_err(|error| format!("Could not seek {}: {error}", path.display()))?;
    let mut fields = [0_u8; FOOTER_FIELDS_LEN as usize];
    file.read_exact(&mut fields)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;

    let footer = EditionFooter {
        payload_len: read_u64_le(&fields[0..8]),
        payload_hash: read_u64_le(&fields[8..16]),
        base_len: read_u64_le(&fields[16..24]),
        schema_version: read_u32_le(&fields[24..28]),
    };
    let expected_len = footer
        .base_len
        .checked_add(footer.payload_len)
        .and_then(|value| value.checked_add(FOOTER_LEN));
    if expected_len != Some(file_len) {
        return Err("Invalid EditionManifest footer layout.".to_owned());
    }
    Ok(Some(footer))
}

fn clean_executable_bytes(path: &Path) -> Result<Vec<u8>, String> {
    match read_embedded_edition(path)? {
        Some(embedded) => read_prefix_bytes(path, embedded.base_len),
        None => {
            fs::read(path).map_err(|error| format!("Could not read {}: {error}", path.display()))
        }
    }
}

fn append_edition_payload(
    mut clean_base: Vec<u8>,
    payload: Vec<u8>,
    schema_version: u32,
) -> Result<Vec<u8>, String> {
    let base_len = clean_base.len() as u64;
    let payload_len = payload.len() as u64;
    let payload_hash = fnv1a64(&payload);
    clean_base.extend_from_slice(&payload);
    clean_base.extend_from_slice(&payload_len.to_le_bytes());
    clean_base.extend_from_slice(&payload_hash.to_le_bytes());
    clean_base.extend_from_slice(&base_len.to_le_bytes());
    clean_base.extend_from_slice(&schema_version.to_le_bytes());
    clean_base.extend_from_slice(MAGIC);
    Ok(clean_base)
}

fn parse_manifest_payload(payload: &[u8]) -> Result<EditionManifest, String> {
    let content = std::str::from_utf8(payload)
        .map_err(|error| format!("edition.yaml must be UTF-8 YAML: {error}"))?;
    serde_yaml::from_str::<EditionManifest>(content)
        .map_err(|error| format!("Could not parse edition.yaml: {error}"))
}

fn validate_manifest(manifest: &EditionManifest) -> Result<(), String> {
    if manifest
        .schema_version
        .is_some_and(|version| version != SUPPORTED_SCHEMA_VERSION)
    {
        return Err(format!(
            "Unsupported edition.yaml schema_version; expected {}.",
            SUPPORTED_SCHEMA_VERSION
        ));
    }
    let Some(title) = manifest.window_title_caption() else {
        return Err(
            "edition.yaml must define edition.window_title, edition.titlebar_caption, window_title, or titlebar_caption."
                .to_owned(),
        );
    };
    if title.chars().count() > 160 {
        return Err("edition window title is too long; keep it under 160 characters.".to_owned());
    }
    Ok(())
}

fn first_non_empty<'a, const N: usize>(values: [Option<&'a str>; N]) -> Option<&'a str> {
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn ensure_exe_target(path: &Path) -> Result<(), String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("exe"))
    {
        Ok(())
    } else {
        Err(format!("Target must be an .exe file: {}", path.display()))
    }
}

fn ensure_output_path_is_new(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "Output already exists: {}. Remove it first, then run the command again.",
            path.display()
        ));
    }
    Ok(())
}

fn ensure_target_is_not_current_exe(path: &Path) -> Result<(), String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("Could not resolve current executable: {error}"))?;
    let target = path
        .canonicalize()
        .unwrap_or_else(|_| absolutize_output_path(path));
    let current = current.canonicalize().unwrap_or(current);
    if target == current {
        return Err("Target executable cannot be the currently running executable.".to_owned());
    }
    Ok(())
}

fn absolutize_output_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn write_new_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("edition-output");
    let tmp = path.with_file_name(format!(".{file_name}.tmp"));
    if tmp.exists() {
        fs::remove_file(&tmp).map_err(|error| {
            format!(
                "Could not remove stale temp file {}: {error}",
                tmp.display()
            )
        })?;
    }
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)
            .map_err(|error| format!("Could not create {}: {error}", tmp.display()))?;
        file.write_all(bytes)
            .map_err(|error| format!("Could not write {}: {error}", tmp.display()))?;
        file.sync_all()
            .map_err(|error| format!("Could not flush {}: {error}", tmp.display()))?;
    }
    fs::rename(&tmp, path).map_err(|error| {
        let _ = fs::remove_file(&tmp);
        format!(
            "Could not move {} to {}: {error}",
            tmp.display(),
            path.display()
        )
    })
}

fn read_prefix_bytes(path: &Path, len: u64) -> Result<Vec<u8>, String> {
    read_range_bytes(path, 0, len)
}

fn read_range_bytes(path: &Path, start: u64, len: u64) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(start))
        .map_err(|error| format!("Could not seek {}: {error}", path.display()))?;
    let mut bytes = vec![0_u8; len as usize];
    file.read_exact(&mut bytes)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    Ok(bytes)
}

fn read_u64_le(bytes: &[u8]) -> u64 {
    let mut array = [0_u8; 8];
    array.copy_from_slice(bytes);
    u64::from_le_bytes(array)
}

fn read_u32_le(bytes: &[u8]) -> u32 {
    let mut array = [0_u8; 4];
    array.copy_from_slice(bytes);
    u32::from_le_bytes(array)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
