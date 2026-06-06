use std::time::{SystemTime as StdSystemTime, UNIX_EPOCH};

fn main() {
    emit_build_date();

    #[cfg(target_os = "windows")]
    embed_windows_resources();
}

fn emit_build_date() {
    println!("cargo:rerun-if-env-changed=YT_DLP_GUI_BUILD_DATE");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    println!(
        "cargo:rustc-env=YT_DLP_GUI_BUILD_DATE={}",
        build_date_string()
    );
}

fn build_date_string() -> String {
    override_build_date()
        .or_else(host_local_date_string)
        .unwrap_or_else(current_utc_date_string)
}

fn override_build_date() -> Option<String> {
    let value = std::env::var("YT_DLP_GUI_BUILD_DATE").ok()?;
    let value = value.trim();
    is_yyyy_mm_dd(value).then(|| value.to_owned())
}

fn is_yyyy_mm_dd(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'.'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'.'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

#[cfg(target_os = "windows")]
fn host_local_date_string() -> Option<String> {
    let mut time = WindowsSystemTime::default();
    unsafe {
        GetLocalTime(&mut time);
    }
    (time.wYear > 0 && (1..=12).contains(&time.wMonth) && (1..=31).contains(&time.wDay))
        .then(|| format!("{:04}.{:02}.{:02}", time.wYear, time.wMonth, time.wDay))
}

#[cfg(not(target_os = "windows"))]
fn host_local_date_string() -> Option<String> {
    None
}

fn current_utc_date_string() -> String {
    let days_since_epoch = StdSystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    let (year, month, day) = civil_date_from_unix_days(days_since_epoch as i64);
    format!("{year:04}.{month:02}.{day:02}")
}

fn civil_date_from_unix_days(days_since_epoch: i64) -> (i64, i64, i64) {
    // Howard Hinnant's civil-from-days algorithm. This keeps build.rs dependency-free while
    // still emitting a stable YYYY.MM.DD build date for the compiled binary.
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(target_os = "windows")]
#[derive(Default)]
#[allow(non_snake_case)]
#[repr(C)]
struct WindowsSystemTime {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(lpSystemTime: *mut WindowsSystemTime);
}

#[cfg(target_os = "windows")]
fn embed_windows_resources() {
    let mut resource = winres::WindowsResource::new();
    resource.set_icon("assets/logo.ico");
    resource
        .compile()
        .expect("failed to embed Windows application icon");
}
