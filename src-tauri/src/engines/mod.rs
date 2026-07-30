mod mock;
mod sd_cpp;
mod sd_server;

use crate::{
    models::{
        EngineConfig, EngineMode, EngineProbe, GeneratedAsset, GenerationRequest, ModelProfile,
    },
    state::AppState,
};

pub async fn generate(
    state: &AppState,
    job_id: &str,
    request: &GenerationRequest,
) -> Result<(GeneratedAsset, Vec<String>), String> {
    let config = state.config();
    let model = config
        .engine
        .models
        .iter()
        .find(|model| model.id == request.model_id && model.enabled)
        .cloned()
        .ok_or_else(|| format!("Model profile '{}' is not available.", request.model_id))?;

    match config.engine.mode {
        EngineMode::Mock => mock::generate(state, job_id, request).await,
        EngineMode::StableDiffusionCpp => {
            sd_cpp::generate(state, request, &config.engine, &model).await
        }
        EngineMode::StableDiffusionCppServer => {
            sd_server::generate(state, job_id, request, &config.engine, &model).await
        }
    }
}

pub async fn probe(config: &EngineConfig) -> Result<EngineProbe, String> {
    match config.mode {
        EngineMode::Mock => Ok(EngineProbe {
            ready: true,
            mode: EngineMode::Mock,
            binary_path: String::new(),
            summary: "Mock engine is ready for workflow viability testing.".to_string(),
            output: "No external process is required in mock mode.".to_string(),
        }),
        EngineMode::StableDiffusionCpp => sd_cpp::probe(config).await,
        EngineMode::StableDiffusionCppServer => sd_server::probe(config).await,
    }
}

pub(crate) fn validate_model_paths(model: &ModelProfile) -> Result<(), String> {
    let has_complete_model = !model.model_path.trim().is_empty();
    let has_diffusion_model = !model.diffusion_model_path.trim().is_empty();
    if !has_complete_model && !has_diffusion_model {
        return Err(
            "Configure either a complete model path or a diffusion model path.".to_string(),
        );
    }

    for (label, path) in [
        ("model", model.model_path.as_str()),
        ("diffusion model", model.diffusion_model_path.as_str()),
        ("VAE", model.vae_path.as_str()),
        ("CLIP-L", model.clip_l_path.as_str()),
        ("T5XXL", model.t5xxl_path.as_str()),
        ("LLM text encoder", model.llm_path.as_str()),
    ] {
        if !path.trim().is_empty() && !std::path::Path::new(path).is_file() {
            return Err(format!(
                "The configured {label} file does not exist: {path}"
            ));
        }
    }
    Ok(())
}
