use std::path::PathBuf;
use std::process::Command;

use crate::error::AppError;
use crate::gemini_config::get_gemini_dir;
use crate::opencode_config;

pub const COMPANION_TOOLS: [&str; 3] = ["ponytail", "rtk", "headroom"];
pub const COMPANION_APPS: [&str; 4] = ["claude", "codex", "gemini", "opencode"];

fn claude_dir() -> PathBuf {
    if let Some(dir) = crate::settings::get_claude_override_dir() {
        return dir;
    }
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
}

fn is_ponytail_installed_for(app: &str) -> bool {
    match app {
        "opencode" => opencode_has_plugin("@dietrichgebert/ponytail"),
        "claude" => json_has_key_containing(
            &claude_dir().join("plugins").join("installed_plugins.json"),
            "ponytail",
        ),
        "codex" => toml_or_text_contains(
            &crate::codex_config::get_codex_config_path(),
            "ponytail",
        ),
        "gemini" => dir_contains_name(&get_gemini_dir().join("extensions"), "ponytail"),
        _ => false,
    }
}

fn is_rtk_installed_for(app: &str) -> bool {
    match app {
        "claude" => json_contains(
            &claude_dir().join("settings.json"),
            &["rtk", "rtk-rewrite"],
        ),
        "codex" => toml_or_text_contains(
            &crate::codex_config::get_codex_config_path(),
            "rtk",
        ) || path_exists(&dirs::home_dir().unwrap_or_default().join(".codex").join("RTK.md")),
        "gemini" => json_contains(
            &get_gemini_dir().join("settings.json"),
            &["rtk", "rtk-rewrite"],
        ),
        "opencode" => opencode_has_plugin("rtk") || opencode_has_plugin("./openclaw"),
        _ => false,
    }
}

fn is_headroom_installed_for(app: &str) -> bool {
    match app {
        "claude" => json_contains(
            &claude_dir().join("settings.json"),
            &["headroom", "headroom_compress", "headroom_retrieve"],
        ),
        "codex" => toml_or_text_contains(
            &crate::codex_config::get_codex_config_path(),
            "headroom",
        ),
        "gemini" => json_contains(
            &get_gemini_dir().join("settings.json"),
            &["headroom", "headroom_compress", "headroom_retrieve"],
        ),
        "opencode" => opencode_has_plugin("headroom")
            || json_contains_value(
                &opencode_config::get_opencode_config_path(),
                "mcpServers",
                "headroom",
            ),
        _ => false,
    }
}

pub fn is_companion_tool_installed(tool: &str, app: &str) -> bool {
    if !COMPANION_TOOLS.contains(&tool) || !COMPANION_APPS.contains(&app) {
        return false;
    }
    match tool {
        "ponytail" => is_ponytail_installed_for(app),
        "rtk" => is_rtk_installed_for(app),
        "headroom" => is_headroom_installed_for(app),
        _ => false,
    }
}

pub fn install_companion_tool(tool: &str, app: &str) -> Result<(), AppError> {
    if !COMPANION_TOOLS.contains(&tool) || !COMPANION_APPS.contains(&app) {
        return Err(AppError::Config(format!(
            "Unsupported companion tool install target: {tool} -> {app}"
        )));
    }

    if tool == "ponytail" && app == "opencode" {
        return opencode_config::add_plugin("@dietrichgebert/ponytail");
    }

    let command = install_command(tool, app).ok_or_else(|| {
        AppError::Config(format!("No install command for companion tool: {tool} -> {app}"))
    })?;

    run_shell_command(command)
}

pub fn uninstall_companion_tool(tool: &str, app: &str) -> Result<(), AppError> {
    if !COMPANION_TOOLS.contains(&tool) || !COMPANION_APPS.contains(&app) {
        return Err(AppError::Config(format!(
            "Unsupported companion tool uninstall target: {tool} -> {app}"
        )));
    }

    if tool == "ponytail" && app == "opencode" {
        return remove_opencode_plugin("@dietrichgebert/ponytail");
    }

    let command = uninstall_command(tool, app).ok_or_else(|| {
        AppError::Config(format!("No uninstall command for companion tool: {tool} -> {app}"))
    })?;

    run_shell_command(command)
}

