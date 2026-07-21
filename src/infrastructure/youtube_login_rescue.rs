use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::cookie_site_index::{
    CookieSiteIndexEntry, read_cookie_site_index_or_default, write_cookie_site_index,
};
use super::process_guard::{TrackedChildProcess, track_child_process};

const CDP_POLL_ATTEMPTS: usize = 80;
const CDP_POLL_INTERVAL: Duration = Duration::from_millis(250);
const COOKIE_POLL_ATTEMPTS: usize = 300;
const COOKIE_POLL_INTERVAL: Duration = Duration::from_secs(2);
const CDP_WS_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YoutubeLoginRescueBrowserKind {
    Brave,
    Chrome,
    Edge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YoutubeLoginRescueBrowserInfo {
    pub kind: YoutubeLoginRescueBrowserKind,
    pub display_name: String,
    pub exe_path: PathBuf,
}

impl YoutubeLoginRescueBrowserInfo {
    pub fn stable_name(&self) -> &'static str {
        match self.kind {
            YoutubeLoginRescueBrowserKind::Brave => "Brave",
            YoutubeLoginRescueBrowserKind::Chrome => "Chrome",
            YoutubeLoginRescueBrowserKind::Edge => "Microsoft Edge",
        }
    }
}

#[derive(Debug)]
pub struct YoutubeLoginRescueSession {
    pub browser: YoutubeLoginRescueBrowserInfo,
    pub user_data_dir: PathBuf,
    pub remote_debugging_port: u16,
    pub process_id: u32,
    pub cdp_version_url: String,
    pub cdp_websocket_url: Option<String>,
    child: Option<Child>,
    process_guard: Option<TrackedChildProcess>,
}

impl YoutubeLoginRescueSession {
    fn ensure_login_browser_is_still_open(&mut self) -> Result<(), String> {
        let Some(child) = self.child.as_mut() else {
            return Err("Login browser was closed before Cookie could be saved.".to_owned());
        };

        match child.try_wait() {
            Ok(Some(_)) => Err("Login browser was closed before Cookie could be saved.".to_owned()),
            Ok(None) => Ok(()),
            Err(error) => Err(format!(
                "Could not inspect login browser while waiting for Cookie: {error}"
            )),
        }
    }

