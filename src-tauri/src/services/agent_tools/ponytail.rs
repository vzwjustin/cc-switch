//! Ponytail lazy-senior-dev ruleset injection via environment variables.

use serde_json::{json, Value};

use crate::app_config::AppType;
use crate::proxy::types::AgentToolsConfig;

const PONYTAIL_ENV_KEY: &str = "PONYTAIL_DEFAULT_MODE";

pub fn apply_ponytail_to_settings(
    app_type: &AppType,
    settings: &Value,
    config: &AgentToolsConfig,
) -> Value {
    let mode = normalize_ponytail_mode(&config.ponytail_mode);
    if mode == "off" {
        return remove_ponytail_from_settings(app_type, settings);
    }

    match app_type {
        AppType::Claude => inject_env_key(settings, "env", PONYTAIL_ENV_KEY, &mode),
        AppType::Codex => inject_env_key(settings, "auth", PONYTAIL_ENV_KEY, &mode),
        AppType::Gemini => inject_env_key(settings, "env", PONYTAIL_ENV_KEY, &mode),
        _ => settings.clone(),
    }
}

pub fn remove_ponytail_from_settings(app_type: &AppType, settings: &Value) -> Value {
    match app_type {
        AppType::Claude => remove_env_key(settings, "env", PONYTAIL_ENV_KEY),
        AppType::Codex => remove_env_key(settings, "auth", PONYTAIL_ENV_KEY),
        AppType::Gemini => remove_env_key(settings, "env", PONYTAIL_ENV_KEY),
        _ => settings.clone(),
    }
}

fn normalize_ponytail_mode(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        "lite" | "full" | "ultra" | "off" => mode.trim().to_ascii_lowercase(),
        _ => "full".to_string(),
    }
}

fn inject_env_key(settings: &Value, section: &str, key: &str, value: &str) -> Value {
    let mut out = settings.clone();
    if let Some(obj) = out.as_object_mut() {
        let section_obj = obj
            .entry(section.to_string())
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .expect("env section object");
        section_obj.insert(key.to_string(), Value::String(value.to_string()));
    }
    out
}

fn remove_env_key(settings: &Value, section: &str, key: &str) -> Value {
    let mut out = settings.clone();
    if let Some(section_obj) = out.get_mut(section).and_then(Value::as_object_mut) {
        section_obj.remove(key);
        if section_obj.is_empty() {
            if let Some(root) = out.as_object_mut() {
                root.remove(section);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ponytail_mode_normalization() {
        let config = AgentToolsConfig {
            ponytail_enabled: true,
            ponytail_mode: "ULTRA".to_string(),
            ..AgentToolsConfig::default()
        };
        let out = apply_ponytail_to_settings(
            &AppType::Gemini,
            &json!({}),
            &config,
        );
        assert_eq!(out["env"][PONYTAIL_ENV_KEY].as_str(), Some("ultra"));
    }
}
