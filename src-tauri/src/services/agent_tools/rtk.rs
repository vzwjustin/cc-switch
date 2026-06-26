//! RTK shell-command rewriter integration for Claude / Codex live configs.

use std::path::PathBuf;
use std::process::Command;

use serde_json::{json, Value};

use crate::app_config::AppType;
use crate::proxy::types::AgentToolsConfig;

const RTK_HOOK_ID: &str = "cc-switch-rtk-rewrite";

pub struct RtkProbe {
    pub installed: bool,
    pub version: Option<String>,
}

pub fn probe_rtk() -> RtkProbe {
    let installed = command_exists("rtk");
    let version = if installed {
        Command::new("rtk")
            .arg("--version")
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    };

    RtkProbe {
        installed,
        version,
    }
}

pub fn apply_rtk_to_settings(
    app_type: &AppType,
    settings: &Value,
    config: &AgentToolsConfig,
) -> Value {
    match app_type {
        AppType::Claude if config.rtk_claude => inject_claude_rtk_hook(settings),
        AppType::Codex if config.rtk_codex => inject_codex_rtk_env(settings),
        _ => settings.clone(),
    }
}

pub fn remove_rtk_from_settings(app_type: &AppType, settings: &Value) -> Value {
    match app_type {
        AppType::Claude => remove_claude_rtk_hook(settings),
        AppType::Codex => remove_codex_rtk_env(settings),
        _ => settings.clone(),
    }
}

fn inject_claude_rtk_hook(settings: &Value) -> Value {
    let mut value = settings.clone();
    let root = value
        .as_object_mut()
        .expect("claude settings must be an object");

    let hooks = root
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .expect("claude hooks must be an object");

    let entries = hooks
        .entry("PreToolUse")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .expect("PreToolUse must be an array");

    entries.retain(|entry| !is_cc_switch_rtk_entry(entry));

    let script = rtk_hook_script_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "bash".to_string());

    let command = if script.ends_with("rtk-rewrite.sh") {
        format!("bash {}", script)
    } else {
        "rtk init -g".to_string()
    };

    entries.push(json!({
        "matcher": "Bash",
        "id": RTK_HOOK_ID,
        "hooks": [{
            "type": "command",
            "command": command
        }]
    }));

    value
}

fn remove_claude_rtk_hook(settings: &Value) -> Value {
    let mut value = settings.clone();
    let Some(hooks) = value.get_mut("hooks").and_then(Value::as_object_mut) else {
        return value;
    };
    let Some(entries) = hooks.get_mut("PreToolUse").and_then(Value::as_array_mut) else {
        return value;
    };

    entries.retain(|entry| !is_cc_switch_rtk_entry(entry));
    if entries.is_empty() {
        hooks.remove("PreToolUse");
    }
    if hooks.is_empty() {
        if let Some(root) = value.as_object_mut() {
            root.remove("hooks");
        }
    }
    value
}

fn is_cc_switch_rtk_entry(entry: &Value) -> bool {
    entry.get("id").and_then(Value::as_str) == Some(RTK_HOOK_ID)
}

fn inject_codex_rtk_env(settings: &Value) -> Value {
    let mut value = settings.clone();
    if let Some(obj) = value.as_object_mut() {
        let auth = obj
            .entry("auth")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .expect("codex auth object");
        auth.insert(
            "RTK_ENABLED".to_string(),
            Value::String("1".to_string()),
        );
    }
    value
}

fn remove_codex_rtk_env(settings: &Value) -> Value {
    let mut value = settings.clone();
    if let Some(auth) = value.get_mut("auth").and_then(Value::as_object_mut) {
        auth.remove("RTK_ENABLED");
    }
    value
}

fn rtk_hook_script_path() -> Option<PathBuf> {
    if let Ok(resource_dir) = std::env::var("TAURI_RESOURCE_DIR") {
        let path = PathBuf::from(resource_dir).join("rtk-rewrite.sh");
        if path.exists() {
            return Some(path);
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_path = manifest_dir.join("resources").join("rtk-rewrite.sh");
    if dev_path.exists() {
        return Some(dev_path);
    }

    None
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
    fn inject_and_remove_claude_rtk_hook() {
        let base = json!({"env": {}});
        let with_hook = inject_claude_rtk_hook(&base);
        assert!(with_hook["hooks"]["PreToolUse"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.get("id").and_then(|v| v.as_str()) == Some(RTK_HOOK_ID)));

        let cleaned = remove_claude_rtk_hook(&with_hook);
        assert!(cleaned.get("hooks").is_none());
    }
}