    pub fn close(&mut self) -> Result<(), String> {
        let mut close_error = None;

        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) => {
                    if let Some(ws_url) = self.cdp_websocket_url.as_deref() {
                        let _ = cdp_call(ws_url, "Browser.close", None);
                        for _ in 0..20 {
                            match child.try_wait() {
                                Ok(Some(_)) => break,
                                Ok(None) => thread::sleep(Duration::from_millis(100)),
                                Err(error) => {
                                    close_error = Some(format!(
                                        "Could not inspect login browser process: {error}"
                                    ));
                                    break;
                                }
                            }
                        }
                    }

                    match child.try_wait() {
                        Ok(Some(_)) => {}
                        Ok(None) => {
                            if let Err(error) = child.kill() {
                                close_error =
                                    Some(format!("Could not close login browser: {error}"));
                            } else {
                                let _ = child.wait();
                            }
                        }
                        Err(error) => {
                            close_error =
                                Some(format!("Could not inspect login browser process: {error}"));
                        }
                    }
                }
                Err(error) => {
                    close_error = Some(format!("Could not inspect login browser process: {error}"));
                }
            }
        }
        self.child = None;
        self.process_guard = None;

        if self.user_data_dir.exists() {
            let _ = fs::remove_dir_all(&self.user_data_dir);
        }

        if let Some(error) = close_error {
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl Drop for YoutubeLoginRescueSession {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[derive(Clone, Debug)]
pub struct YoutubeLoginRescueCookieExport {
    pub browser: YoutubeLoginRescueBrowserInfo,
    pub site_display_name: String,
    pub site_id: String,
    pub cookie_file_path: PathBuf,
    pub exported_cookie_count: usize,
    pub auth_cookie_count: usize,
}

#[derive(Debug)]
pub enum YoutubeLoginRescueEvent {
    CdpReady(YoutubeLoginRescueBrowserInfo),
    CookieExported(YoutubeLoginRescueCookieExport),
    Failed(String),
}

#[derive(Clone, Debug)]
struct CookieRescueSite {
    display_name: String,
    site_id: String,
    root_domain: String,
    is_youtube: bool,
}

#[derive(Clone, Debug)]
struct CdpPageTarget {
    ws_url: String,
    url: String,
}

#[derive(Clone, Debug)]
struct CdpCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    expires: Option<f64>,
    secure: bool,
    http_only: bool,
}

pub fn detect_default_youtube_login_rescue_browser()
-> Result<Option<YoutubeLoginRescueBrowserInfo>, String> {
    detect_default_supported_browser()
}

pub fn launch_youtube_login_rescue_session(
    browser: YoutubeLoginRescueBrowserInfo,
    profile_root_path: PathBuf,
    target_url: &str,
) -> Result<YoutubeLoginRescueSession, String> {
    if !browser.exe_path.is_file() {
        return Err(format!(
            "{} was detected, but the executable was not found: {}",
            browser.display_name,
            browser.exe_path.display()
        ));
    }

    let port = find_free_local_port()?;
    let user_data_dir = create_temp_profile_dir(&profile_root_path)?;
    let version_url = format!("http://127.0.0.1:{port}/json/version");

    let mut command = Command::new(&browser.exe_path);
    command
        .arg(format!("--user-data-dir={}", user_data_dir.display()))
        .arg("--remote-debugging-address=127.0.0.1")
        .arg(format!("--remote-debugging-port={port}"))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--new-window")
        .arg(target_url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = command.spawn().map_err(|error| {
        let _ = fs::remove_dir_all(&user_data_dir);
        format!("Could not launch {}: {error}", browser.display_name)
    })?;
    let process_id = child.id();
    let process_guard = track_child_process(
        &child,
        format!("cookie rescue browser {}", browser.display_name),
    );

    match wait_for_cdp_version(&version_url) {
        Ok((cdp_version_url, cdp_websocket_url)) => Ok(YoutubeLoginRescueSession {
            browser,
            user_data_dir,
            remote_debugging_port: port,
            process_id,
            cdp_version_url,
            cdp_websocket_url,
            child: Some(child),
            process_guard: Some(process_guard),
        }),
        Err(error) => {
            let mut session = YoutubeLoginRescueSession {
                browser,
                user_data_dir,
                remote_debugging_port: port,
                process_id,
                cdp_version_url: version_url,
                cdp_websocket_url: None,
                child: Some(child),
                process_guard: Some(process_guard),
            };
            let _ = session.close();
            Err(error)
        }
    }
}

pub fn run_youtube_login_rescue_cookie_export(
    browser: YoutubeLoginRescueBrowserInfo,
    target_url: String,
    cookie_dir_path: PathBuf,
    profile_root_path: PathBuf,
    tx: Sender<YoutubeLoginRescueEvent>,
) {
    let result = run_youtube_login_rescue_cookie_export_inner(
        browser,
        target_url,
        cookie_dir_path,
        profile_root_path,
        &tx,
    );
    if let Err(error) = result {
        let _ = tx.send(YoutubeLoginRescueEvent::Failed(error));
    }
}

fn run_youtube_login_rescue_cookie_export_inner(
    browser: YoutubeLoginRescueBrowserInfo,
    target_url: String,
    cookie_dir_path: PathBuf,
    profile_root_path: PathBuf,
    tx: &Sender<YoutubeLoginRescueEvent>,
) -> Result<(), String> {
    let normalized_target_url = normalize_cookie_rescue_target_url(&target_url)?;
    let target_site = site_from_url(&normalized_target_url)
        .ok_or_else(|| "Cookie Rescue needs an http:// or https:// website URL.".to_owned())?;
    let mut session =
        launch_youtube_login_rescue_session(browser, profile_root_path, &normalized_target_url)?;
    let browser_info = session.browser.clone();
    let _ = tx.send(YoutubeLoginRescueEvent::CdpReady(browser_info.clone()));

    let cookies = wait_for_cookie_rescue_cookies(&mut session, &target_site)?;
    let site = target_site;
    let filtered = filter_cookies_for_site(cookies, &site);
    let auth_cookie_count = filtered
        .iter()
        .filter(|cookie| is_cookie_rescue_auth_cookie(cookie, &site))
        .count();
    if filtered.is_empty() {
        return Err(format!(
            "No cookies were found for {} after confirmation.",
            site.display_name
        ));
    }

    let cookie_file_path = cookie_dir_path.join(format!("{}.cookies.txt", site.site_id));
    write_netscape_cookie_file(&cookie_file_path, &filtered, &site)?;
    write_cookie_rescue_site_index(
        &cookie_dir_path,
        &site,
        &normalized_target_url,
        &cookie_file_path,
    )?;
    let exported_cookie_count = filtered.len();
    let _ = session.close();

    let _ = tx.send(YoutubeLoginRescueEvent::CookieExported(
        YoutubeLoginRescueCookieExport {
            browser: browser_info,
            site_display_name: site.display_name,
            site_id: site.site_id,
            cookie_file_path,
            exported_cookie_count,
            auth_cookie_count,
        },
    ));

    Ok(())
}

fn wait_for_cookie_rescue_cookies(
    session: &mut YoutubeLoginRescueSession,
    target_site: &CookieRescueSite,
) -> Result<Vec<CdpCookie>, String> {
    let mut last_error = String::new();

    for _ in 0..COOKIE_POLL_ATTEMPTS {
        session.ensure_login_browser_is_still_open()?;
        let mut latest_cookies = None;

        match read_all_browser_cookies(session) {
            Ok(cookies) => {
                let filtered = filter_cookies_for_site(cookies.clone(), target_site);
                let auth_cookie_count = filtered
                    .iter()
                    .filter(|cookie| is_cookie_rescue_auth_cookie(cookie, target_site))
                    .count();
                latest_cookies = Some(cookies);

                if cookie_rescue_site_supports_auto_detection(target_site) {
                    if auth_cookie_count > 0 {
                        return latest_cookies.ok_or_else(|| {
                            "Cookie Rescue could not read cookies after auto detection.".to_owned()
                        });
                    }

                    if target_site.is_youtube {
                        last_error = "Waiting for YouTube login cookies.".to_owned();
                    } else if filtered.is_empty() {
                        last_error =
                            format!("Waiting for cookies from {}.", target_site.display_name);
                    } else {
                        last_error = format!(
                            "Cookies were found for {}, but no known login cookie was detected yet.",
                            target_site.display_name
                        );
                    }
                } else if filtered.is_empty() {
                    last_error = format!(
                        "Waiting for cookies from {} or manual confirmation.",
                        target_site.display_name
                    );
                } else {
                    last_error = format!(
                        "Cookies were found for {}. Waiting for manual confirmation.",
                        target_site.display_name
                    );
                }
            }
            Err(error) => {
                session.ensure_login_browser_is_still_open()?;
                if is_closed_login_browser_page_error(&error) {
                    return Err("Login browser was closed before Cookie could be saved.".to_owned());
                }
                last_error = error;
            }
        }

        match cookie_rescue_confirmation_was_clicked(session) {
            Ok(true) => {
                if let Some(cookies) = latest_cookies {
                    return Ok(cookies);
                }
                return read_all_browser_cookies(session);
            }
            Ok(false) => {}
            Err(error) => {
                session.ensure_login_browser_is_still_open()?;
                if is_closed_login_browser_page_error(&error) {
                    return Err("Login browser was closed before Cookie could be saved.".to_owned());
                }
                if last_error.trim().is_empty() {
                    last_error = error;
                }
            }
        }

        thread::sleep(COOKIE_POLL_INTERVAL);
    }

    if last_error.trim().is_empty() {
        Err(format!(
            "Timed out waiting for login cookies or confirmation from {}.",
            target_site.display_name
        ))
    } else {
        Err(format!(
            "Timed out waiting for login cookies or confirmation from {}. Last status: {last_error}",
            target_site.display_name
        ))
    }
}

fn cookie_rescue_confirmation_was_clicked(
    session: &YoutubeLoginRescueSession,
) -> Result<bool, String> {
    let targets = read_page_targets(session.remote_debugging_port)?;
    let mut last_error = String::new();
    let mut saw_page = false;

    for target in targets {
        if !target.url.starts_with("http://") && !target.url.starts_with("https://") {
            continue;
        }
        saw_page = true;
        match inject_cookie_rescue_confirmation_button(&target.ws_url) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(error) => last_error = error,
        }
    }

    if saw_page {
        Ok(false)
    } else if last_error.trim().is_empty() {
        Err("No website page is available in the login browser yet.".to_owned())
    } else {
        Err(last_error)
    }
}

fn inject_cookie_rescue_confirmation_button(ws_url: &str) -> Result<bool, String> {
    let response = cdp_call(
        ws_url,
        "Runtime.evaluate",
        Some(json!({
            "expression": cookie_rescue_confirmation_button_script(),
            "awaitPromise": false,
            "returnByValue": true
        })),
    )?;

    Ok(response
        .get("result")
        .and_then(|result| result.get("result"))
        .and_then(|result| result.get("value"))
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn cookie_rescue_confirmation_button_script() -> &'static str {
    r#"(() => {
  const rootId = '__yt_dlp_gui_cookie_rescue_confirm_root';
  const flag = '__ytDlpGuiCookieRescueConfirmed';
  const buttonText = '我已完成登入，儲存 cookies';
  const clickedText = '正在儲存 cookies...';

  if (!document || !document.documentElement || !document.body) {
    return !!window[flag];
  }

  let root = document.getElementById(rootId);
  if (!root) {
    root = document.createElement('div');
    root.id = rootId;
    root.setAttribute('data-yt-dlp-gui-cookie-rescue', 'true');
    root.style.cssText = [
      'position:fixed',
      'top:0',
      'left:0',
      'right:0',
      'z-index:2147483647',
      'display:flex',
      'justify-content:center',
      'align-items:center',
      'gap:10px',
      'padding:10px 12px',
      'box-sizing:border-box',
      'background:rgba(17,24,39,.96)',
      'color:white',
      'font:14px system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif',
      'box-shadow:0 8px 24px rgba(0,0,0,.25)',
      'pointer-events:auto'
    ].join(';');

    const note = document.createElement('span');
    note.textContent = 'yt-dlp-gui Cookie Rescue';
    note.style.cssText = 'opacity:.86;white-space:nowrap';

    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = buttonText;
    button.style.cssText = [
      'appearance:none',
      'border:0',
      'border-radius:999px',
      'padding:8px 14px',
      'font:600 14px system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif',
      'background:white',
      'color:#111827',
      'cursor:pointer',
      'box-shadow:0 2px 8px rgba(0,0,0,.2)'
    ].join(';');
    button.addEventListener('click', () => {
      window[flag] = true;
      button.disabled = true;
      button.textContent = clickedText;
      button.style.cursor = 'default';
      button.style.opacity = '.72';
    });

    root.appendChild(note);
    root.appendChild(button);
    document.documentElement.appendChild(root);
  }

  return !!window[flag];
})()"#
}

