use std::{path::Path, process::Stdio, time::Duration};

use tokio::{process::Command, time::timeout};
use uuid::Uuid;

use crate::{
    models::{
        AssetKind, BackendPreference, EngineConfig, EngineMode, EngineProbe, GeneratedAsset,
        GenerationRequest, LoraProfile, ModelProfile,
    },
    state::AppState,
};

use super::validate_model_paths;

pub async fn probe(config: &EngineConfig) -> Result<EngineProbe, String> {
    let binary = config.binary_path.trim();
    if binary.is_empty() {
        return Ok(not_ready(binary, "Set the path to the sd-cli executable."));
    }
    if !Path::new(binary).is_file() {
        return Ok(not_ready(
            binary,
            "The configured executable does not exist.",
        ));
    }

    let output = timeout(
        Duration::from_secs(12),
        Command::new(binary)
            .arg("--help")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| "Engine probe timed out after 12 seconds.".to_string())?
    .map_err(|error| format!("Unable to launch engine: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}").trim().to_string();

    Ok(EngineProbe {
        ready: output.status.success() || !combined.is_empty(),
        mode: EngineMode::StableDiffusionCpp,
        binary_path: binary.to_string(),
        summary: if output.status.success() {
            "stable-diffusion.cpp process launched successfully.".to_string()
        } else {
            "The process launched but returned a non-zero status.".to_string()
        },
        output: combined.chars().take(8_000).collect(),
    })
}

pub async fn generate(
    state: &AppState,
    request: &GenerationRequest,
    engine: &EngineConfig,
    model: &ModelProfile,
) -> Result<(GeneratedAsset, Vec<String>), String> {
    let binary = engine.binary_path.trim();
    if binary.is_empty() || !Path::new(binary).is_file() {
        return Err("The stable-diffusion.cpp binary path is not valid.".to_string());
    }

    validate_model_paths(model)?;

    let asset_id = Uuid::new_v4().to_string();
    let output_path = state.paths().outputs.join(format!("{asset_id}.png"));
    let mut args = build_model_args(model);
    args.extend(build_backend_args(&engine.backend));
    let prompt = prompt_with_loras(request, &engine.loras, model)?;

    args.extend([
        "-p".to_string(),
        prompt,
        "-W".to_string(),
        request.width.to_string(),
        "-H".to_string(),
        request.height.to_string(),
        "--steps".to_string(),
        request.steps.to_string(),
        "--seed".to_string(),
        request.seed.unwrap_or(42).to_string(),
        "-o".to_string(),
        output_path.to_string_lossy().to_string(),
    ]);

    if !request.negative_prompt.trim().is_empty() {
        args.extend(["-n".to_string(), request.negative_prompt.clone()]);
    }
    if !model.lora_directory.trim().is_empty() && !request.loras.is_empty() {
        args.extend(["--lora-model-dir".to_string(), model.lora_directory.clone()]);
    }
    args.extend(model.extra_args.clone());

    let command_preview = format!("{} {}", binary, shell_preview(&args));
    let output = Command::new(binary)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|error| format!("Unable to launch stable-diffusion.cpp: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let logs = vec![command_preview, stdout.clone(), stderr.clone()];

    if !output.status.success() {
        return Err(format!(
            "stable-diffusion.cpp exited with {}.\n{}",
            output.status,
            stderr.chars().take(4_000).collect::<String>()
        ));
    }
    if !output_path.is_file() {
        return Err(format!(
            "The engine reported success but did not create {}.",
            output_path.display()
        ));
    }

    Ok((
        GeneratedAsset {
            id: asset_id,
            path: output_path.to_string_lossy().to_string(),
            mime_type: "image/png".to_string(),
            width: request.width,
            height: request.height,
            kind: AssetKind::Image,
        },
        logs,
    ))
}

fn not_ready(binary: &str, summary: &str) -> EngineProbe {
    EngineProbe {
        ready: false,
        mode: EngineMode::StableDiffusionCpp,
        binary_path: binary.to_string(),
        summary: summary.to_string(),
        output: String::new(),
    }
}

fn build_model_args(model: &ModelProfile) -> Vec<String> {
    let mut args = Vec::new();
    if !model.model_path.trim().is_empty() {
        args.extend(["-m".to_string(), model.model_path.clone()]);
    }
    if !model.diffusion_model_path.trim().is_empty() {
        args.extend([
            "--diffusion-model".to_string(),
            model.diffusion_model_path.clone(),
        ]);
    }
    if !model.vae_path.trim().is_empty() {
        args.extend(["--vae".to_string(), model.vae_path.clone()]);
    }
    if !model.clip_l_path.trim().is_empty() {
        args.extend(["--clip_l".to_string(), model.clip_l_path.clone()]);
    }
    if !model.t5xxl_path.trim().is_empty() {
        args.extend(["--t5xxl".to_string(), model.t5xxl_path.clone()]);
    }
    if !model.llm_path.trim().is_empty() {
        args.extend(["--llm".to_string(), model.llm_path.clone()]);
    }
    args
}

fn build_backend_args(preference: &BackendPreference) -> Vec<String> {
    match preference {
        BackendPreference::Auto => Vec::new(),
        BackendPreference::Vulkan => vec![
            "--backend".to_string(),
            "diffusion=vulkan0,te=cpu,vae=cpu".to_string(),
            "--params-backend".to_string(),
            "diffusion=vulkan0,te=cpu,vae=cpu".to_string(),
        ],
        BackendPreference::Metal => vec![
            "--backend".to_string(),
            "diffusion=MTL0,te=cpu,vae=cpu".to_string(),
            "--params-backend".to_string(),
            "diffusion=MTL0,te=cpu,vae=cpu".to_string(),
        ],
        BackendPreference::Cpu => vec![
            "--backend".to_string(),
            "all=CPU".to_string(),
            "--params-backend".to_string(),
            "all=CPU".to_string(),
            "--offload-to-cpu".to_string(),
        ],
    }
}

fn prompt_with_loras(
    request: &GenerationRequest,
    profiles: &[LoraProfile],
    model: &ModelProfile,
) -> Result<String, String> {
    let mut prompt = request.prompt.trim().to_string();
    for selection in &request.loras {
        let profile = profiles
            .iter()
            .find(|profile| profile.id == selection.id)
            .ok_or_else(|| format!("LoRA profile '{}' was not found.", selection.id))?;
        if !profile.compatible_families.is_empty()
            && !profile
                .compatible_families
                .iter()
                .any(|family| family.eq_ignore_ascii_case(&model.family))
        {
            return Err(format!(
                "LoRA '{}' is not marked compatible with model family '{}'.",
                profile.label, model.family
            ));
        }
        let stem = Path::new(&profile.path)
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("LoRA path is invalid: {}", profile.path))?;
        prompt.push_str(&format!(" <lora:{stem}:{:.3}>", selection.strength));
        if !profile.trigger.trim().is_empty() {
            prompt.push(' ');
            prompt.push_str(profile.trigger.trim());
        }
    }
    Ok(prompt)
}

fn shell_preview(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.contains(' ') {
                format!("\"{}\"", arg.replace('"', "\\\""))
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
