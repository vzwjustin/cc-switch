#![allow(non_snake_case)]

use serde::Serialize;

use crate::companion_tools::{self, COMPANION_APPS, COMPANION_TOOLS};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionToolAppStatus {
    pub app: String,
    pub installed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionToolStatus {
    pub tool: String,
    pub apps: Vec<CompanionToolAppStatus>,
}

#[tauri::command]
pub async fn get_companion_tools_status() -> Result<Vec<CompanionToolStatus>, String> {
    Ok(COMPANION_TOOLS
        .iter()
        .map(|tool| CompanionToolStatus {
            tool: (*tool).to_string(),
            apps: COMPANION_APPS
                .iter()
                .map(|app| CompanionToolAppStatus {
                    app: (*app).to_string(),
                    installed: companion_tools::is_companion_tool_installed(tool, app),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub async fn install_companion_tool(tool: String, app: String) -> Result<(), String> {
    companion_tools::install_companion_tool(&tool, &app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_companion_tool(tool: String, app: String) -> Result<(), String> {
    companion_tools::uninstall_companion_tool(&tool, &app).map_err(|e| e.to_string())
}