fn is_closed_login_browser_page_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("could not find a cdp page target")
        || normalized.contains("no cdp page target")
        || normalized.contains("cdp websocket was closed")
}

fn read_all_browser_cookies(session: &YoutubeLoginRescueSession) -> Result<Vec<CdpCookie>, String> {
    if let Some(ws_url) = session.cdp_websocket_url.as_deref() {
        if let Ok(cookies) = read_cookies_via_storage(ws_url) {
            return Ok(cookies);
        }
    }

    let target_ws_urls = read_page_websocket_urls(session.remote_debugging_port)?;
    let mut last_error = String::new();
    for ws_url in target_ws_urls {
        match read_cookies_via_network(&ws_url).or_else(|_| read_cookies_via_storage(&ws_url)) {
            Ok(cookies) => return Ok(cookies),
            Err(error) => last_error = error,
        }
    }

    if last_error.is_empty() {
        Err("Could not find a CDP page target for the login browser.".to_owned())
    } else {
        Err(last_error)
    }
}

fn read_cookies_via_storage(ws_url: &str) -> Result<Vec<CdpCookie>, String> {
    let response = cdp_call(ws_url, "Storage.getCookies", Some(json!({})))?;
    parse_cookies_from_result(&response)
}

fn read_cookies_via_network(ws_url: &str) -> Result<Vec<CdpCookie>, String> {
    let _ = cdp_call(ws_url, "Network.enable", Some(json!({})));
    let response = cdp_call(ws_url, "Network.getAllCookies", Some(json!({})))?;
    parse_cookies_from_result(&response)
}

fn parse_cookies_from_result(response: &Value) -> Result<Vec<CdpCookie>, String> {
    let cookies = response
        .get("result")
        .and_then(|result| result.get("cookies"))
        .and_then(Value::as_array)
        .ok_or_else(|| "CDP cookie response did not contain a cookies array.".to_owned())?;

    Ok(cookies.iter().filter_map(parse_cdp_cookie).collect())
}