fn install_command(tool: &str, app: &str) -> Option<&'static str> {
    match (tool, app) {
        ("ponytail", "claude") => Some(
            "claude plugin marketplace add DietrichGebert/ponytail && claude plugin install ponytail@ponytail",
        ),
        ("ponytail", "codex") => Some(
            "codex plugin marketplace add DietrichGebert/ponytail && codex plugin install ponytail@ponytail",
        ),
        ("ponytail", "gemini") => {
            Some("gemini extensions install https://github.com/DietrichGebert/ponytail")
        }
        ("rtk", "claude") => Some("rtk init -g --auto-patch"),
        ("rtk", "codex") => Some("rtk init -g --codex --auto-patch"),
        ("rtk", "gemini") => Some("rtk init -g --gemini --auto-patch"),
        ("rtk", "opencode") => Some("rtk init -g --opencode --auto-patch"),
        ("headroom", "claude") => Some("headroom mcp install"),
        ("headroom", "codex") => {
            Some("headroom install apply --preset persistent-service --target codex")
        }
        ("headroom", "gemini") => {
            Some("headroom install apply --preset persistent-service --target gemini")
        }
        ("headroom", "opencode") => {
            Some("headroom install apply --preset persistent-service --target opencode")
        }
        _ => None,
    }
}

fn uninstall_command(tool: &str, app: &str) -> Option<&'static str> {
    match (tool, app) {
        ("ponytail", "claude") => Some("claude plugin remove ponytail@ponytail"),
        ("ponytail", "codex") => Some("codex plugin remove ponytail@ponytail"),
        ("ponytail", "gemini") => {
            Some("gemini extensions uninstall https://github.com/DietrichGebert/ponytail")
        }
        ("rtk", "claude") => Some("rtk init -g --uninstall"),
        ("rtk", "codex") => Some("rtk init -g --codex --uninstall"),
        ("rtk", "gemini") => Some("rtk init -g --gemini --uninstall"),
        ("rtk", "opencode") => Some("rtk init -g --opencode --uninstall"),
        ("headroom", "claude") => Some("headroom mcp uninstall"),
        ("headroom", "codex") => Some("headroom unwrap codex"),
        ("headroom", "gemini") => Some("headroom unwrap gemini"),
        ("headroom", "opencode") => Some("headroom unwrap opencode"),
        _ => None,
    }
}

fn remove_opencode_plugin(plugin_name: &str) -> Result<(), AppError> {
    let mut config = opencode_config::read_opencode_config()?;
    if let Some(arr) = config.get_mut("plugin").and_then(|v| v.as_array_mut()) {
        arr.retain(|v| v.as_str() != Some(plugin_name));
        if arr.is_empty() {
            if let Some(obj) = config.as_object_mut() {
                obj.remove("plugin");
            }
        }
    }
    opencode_config::write_opencode_config(&config).map_err(AppError::from)
}

fn opencode_has_plugin(plugin_name: &str) -> bool {
    opencode_config::read_opencode_config()
        .ok()
        .and_then(|config| {
            config
                .get("plugin")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().any(|entry| {
                        entry
                            .as_str()
                            .map(|s| s.contains(plugin_name))
                            .unwrap_or(false)
                    })
                })
        })
        .unwrap_or(false)
}

fn path_exists(path: &std::path::Path) -> bool {
    path.exists()
}

fn dir_contains_name(dir: &std::path::Path, needle: &str) -> bool {
    if !dir.exists() {
        return false;
    }
    std::fs::read_dir(dir).ok().is_some_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains(needle)
        })
    })
}

fn json_has_key_containing(path: &std::path::Path, needle: &str) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };
    content.to_ascii_lowercase().contains(needle)
}

fn json_contains(path: &std::path::Path, needles: &[&str]) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };
    let lower = content.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

fn json_contains_value(path: &std::path::Path, field: &str, needle: &str) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    value
        .get(field)
        .map(|v| v.to_string().to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}

fn toml_or_text_contains(path: &std::path::Path, needle: &str) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };
    content.to_ascii_lowercase().contains(needle)
}

fn run_shell_command(command: &str) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", command])
        .output()
        .map_err(|e| AppError::Config(format!("Failed to run companion tool command: {e}")))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bash")
        .arg("-lc")
        .arg(command)
        .output()
        .map_err(|e| AppError::Config(format!("Failed to run companion tool command: {e}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };
    Err(AppError::Config(if detail.is_empty() {
        format!("Companion tool command failed: {command}")
    } else {
        detail
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_commands_exist_for_all_targets() {
        for tool in COMPANION_TOOLS {
            for app in COMPANION_APPS {
                if *tool == "ponytail" && *app == "opencode" {
                    continue;
                }
                assert!(
                    install_command(tool, app).is_some(),
                    "missing install command for {tool} -> {app}"
                );
            }
        }
    }
}
