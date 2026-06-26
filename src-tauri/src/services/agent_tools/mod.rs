//! Agent optimization tools integration (Headroom, RTK, Ponytail).

mod headroom;
mod ponytail;
mod rtk;

pub use headroom::{
    chain_url_through_headroom, ensure_headroom_proxy, headroom_base_url, stop_headroom_proxy,
};
pub use ponytail::apply_ponytail_to_settings;
pub use rtk::apply_rtk_to_settings;

use crate::app_config::AppType;
use crate::database::Database;
use crate::error::AppError;
use crate::proxy::types::{AgentToolsConfig, AgentToolsStatus};
use crate::services::provider::live::sync_current_provider_for_app_to_live;
use crate::store::AppState;

const PONYTAIL_SKILL_ID: &str = "ponytail";
pub fn apply_to_provider_settings(
    app_type: &AppType,
    settings: &serde_json::Value,
    config: &AgentToolsConfig,
    proxy_takeover_active: bool,
) -> serde_json::Value {
    let mut effective = settings.clone();

    if config.headroom_enabled && !proxy_takeover_active {
        effective = headroom::apply_headroom_to_live_settings(app_type, &effective, config);
    }

    if config.rtk_enabled {
        effective = apply_rtk_to_settings(app_type, &effective, config);
    }

    if config.ponytail_enabled {
        effective = apply_ponytail_to_settings(app_type, &effective, config);
    } else {
        effective = ponytail::remove_ponytail_from_settings(app_type, &effective);
    }

    if !config.rtk_enabled {
        effective = rtk::remove_rtk_from_settings(app_type, &effective);
    }

    if !config.headroom_enabled || proxy_takeover_active {
        effective = headroom::remove_headroom_from_live_settings(app_type, &effective);
    }

    effective
}

/// Persist config, (re)start Headroom if needed, optionally install Ponytail skill,
/// and re-sync live configs for Claude/Codex/Gemini.
pub fn apply_config(state: &AppState, config: &AgentToolsConfig) -> Result<(), AppError> {
    state.db.set_agent_tools_config(config)?;

    if config.headroom_enabled && config.headroom_auto_start {
        if let Err(err) = ensure_headroom_proxy(config.headroom_port) {
            log::warn!("Failed to auto-start Headroom proxy: {err}");
        }
    } else if !config.headroom_enabled {
        stop_headroom_proxy();
    }

    if config.ponytail_enabled && config.ponytail_install_skill {
        if let Err(err) = ensure_ponytail_skill(state) {
            log::warn!("Failed to install Ponytail skill: {err}");
        }
    }

    for app_type in [AppType::Claude, AppType::Codex, AppType::Gemini] {
        if let Err(err) = sync_current_provider_for_app_to_live(state, &app_type) {
            log::warn!("Failed to re-sync {:?} live config after agent tools change: {err}", app_type);
        }
    }

    Ok(())
}

pub fn probe_status(db: &Database) -> AgentToolsStatus {
    let headroom = headroom::probe_headroom();
    let rtk = rtk::probe_rtk();
    let ponytail_skill_installed = db
        .get_all_installed_skills()
        .map(|skills| {
            skills
                .values()
                .any(|s| s.id == PONYTAIL_SKILL_ID || s.directory == PONYTAIL_SKILL_ID)
        })
        .unwrap_or(false);

    AgentToolsStatus {
        headroom_installed: headroom.installed,
        headroom_running: headroom.running,
        headroom_version: headroom.version,
        rtk_installed: rtk.installed,
        rtk_version: rtk.version,
        ponytail_skill_installed,
    }
}

fn ensure_ponytail_skill(state: &AppState) -> Result<(), AppError> {
    let skills = state.db.get_all_installed_skills()?;
    if skills
        .values()
        .any(|s| s.id == PONYTAIL_SKILL_ID || s.directory == PONYTAIL_SKILL_ID)
    {
        return Ok(());
    }

    log::info!(
        "Ponytail skill is not installed; enable it from the Skills page (DietrichGebert/ponytail)"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ponytail_injects_env_for_claude() {
        let config = AgentToolsConfig {
            ponytail_enabled: true,
            ponytail_mode: "lite".to_string(),
            ..AgentToolsConfig::default()
        };
        let out = apply_ponytail_to_settings(
            &AppType::Claude,
            &json!({"env": {"ANTHROPIC_API_KEY": "sk-test"}}),
            &config,
        );
        assert_eq!(
            out["env"]["PONYTAIL_DEFAULT_MODE"].as_str(),
            Some("lite")
        );
    }

    #[test]
    fn headroom_chain_replaces_origin() {
        let (url, header) = chain_url_through_headroom(
            "https://api.anthropic.com/v1/messages?beta=true",
            "https://api.anthropic.com",
            8787,
        );
        assert!(url.starts_with("http://127.0.0.1:8787/"));
        assert_eq!(
            header,
            Some((
                "x-headroom-base-url".to_string(),
                "https://api.anthropic.com".to_string()
            ))
        );
    }
}