fn parse_cdp_cookie(value: &Value) -> Option<CdpCookie> {
    let name = value.get("name")?.as_str()?.to_owned();
    let domain = value.get("domain")?.as_str()?.to_owned();
    let cookie_value = value
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let path = value
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("/")
        .to_owned();
    let expires = value
        .get("expires")
        .and_then(Value::as_f64)
        .filter(|value| *value > 0.0);
    let secure = value
        .get("secure")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let http_only = value
        .get("httpOnly")
        .or_else(|| value.get("httponly"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Some(CdpCookie {
        name,
        value: cookie_value,
        domain,
        path,
        expires,
        secure,
        http_only,
    })
}

fn install_cookie_rescue_start_page(session: &YoutubeLoginRescueSession) -> Result<(), String> {
    let target = wait_for_first_page_target(session.remote_debugging_port)?;
    let html = cookie_rescue_start_page_html(&session.browser.display_name);
    let html_literal = serde_json::to_string(&html)
        .map_err(|error| format!("Could not encode Cookie Rescue start page: {error}"))?;
    let expression = format!("document.open();document.write({html_literal});document.close();");
    cdp_call(
        &target.ws_url,
        "Runtime.evaluate",
        Some(json!({
            "expression": expression,
            "awaitPromise": false,
            "returnByValue": true
        })),
    )?;
    Ok(())
}

fn cookie_rescue_start_page_html(browser_name: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>yt-dlp-gui Cookie Rescue</title>
<style>
:root {{ color-scheme: light dark; }}
body {{
  margin: 0;
  min-height: 100vh;
  display: grid;
  place-items: center;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: Canvas;
  color: CanvasText;
}}
main {{
  width: min(680px, calc(100vw - 48px));
  padding: 32px;
  border: 1px solid color-mix(in srgb, CanvasText 16%, transparent);
  border-radius: 18px;
  box-shadow: 0 20px 60px color-mix(in srgb, CanvasText 12%, transparent);
}}
h1 {{ margin: 0 0 12px; font-size: 24px; }}
p {{ margin: 10px 0; line-height: 1.6; }}
.note {{ opacity: .74; }}
kbd {{
  padding: 2px 6px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, CanvasText 20%, transparent);
  background: color-mix(in srgb, CanvasText 6%, transparent);
}}
</style>
</head>
<body>
<main>
  <h1>yt-dlp-gui Cookie Rescue</h1>
  <p>Please type the website that needs login in the browser address bar, then sign in there.</p>
  <p>For supported mainstream sites, cookies may be saved automatically after known login cookies are detected.</p>
  <p>For other sites, finish signing in and press the in-page button to save cookies in this dedicated {browser_name} window.</p>
  <p class="note">This is a dedicated temporary login environment. It does not read your regular browser profile.</p>
  <p class="note">You can use <kbd>Ctrl</kbd> + <kbd>L</kbd> to focus the address bar.</p>
</main>
</body>
</html>"#
    )
}

fn detect_cookie_rescue_site(
    session: &YoutubeLoginRescueSession,
) -> Result<Option<CookieRescueSite>, String> {
    let targets = read_page_targets(session.remote_debugging_port)?;
    for target in targets.iter().rev().chain(targets.iter()) {
        if let Some(site) = site_from_url(&target.url) {
            return Ok(Some(site));
        }
    }
    Ok(None)
}

fn site_from_url(url: &str) -> Option<CookieRescueSite> {
    let host = host_from_http_url(url)?;
    let root_domain = registrable_domain_guess(&host)?;
    let is_youtube = root_domain == "youtube.com" || root_domain == "youtu.be";
    let canonical_root = if is_youtube {
        "youtube.com".to_owned()
    } else {
        root_domain
    };
    let site_id = site_id_from_domain(&canonical_root);
    let display_name = if is_youtube {
        "YouTube".to_owned()
    } else {
        canonical_root.clone()
    };
    Some(CookieRescueSite {
        display_name,
        site_id,
        root_domain: canonical_root,
        is_youtube,
    })
}

pub fn normalize_cookie_rescue_target_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Please enter a website URL.".to_owned());
    }

    let lower = trimmed.to_ascii_lowercase();
    for blocked in [
        "file:",
        "javascript:",
        "data:",
        "about:",
        "chrome:",
        "edge:",
        "brave:",
    ] {
        if lower.starts_with(blocked) {
            return Err("Cookie Rescue only accepts http:// or https:// website URLs.".to_owned());
        }
    }

    let candidate = if lower.starts_with("https://") {
        format!("https://{}", &trimmed[8..])
    } else if lower.starts_with("http://") {
        format!("http://{}", &trimmed[7..])
    } else {
        format!("https://{trimmed}")
    };

    let host = host_from_http_url(&candidate)
        .ok_or_else(|| "Cookie Rescue needs a valid website URL.".to_owned())?;
    if !is_valid_cookie_rescue_host(&host) {
        return Err("Cookie Rescue needs a normal website domain.".to_owned());
    }

    Ok(candidate)
}

fn host_from_http_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let rest = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?.trim();
    if authority.is_empty() {
        return None;
    }
    let host = authority
        .rsplit('@')
        .next()
        .unwrap_or(authority)
        .split(':')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host.is_empty() || host == "localhost" || host == "127.0.0.1" {
        return None;
    }
    Some(host)
}

fn is_valid_cookie_rescue_host(host: &str) -> bool {
    if host.is_empty()
        || !host.contains('.')
        || host.len() > 253
        || host.contains(' ')
        || host.contains('\\')
        || host.contains('/')
    {
        return false;
    }

    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    })
}

