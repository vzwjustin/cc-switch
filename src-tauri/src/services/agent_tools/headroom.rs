//! Headroom proxy lifecycle and live-config / forwarder chaining helpers.

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::app_config::AppType;
use crate::proxy::types::AgentToolsConfig;

static HEADROOM_CHILD: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));

pub struct HeadroomProbe {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
}

pub fn headroom_base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

pub fn probe_headroom() -> HeadroomProbe {
    let installed = command_exists("headroom");
    let version = if installed {
        Command::new("headroom")
            .arg("--version")
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    };

    HeadroomProbe {
        installed,
        running: is_headroom_running(),
        version,
    }
}

pub fn is_headroom_running() -> bool {
    if let Ok(mut guard) = HEADROOM_CHILD.lock() {
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    *guard = None;
                    return false;
                }
                Ok(None) => return true,
                Err(_) => {
                    *guard = None;
                }
            }
        }
    }

    // Also treat an external Headroom process as running when the port accepts connections.
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], 8787)),
        std::time::Duration::from_millis(200),
    )
    .is_ok()
}

pub fn ensure_headroom_proxy(port: u16) -> Result<(), String> {
    if is_headroom_running() {
        return Ok(());
    }

    let headroom = if command_exists("headroom") {
        "headroom".to_string()
    } else {
        return Err("headroom binary not found on PATH".to_string());
    };

    let mut child = Command::new(&headroom)
        .args(["proxy", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start headroom proxy: {e}"))?;

    // Give the proxy a moment to bind.
    std::thread::sleep(std::time::Duration::from_millis(400));
    if child.try_wait().ok().flatten().is_some() {
        return Err("Headroom proxy exited immediately after start".to_string());
    }

    if let Ok(mut guard) = HEADROOM_CHILD.lock() {
        *guard = Some(child);
    }

    Ok(())
}

pub fn stop_headroom_proxy() {
    if let Ok(mut guard) = HEADROOM_CHILD.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn apply_headroom_to_live_settings(
    app_type: &AppType,
    settings: &Value,
    config: &AgentToolsConfig,
) -> Value {
    let mut value = settings.clone();
    let proxy_url = headroom_base_url(config.headroom_port);

    match app_type {
        AppType::Claude => {
            let root = value.as_object_mut().expect("claude settings object");
            let env = root
                .entry("env")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .expect("claude env object");
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                Value::String(proxy_url.clone()),
            );
        }
        AppType::Codex => {
            if let Some(obj) = value.as_object_mut() {
                let auth = obj
                    .entry("auth")
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .expect("codex auth object");
                auth.insert(
                    "OPENAI_BASE_URL".to_string(),
                    Value::String(format!("{proxy_url}/v1")),
                );
            }
        }
        AppType::Gemini => {
            if let Some(obj) = value.as_object_mut() {
                let env = obj
                    .entry("env")
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .expect("gemini env object");
                env.insert(
                    "GOOGLE_GEMINI_BASE_URL".to_string(),
                    Value::String(format!("{proxy_url}/v1beta")),
                );
            }
        }
        _ => {}
    }

    value
}

pub fn remove_headroom_from_live_settings(app_type: &AppType, settings: &Value) -> Value {
    let mut value = settings.clone();
    match app_type {
        AppType::Claude => {
            if let Some(env) = value
                .get_mut("env")
                .and_then(Value::as_object_mut)
            {
                env.remove("ANTHROPIC_BASE_URL");
            }
        }
        AppType::Codex => {
            if let Some(auth) = value
                .get_mut("auth")
                .and_then(Value::as_object_mut)
            {
                auth.remove("OPENAI_BASE_URL");
            }
        }
        AppType::Gemini => {
            if let Some(env) = value
                .get_mut("env")
                .and_then(Value::as_object_mut)
            {
                env.remove("GOOGLE_GEMINI_BASE_URL");
            }
        }
        _ => {}
    }
    value
}

/// Route an upstream URL through the local Headroom proxy and return the custom upstream header.
pub fn chain_url_through_headroom(
    url: &str,
    base_url: &str,
    port: u16,
) -> (String, Option<(String, String)>) {
    let proxy_origin = headroom_base_url(port);
    let chained = replace_url_origin(url, &proxy_origin);
    let header = (
        "x-headroom-base-url".to_string(),
        base_url.trim_end_matches('/').to_string(),
    );
    (chained, Some(header))
}

fn replace_url_origin(url: &str, new_origin: &str) -> String {
    let parsed = match url::Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => return format!("{new_origin}{url}"),
    };

    let path = parsed.path();
    let query = parsed.query().map(|q| format!("?{q}")).unwrap_or_default();
    let fragment = parsed
        .fragment()
        .map(|f| format!("#{f}"))
        .unwrap_or_default();

    format!("{new_origin}{path}{query}{fragment}")
}

fn command_exists(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg(cmd)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_url_origin_preserves_path_and_query() {
        let out = replace_url_origin(
            "https://api.anthropic.com/v1/messages?beta=true",
            "http://127.0.0.1:8787",
        );
        assert_eq!(
            out,
            "http://127.0.0.1:8787/v1/messages?beta=true"
        );
    }
}
