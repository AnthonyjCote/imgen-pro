use std::{fs, path::PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::State;

use crate::{
    automation, design, engines, jobs,
    models::{
        AppConfig, AutomationStatus, CapabilitySummary, CreativeBriefRequest, CreativePlan,
        DesignRequest, EngineProbe, GeneratedAsset, GenerationJob, GenerationRequest,
        TextModelProbe, TextProviderMode,
    },
    state::AppState,
    text_model,
};

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config()
}

#[tauri::command]
pub fn save_config(config: AppConfig, state: State<AppState>) -> Result<AppConfig, String> {
    validate_config(&config)?;
    state.replace_config(config)
}

#[tauri::command]
pub fn get_capabilities(state: State<AppState>) -> CapabilitySummary {
    let config = state.config();
    CapabilitySummary {
        image_generation: true,
        video_generation: false,
        loras: true,
        svg_composition: true,
        text_design_generation: config.text_model.mode != TextProviderMode::Disabled,
        automation_api: true,
        engine_mode: config.engine.mode,
    }
}

#[tauri::command]
pub fn list_jobs(state: State<AppState>) -> Vec<GenerationJob> {
    state.list_jobs()
}

#[tauri::command]
pub fn get_job(id: String, state: State<AppState>) -> Option<GenerationJob> {
    state.get_job(&id)
}

#[tauri::command]
pub fn enqueue_generation(
    request: GenerationRequest,
    state: State<AppState>,
) -> Result<GenerationJob, String> {
    jobs::enqueue(state.inner().clone(), request)
}

#[tauri::command]
pub fn cancel_job(id: String, state: State<AppState>) -> Result<bool, String> {
    jobs::cancel(&state, &id)
}

#[tauri::command]
pub async fn probe_engine(state: State<'_, AppState>) -> Result<EngineProbe, String> {
    engines::probe(&state.config().engine).await
}

#[tauri::command]
pub async fn probe_text_model(state: State<'_, AppState>) -> Result<TextModelProbe, String> {
    text_model::probe(&state.config().text_model).await
}

#[tauri::command]
pub async fn generate_creative_plan(
    request: CreativeBriefRequest,
    state: State<'_, AppState>,
) -> Result<CreativePlan, String> {
    text_model::generate_plan(&state.config().text_model, request).await
}

#[tauri::command]
pub fn render_design(
    request: DesignRequest,
    state: State<AppState>,
) -> Result<GeneratedAsset, String> {
    design::render(&state, request)
}

#[tauri::command]
pub fn read_asset_preview(path: String, state: State<AppState>) -> Result<String, String> {
    let path = PathBuf::from(path);
    if !state.is_path_inside_app_data(&path) {
        return Err("Asset preview path is outside the Imgen Pro data directory.".to_string());
    }
    let bytes =
        fs::read(&path).map_err(|error| format!("Unable to read asset preview: {error}"))?;
    let mime = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

#[tauri::command]
pub async fn start_automation_server(
    state: State<'_, AppState>,
) -> Result<AutomationStatus, String> {
    automation::start(state.inner().clone()).await
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.engine.models.is_empty() {
        return Err("At least one model profile is required.".to_string());
    }
    if !config
        .engine
        .models
        .iter()
        .any(|model| model.id == config.engine.active_model_id)
    {
        return Err("The active model ID does not match a model profile.".to_string());
    }
    if config.text_model.mode != TextProviderMode::Disabled {
        if config.text_model.base_url.trim().is_empty() {
            return Err("Local text model base URL cannot be empty.".to_string());
        }
        if config.text_model.model.trim().is_empty() {
            return Err("Local text model name cannot be empty.".to_string());
        }
        if !(0.0..=2.0).contains(&config.text_model.temperature) {
            return Err("Local text model temperature must be between 0 and 2.".to_string());
        }
        if !(64..=8_192).contains(&config.text_model.max_tokens) {
            return Err("Local text model max tokens must be between 64 and 8192.".to_string());
        }
    }
    if config.automation.port == 0 {
        return Err("Automation port must be greater than zero.".to_string());
    }
    if config.automation.token.trim().len() < 12 {
        return Err("Automation token must contain at least 12 characters.".to_string());
    }
    Ok(())
}