fn registrable_domain_guess(host: &str) -> Option<String> {
    let labels = host
        .trim_matches('.')
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if labels.len() < 2 {
        return None;
    }

    // Lightweight eTLD+1 guess for common multi-label public suffixes.
    let suffix2 = format!("{}.{}", labels[labels.len() - 2], labels[labels.len() - 1]);
    let common_two_level_suffix = matches!(
        suffix2.as_str(),
        "co.jp"
            | "com.au"
            | "com.br"
            | "com.cn"
            | "com.hk"
            | "com.sg"
            | "co.uk"
            | "org.uk"
            | "net.au"
    );
    let take = if common_two_level_suffix && labels.len() >= 3 {
        3
    } else {
        2
    };
    Some(labels[labels.len() - take..].join("."))
}

fn site_id_from_domain(domain: &str) -> String {
    let mut id = domain
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    while id.contains("__") {
        id = id.replace("__", "_");
    }
    id.trim_matches('_').to_owned()
}

fn filter_cookies_for_site(cookies: Vec<CdpCookie>, site: &CookieRescueSite) -> Vec<CdpCookie> {
    let mut filtered = cookies
        .into_iter()
        .filter(|cookie| cookie_matches_site(cookie, site))
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then(left.path.cmp(&right.path))
            .then(left.name.cmp(&right.name))
    });
    filtered.dedup_by(|left, right| {
        left.domain == right.domain && left.path == right.path && left.name == right.name
    });
    filtered
}

fn cookie_matches_site(cookie: &CdpCookie, site: &CookieRescueSite) -> bool {
    let normalized = cookie.domain.trim_start_matches('.').to_ascii_lowercase();
    if normalized == site.root_domain || normalized.ends_with(&format!(".{}", site.root_domain)) {
        return true;
    }
    site.is_youtube && is_youtube_google_domain(&normalized)
}

fn is_youtube_google_domain(domain: &str) -> bool {
    let normalized = domain.trim_start_matches('.').to_ascii_lowercase();
    normalized == "youtube.com"
        || normalized.ends_with(".youtube.com")
        || normalized == "youtu.be"
        || normalized.ends_with(".youtu.be")
        || normalized == "google.com"
        || normalized.ends_with(".google.com")
        || normalized == "accounts.google.com"
}

fn cookie_rescue_site_supports_auto_detection(site: &CookieRescueSite) -> bool {
    site.is_youtube
        || matches!(
            site.root_domain.as_str(),
            "instagram.com"
                | "facebook.com"
                | "twitter.com"
                | "x.com"
                | "tiktok.com"
                | "reddit.com"
        )
}

fn is_cookie_rescue_auth_cookie(cookie: &CdpCookie, site: &CookieRescueSite) -> bool {
    if site.is_youtube {
        return is_youtube_auth_cookie(cookie);
    }

    if cookie.value.trim().is_empty() {
        return false;
    }

    let name = cookie.name.to_ascii_lowercase();
    match site.root_domain.as_str() {
        // Instagram sets visitor and CSRF cookies before login.  In particular,
        // `csrftoken` used to match the old generic `token` substring and made
        // Cookie Rescue close before the user actually signed in.
        "instagram.com" => matches!(name.as_str(), "sessionid" | "ds_user_id"),
        "facebook.com" => matches!(name.as_str(), "c_user" | "xs"),
        "twitter.com" | "x.com" => matches!(name.as_str(), "auth_token" | "twid"),
        "tiktok.com" => matches!(
            name.as_str(),
            "sessionid" | "sid_tt" | "uid_tt" | "sid_guard" | "passport_csrf_token"
        ),
        "reddit.com" => matches!(name.as_str(), "reddit_session" | "token_v2"),
        _ => false,
    }
}

fn is_youtube_auth_cookie(cookie: &CdpCookie) -> bool {
    let name = cookie.name.as_str();
    matches!(
        name,
        "SID"
            | "HSID"
            | "SSID"
            | "APISID"
            | "SAPISID"
            | "SIDCC"
            | "LOGIN_INFO"
            | "__Secure-1PSID"
            | "__Secure-3PSID"
            | "__Secure-1PAPISID"
            | "__Secure-3PAPISID"
            | "__Secure-1PSIDCC"
            | "__Secure-3PSIDCC"
    )
}

fn write_cookie_rescue_site_index(
    cookie_dir_path: &Path,
    site: &CookieRescueSite,
    target_url: &str,
    cookie_file_path: &Path,
) -> Result<(), String> {
    fs::create_dir_all(cookie_dir_path)
        .map_err(|error| format!("Could not create cookie directory: {error}"))?;
    let cookie_file = cookie_file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();

    let mut index = read_cookie_site_index_or_default(cookie_dir_path);
    let entry = CookieSiteIndexEntry {
        id: site.site_id.to_owned(),
        display_name: site.display_name.to_owned(),
        login_url: target_url.to_owned(),
        match_domains: cookie_match_domains_for_site(site),
        cookie_domains: cookie_domains_for_site(site),
        cookie_file,
        builtin: site.is_youtube,
        updated_unix: current_unix_timestamp(),
    };

    if let Some(existing) = index.sites.iter_mut().find(|item| item.id == site.site_id) {
        *existing = entry;
    } else {
        index.sites.push(entry);
    }
    index.sites.sort_by(|left, right| left.id.cmp(&right.id));

    write_cookie_site_index(cookie_dir_path, &index)
}

fn cookie_match_domains_for_site(site: &CookieRescueSite) -> Vec<String> {
    if site.is_youtube {
        return vec!["youtube.com".to_owned(), "youtu.be".to_owned()];
    }

    vec![site.root_domain.clone()]
}

