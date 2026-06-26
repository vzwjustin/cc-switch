//! Global OpenCode config sync for agent tools (plugins, mode files).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::opencode_config::{self, get_opencode_dir};
use crate::proxy::types::AgentToolsConfig;

const CC_SWITCH_STATE_FILE: &str = ".cc-switch-agent-tools.json";
const RTK_PLUGIN_FILENAME: &str = "cc-switch-rtk.ts";
const PONYTAIL_MODE_FILE: &str = ".ponytail-active";
const PONYTAIL_NPM_PLUGIN: &str = "@dietrichgebert/ponytail";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeAgentToolsState {
    #[serde(default)]
    rtk_plugin_managed: bool,
    #[serde(default)]
    ponytail_plugin_managed: bool,
    #[serde(default)]
    ponytail_mode_managed: bool,
}

pub fn sync_opencode_global(config: &AgentToolsConfig) -> Result<(), AppError> {
    let mut state = read_state();

    if config.rtk_enabled && config.rtk_opencode {
        install_rtk_plugin()?;
        state.rtk_plugin_managed = true;
    } else if state.rtk_plugin_managed {
        remove_rtk_plugin()?;
        state.rtk_plugin_managed = false;
    }

    if config.ponytail_enabled {
        let mode = normalize_ponytail_mode(&config.ponytail_mode);
        if mode != "off" {
            write_ponytail_mode_file(&mode)?;
            state.ponytail_mode_managed = true;
            opencode_config::add_plugin(PONYTAIL_NPM_PLUGIN)?;
            state.ponytail_plugin_managed = true;
        } else {
            if state.ponytail_mode_managed {
                remove_ponytail_mode_file()?;
                state.ponytail_mode_managed = false;
            }
            if state.ponytail_plugin_managed {
                opencode_config::remove_plugin_exact(PONYTAIL_NPM_PLUGIN)?;
                state.ponytail_plugin_managed = false;
            }
        }
    } else {
        if state.ponytail_mode_managed {
            remove_ponytail_mode_file()?;
            state.ponytail_mode_managed = false;
        }
        if state.ponytail_plugin_managed {
            opencode_config::remove_plugin_exact(PONYTAIL_NPM_PLUGIN)?;
            state.ponytail_plugin_managed = false;
        }
    }

    write_state(&state)?;
    Ok(())
}

fn plugins_dir() -> PathBuf {
    get_opencode_dir().join("plugins")
}

fn rtk_plugin_path() -> PathBuf {
    plugins_dir().join(RTK_PLUGIN_FILENAME)
}

fn state_path() -> PathBuf {
    get_opencode_dir().join(CC_SWITCH_STATE_FILE)
}

fn ponytail_mode_path() -> PathBuf {
    get_opencode_dir().join(PONYTAIL_MODE_FILE)
}

fn install_rtk_plugin() -> Result<(), AppError> {
    let dir = plugins_dir();
    fs::create_dir_all(&dir).map_err(|e| AppError::io(&dir, e))?;
    let path = rtk_plugin_path();
    fs::write(&path, include_str!("../../../resources/cc-switch-rtk.ts"))
        .map_err(|e| AppError::io(&path, e))?;
    log::info!("Wrote OpenCode RTK plugin to {}", path.display());
    Ok(())
}

fn remove_rtk_plugin() -> Result<(), AppError> {
    remove_file_if_exists(&rtk_plugin_path())
}

fn write_ponytail_mode_file(mode: &str) -> Result<(), AppError> {
    let dir = get_opencode_dir();
    fs::create_dir_all(&dir).map_err(|e| AppError::io(&dir, e))?;
    let path = ponytail_mode_path();
    fs::write(&path, format!("{mode}\n")).map_err(|e| AppError::io(&path, e))?;
    Ok(())
}

fn remove_ponytail_mode_file() -> Result<(), AppError> {
    remove_file_if_exists(&ponytail_mode_path())
}

fn remove_file_if_exists(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| AppError::io(path, e))?;
    }
    Ok(())
}

fn read_state() -> OpenCodeAgentToolsState {
    let path = state_path();
    if !path.exists() {
        return OpenCodeAgentToolsState::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn write_state(state: &OpenCodeAgentToolsState) -> Result<(), AppError> {
    let dir = get_opencode_dir();
    fs::create_dir_all(&dir).map_err(|e| AppError::io(&dir, e))?;
    let path = state_path();
    let content = serde_json::to_string_pretty(state).map_err(|e| AppError::JsonSerialize { source: e })?;
    fs::write(&path, format!("{content}\n")).map_err(|e| AppError::io(&path, e))?;
    Ok(())
}

fn normalize_ponytail_mode(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        "lite" | "full" | "ultra" | "off" => mode.trim().to_ascii_lowercase(),
        _ => "full".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ponytail_mode_defaults_to_full() {
        assert_eq!(normalize_ponytail_mode("bogus"), "full");
        assert_eq!(normalize_ponytail_mode("lite"), "lite");
    }
}