fn cookie_domains_for_site(site: &CookieRescueSite) -> Vec<String> {
    if site.is_youtube {
        return vec![
            "youtube.com".to_owned(),
            ".youtube.com".to_owned(),
            "youtu.be".to_owned(),
            ".youtu.be".to_owned(),
            "google.com".to_owned(),
            ".google.com".to_owned(),
            "accounts.google.com".to_owned(),
        ];
    }

    vec![site.root_domain.clone(), format!(".{}", site.root_domain)]
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn write_netscape_cookie_file(
    path: &Path,
    cookies: &[CdpCookie],
    site: &CookieRescueSite,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create cookie directory: {error}"))?;
    }

    let mut output = String::new();
    output.push_str("# Netscape HTTP Cookie File\n");
    output.push_str("# Generated by yt-dlp-gui Cookie Rescue. Do not share this file.\n");
    output.push_str(&format!(
        "# This file contains cookies for {} and is used by yt-dlp.\n\n",
        site.display_name
    ));

    for cookie in cookies {
        let mut domain = cookie.domain.trim().to_owned();
        if domain.is_empty() || cookie.name.is_empty() {
            continue;
        }
        let include_subdomains = if domain.starts_with('.') {
            "TRUE"
        } else {
            "FALSE"
        };
        if cookie.http_only && !domain.starts_with("#HttpOnly_") {
            domain = format!("#HttpOnly_{domain}");
        }
        let expires = cookie.expires.map(|value| value as i64).unwrap_or(0).max(0);
        let secure = if cookie.secure { "TRUE" } else { "FALSE" };
        let path = if cookie.path.trim().is_empty() {
            "/"
        } else {
            cookie.path.as_str()
        };
        output.push_str(&format!(
            "{domain}\t{include_subdomains}\t{path}\t{secure}\t{expires}\t{}\t{}\n",
            cookie.name, cookie.value
        ));
    }

    fs::write(path, output).map_err(|error| format!("Could not write cookies.txt: {error}"))
}

fn wait_for_first_page_target(port: u16) -> Result<CdpPageTarget, String> {
    let mut last_error = String::new();
    for _ in 0..CDP_POLL_ATTEMPTS {
        match read_page_targets(port) {
            Ok(targets) => {
                if let Some(target) = targets.into_iter().next() {
                    return Ok(target);
                }
                last_error = "No CDP page target is available yet.".to_owned();
            }
            Err(error) => last_error = error,
        }
        thread::sleep(CDP_POLL_INTERVAL);
    }
    Err(format!(
        "Could not find a CDP page target for the login browser. Last error: {last_error}"
    ))
}

fn read_page_websocket_urls(port: u16) -> Result<Vec<String>, String> {
    Ok(read_page_targets(port)?
        .into_iter()
        .map(|target| target.ws_url)
        .collect())
}

fn read_page_targets(port: u16) -> Result<Vec<CdpPageTarget>, String> {
    let list_url = format!("http://127.0.0.1:{port}/json/list");
    let mut response = ureq::get(&list_url)
        .call()
        .map_err(|error| format!("Could not read CDP target list: {error}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("CDP target list returned HTTP {status}"));
    }

    let mut body = String::new();
    response
        .body_mut()
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| format!("Could not read CDP target list response: {error}"))?;

    let json: Value = serde_json::from_str(&body)
        .map_err(|error| format!("Could not parse CDP target list response: {error}"))?;
    let Some(targets) = json.as_array() else {
        return Err("CDP target list was not an array.".to_owned());
    };

    let mut page_targets = Vec::new();
    for target in targets {
        let target_type = target
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if target_type != "page" {
            continue;
        }
        let Some(ws_url) = target
            .get("webSocketDebuggerUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
        else {
            continue;
        };
        let url = target
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned();
        page_targets.push(CdpPageTarget { ws_url, url });
    }
    Ok(page_targets)
}

fn cdp_call(ws_url: &str, method: &str, params: Option<Value>) -> Result<Value, String> {
    let endpoint = parse_ws_url(ws_url)?;
    let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
        .map_err(|error| format!("Could not connect to CDP websocket: {error}"))?;
    stream
        .set_read_timeout(Some(CDP_WS_TIMEOUT))
        .map_err(|error| format!("Could not set CDP read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(CDP_WS_TIMEOUT))
        .map_err(|error| format!("Could not set CDP write timeout: {error}"))?;

    websocket_handshake(&mut stream, &endpoint)?;

    let mut request = json!({
        "id": 1,
        "method": method,
    });
    if let Some(params) = params {
        request["params"] = params;
    }
    send_ws_frame(&mut stream, 0x1, request.to_string().as_bytes(), true)?;

    for _ in 0..64 {
        let message = read_ws_text_message(&mut stream)?;
        let response: Value = serde_json::from_str(&message)
            .map_err(|error| format!("Could not parse CDP websocket response: {error}"))?;
        if response.get("id").and_then(Value::as_u64) != Some(1) {
            continue;
        }
        if let Some(error) = response.get("error") {
            return Err(format!("CDP {method} failed: {error}"));
        }
        return Ok(response);
    }

    Err(format!("CDP {method} did not return a matching response."))
}

#[derive(Clone, Debug)]
struct WsEndpoint {
    host: String,
    port: u16,
    path: String,
}

fn parse_ws_url(ws_url: &str) -> Result<WsEndpoint, String> {
    let rest = ws_url
        .strip_prefix("ws://")
        .ok_or_else(|| format!("Only local ws:// CDP endpoints are supported: {ws_url}"))?;
    let (authority, path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (rest, "/".to_owned()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|error| format!("Invalid CDP websocket port: {error}"))?;
            (host.to_owned(), port)
        }
        None => (authority.to_owned(), 80),
    };
    if host != "127.0.0.1" && host != "localhost" {
        return Err(format!("Refusing non-local CDP websocket host: {host}"));
    }
    Ok(WsEndpoint { host, port, path })
}

fn websocket_handshake(stream: &mut TcpStream, endpoint: &WsEndpoint) -> Result<(), String> {
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n",
        endpoint.path, endpoint.host, endpoint.port
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("Could not send CDP websocket handshake: {error}"))?;

    let mut response = Vec::new();
    let mut byte = [0_u8; 1];
    while response.len() < 16 * 1024 {
        stream
            .read_exact(&mut byte)
            .map_err(|error| format!("Could not read CDP websocket handshake: {error}"))?;
        response.push(byte[0]);
        if response.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let response_text = String::from_utf8_lossy(&response);
    if !response_text.starts_with("HTTP/1.1 101") && !response_text.starts_with("HTTP/1.0 101") {
        return Err(format!(
            "CDP websocket handshake failed: {}",
            response_text.lines().next().unwrap_or("unknown response")
        ));
    }
    Ok(())
}

fn send_ws_frame(
    stream: &mut TcpStream,
    opcode: u8,
    payload: &[u8],
    masked: bool,
) -> Result<(), String> {
    let mut frame = Vec::new();
    frame.push(0x80 | (opcode & 0x0f));
    let mask_bit = if masked { 0x80 } else { 0x00 };
    if payload.len() <= 125 {
        frame.push(mask_bit | payload.len() as u8);
    } else if payload.len() <= u16::MAX as usize {
        frame.push(mask_bit | 126);
        frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        frame.push(mask_bit | 127);
        frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }

    if masked {
        let mask = [0x12, 0x34, 0x56, 0x78];
        frame.extend_from_slice(&mask);
        for (index, byte) in payload.iter().enumerate() {
            frame.push(byte ^ mask[index % 4]);
        }
    } else {
        frame.extend_from_slice(payload);
    }

    stream
        .write_all(&frame)
        .map_err(|error| format!("Could not send CDP websocket frame: {error}"))
}

fn read_ws_text_message(stream: &mut TcpStream) -> Result<String, String> {
    let mut message = Vec::new();
    let mut continuation_active = false;

    loop {
        let mut header = [0_u8; 2];
        stream
            .read_exact(&mut header)
            .map_err(|error| format!("Could not read CDP websocket frame: {error}"))?;
        let fin = header[0] & 0x80 != 0;
        let opcode = header[0] & 0x0f;
        let masked = header[1] & 0x80 != 0;
        let mut len = (header[1] & 0x7f) as u64;
        if len == 126 {
            let mut buf = [0_u8; 2];
            stream
                .read_exact(&mut buf)
                .map_err(|error| format!("Could not read CDP websocket frame length: {error}"))?;
            len = u16::from_be_bytes(buf) as u64;
        } else if len == 127 {
            let mut buf = [0_u8; 8];
            stream
                .read_exact(&mut buf)
                .map_err(|error| format!("Could not read CDP websocket frame length: {error}"))?;
            len = u64::from_be_bytes(buf);
        }

        let mut mask = [0_u8; 4];
        if masked {
            stream
                .read_exact(&mut mask)
                .map_err(|error| format!("Could not read CDP websocket frame mask: {error}"))?;
        }

        let mut payload = vec![0_u8; len as usize];
        stream
            .read_exact(&mut payload)
            .map_err(|error| format!("Could not read CDP websocket frame payload: {error}"))?;
        if masked {
            for (index, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[index % 4];
            }
        }

        match opcode {
            0x0 => {
                if continuation_active {
                    message.extend_from_slice(&payload);
                    if fin {
                        return String::from_utf8(message).map_err(|error| {
                            format!("CDP websocket message was not UTF-8: {error}")
                        });
                    }
                }
            }
            0x1 => {
                message.extend_from_slice(&payload);
                if fin {
                    return String::from_utf8(message)
                        .map_err(|error| format!("CDP websocket message was not UTF-8: {error}"));
                }
                continuation_active = true;
            }
            0x8 => return Err("CDP websocket was closed.".to_owned()),
            0x9 => {
                let _ = send_ws_frame(stream, 0xA, &payload, true);
            }
            0xA => {}
            _ => {}
        }
    }
}

fn find_free_local_port() -> Result<u16, String> {
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .map_err(|error| format!("Could not reserve a local CDP port: {error}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|error| format!("Could not read the reserved CDP port: {error}"))
}

fn create_temp_profile_dir(profile_root_path: &Path) -> Result<PathBuf, String> {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let dir = profile_root_path.join(format!("session-{pid}-{nanos}"));
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create a temporary Cookie Rescue profile: {error}"))?;
    Ok(dir)
}

fn wait_for_cdp_version(version_url: &str) -> Result<(String, Option<String>), String> {
    let mut last_error = String::new();
    for _ in 0..CDP_POLL_ATTEMPTS {
        match read_cdp_version(version_url) {
            Ok(websocket_url) => return Ok((version_url.to_owned(), websocket_url)),
            Err(error) => last_error = error,
        }
        thread::sleep(CDP_POLL_INTERVAL);
    }

    if last_error.trim().is_empty() {
        Err("The login browser did not expose the local CDP endpoint in time.".to_owned())
    } else {
        Err(format!(
            "The login browser did not expose the local CDP endpoint in time. Last error: {last_error}"
        ))
    }
}

fn read_cdp_version(version_url: &str) -> Result<Option<String>, String> {
    let mut response = ureq::get(version_url)
        .call()
        .map_err(|error| format!("CDP endpoint is not ready: {error}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("CDP endpoint returned HTTP {status}"));
    }

    let mut body = String::new();
    response
        .body_mut()
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| format!("Could not read CDP version response: {error}"))?;

    let json: Value = serde_json::from_str(&body)
        .map_err(|error| format!("Could not parse CDP version response: {error}"))?;
    let websocket_url = json
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Ok(websocket_url)
}

#[cfg(test)]
mod tests {
    use super::is_closed_login_browser_page_error;

    #[test]
    fn cookie_rescue_closed_browser_errors_are_terminal() {
        assert!(is_closed_login_browser_page_error(
            "Could not find a CDP page target for the login browser."
        ));
        assert!(is_closed_login_browser_page_error(
            "CDP websocket was closed."
        ));
        assert!(!is_closed_login_browser_page_error(
            "Waiting for YouTube login cookies."
        ));
    }
}

#[cfg(target_os = "windows")]
fn detect_default_supported_browser() -> Result<Option<YoutubeLoginRescueBrowserInfo>, String> {
    let Some(prog_id) = windows_default_https_prog_id()? else {
        return Ok(first_installed_supported_browser());
    };
    let Some(kind) = browser_kind_from_prog_id(&prog_id) else {
        return Ok(first_installed_supported_browser());
    };

    match browser_info_for_kind(kind) {
        Some(info) => Ok(Some(info)),
        None => Ok(first_installed_supported_browser()),
    }
}

#[cfg(not(target_os = "windows"))]
fn detect_default_supported_browser() -> Result<Option<YoutubeLoginRescueBrowserInfo>, String> {
    Ok(first_installed_supported_browser())
}

#[cfg(target_os = "windows")]
fn windows_default_https_prog_id() -> Result<Option<String>, String> {
    let mut command = Command::new("reg");
    let output = command
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice",
            "/v",
            "ProgId",
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("Could not query the Windows default browser: {error}"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.to_ascii_lowercase().starts_with("progid") {
            continue;
        }
        let parts = trimmed.split_whitespace().collect::<Vec<_>>();
        if let Some(value) = parts.last().filter(|value| !value.trim().is_empty()) {
            return Ok(Some((*value).to_owned()));
        }
    }
    Ok(None)
}

fn browser_kind_from_prog_id(prog_id: &str) -> Option<YoutubeLoginRescueBrowserKind> {
    let normalized = prog_id.trim().to_ascii_lowercase();
    if normalized.contains("brave") {
        Some(YoutubeLoginRescueBrowserKind::Brave)
    } else if normalized.contains("chrome") {
        Some(YoutubeLoginRescueBrowserKind::Chrome)
    } else if normalized.contains("msedge") || normalized.contains("edge") {
        Some(YoutubeLoginRescueBrowserKind::Edge)
    } else {
        None
    }
}

fn first_installed_supported_browser() -> Option<YoutubeLoginRescueBrowserInfo> {
    [
        YoutubeLoginRescueBrowserKind::Brave,
        YoutubeLoginRescueBrowserKind::Chrome,
        YoutubeLoginRescueBrowserKind::Edge,
    ]
    .into_iter()
    .find_map(browser_info_for_kind)
}

fn browser_info_for_kind(
    kind: YoutubeLoginRescueBrowserKind,
) -> Option<YoutubeLoginRescueBrowserInfo> {
    let display_name = match kind {
        YoutubeLoginRescueBrowserKind::Brave => "Brave",
        YoutubeLoginRescueBrowserKind::Chrome => "Chrome",
        YoutubeLoginRescueBrowserKind::Edge => "Microsoft Edge",
    };

    browser_exe_candidates(kind)
        .into_iter()
        .find(|path| path.is_file())
        .map(|exe_path| YoutubeLoginRescueBrowserInfo {
            kind,
            display_name: display_name.to_owned(),
            exe_path,
        })
}

fn browser_exe_candidates(kind: YoutubeLoginRescueBrowserKind) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    match kind {
        YoutubeLoginRescueBrowserKind::Brave => {
            push_env_path(
                &mut candidates,
                "LOCALAPPDATA",
                r"BraveSoftware\Brave-Browser\Application\brave.exe",
            );
            push_env_path(
                &mut candidates,
                "PROGRAMFILES",
                r"BraveSoftware\Brave-Browser\Application\brave.exe",
            );
            push_env_path(
                &mut candidates,
                "PROGRAMFILES(X86)",
                r"BraveSoftware\Brave-Browser\Application\brave.exe",
            );
        }
        YoutubeLoginRescueBrowserKind::Chrome => {
            push_env_path(
                &mut candidates,
                "PROGRAMFILES",
                r"Google\Chrome\Application\chrome.exe",
            );
            push_env_path(
                &mut candidates,
                "PROGRAMFILES(X86)",
                r"Google\Chrome\Application\chrome.exe",
            );
            push_env_path(
                &mut candidates,
                "LOCALAPPDATA",
                r"Google\Chrome\Application\chrome.exe",
            );
        }
        YoutubeLoginRescueBrowserKind::Edge => {
            push_env_path(
                &mut candidates,
                "PROGRAMFILES(X86)",
                r"Microsoft\Edge\Application\msedge.exe",
            );
            push_env_path(
                &mut candidates,
                "PROGRAMFILES",
                r"Microsoft\Edge\Application\msedge.exe",
            );
            push_env_path(
                &mut candidates,
                "LOCALAPPDATA",
                r"Microsoft\Edge\Application\msedge.exe",
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        match kind {
            YoutubeLoginRescueBrowserKind::Brave => candidates.push(PathBuf::from("brave-browser")),
            YoutubeLoginRescueBrowserKind::Chrome => {
                candidates.push(PathBuf::from("google-chrome"))
            }
            YoutubeLoginRescueBrowserKind::Edge => candidates.push(PathBuf::from("microsoft-edge")),
        }
    }

    candidates
}

fn push_env_path(candidates: &mut Vec<PathBuf>, env_key: &str, relative: &str) {
    if let Some(base) = std::env::var_os(env_key) {
        candidates.push(PathBuf::from(base).join(relative));
    }
}
