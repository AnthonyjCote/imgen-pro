use std::{fs, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use crate::{
    models::{
        AssetKind, EngineConfig, EngineMode, EngineProbe, GeneratedAsset, GenerationRequest,
        LoraProfile, ModelProfile,
    },
    state::AppState,
};

pub async fn probe(config: &EngineConfig) -> Result<EngineProbe, String> {
    let client = client()?;
    let url = endpoint(&config.server_url, "/sdcpp/v1/capabilities")?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| format!("Unable to reach stable-diffusion.cpp server: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Unable to read stable-diffusion.cpp server response: {error}"))?;

    Ok(EngineProbe {
        ready: status.is_success(),
        mode: EngineMode::StableDiffusionCppServer,
        binary_path: config.server_url.clone(),
        summary: if status.is_success() {
            "stable-diffusion.cpp server is ready.".to_string()
        } else {
            format!(
                "stable-diffusion.cpp server returned HTTP {}.",
                status.as_u16()
            )
        },
        output: body.chars().take(8_000).collect(),
    })
}

pub async fn generate(
    state: &AppState,
    request: &GenerationRequest,
    engine: &EngineConfig,
    model: &ModelProfile,
) -> Result<(GeneratedAsset, Vec<String>), String> {
    let client = client()?;
    let asset_id = Uuid::new_v4().to_string();
    let output_path = state.paths().outputs.join(format!("{asset_id}.png"));
    let prompt = prompt_with_lora_triggers(request, &engine.loras, model)?;
    let lora = build_lora_payload(request, &engine.loras, model)?;
    let payload = json!({
        "prompt": prompt,
        "negative_prompt": request.negative_prompt,
        "clip_skip": -1,
        "width": request.width,
        "height": request.height,
        "strength": 0.75,
        "seed": request.seed.map(|seed| seed as i64).unwrap_or(-1),
        "batch_count": 1,
        "auto_resize_ref_image": true,
        "increase_ref_index": false,
        "control_strength": 0.9,
        "ip_adapter_strength": 1.0,
        "embed_image_metadata": true,
        "init_image": null,
        "ref_images": [],
        "mask_image": null,
        "control_image": null,
        "ip_adapter_image": null,
        "sample_params": {
            "sample_steps": request.steps,
            "shifted_timestep": 0,
            "custom_sigmas": [],
            "guidance": {
                "txt_cfg": 1.0,
                "distilled_guidance": 3.5,
                "slg": {
                    "layers": [7, 8, 9],
                    "layer_start": 0.01,
                    "layer_end": 0.2,
                    "scale": 0.0
                }
            }
        },
        "lora": lora,
        "hires": {
            "enabled": false,
            "upscaler": "Latent",
            "scale": 2.0,
            "target_width": 0,
            "target_height": 0,
            "steps": 0,
            "denoising_strength": 0.7,
            "custom_sigmas": [],
            "upscale_tile_size": 128
        },
        "vae_tiling_params": {
            "enabled": false,
            "temporal_tiling": false,
            "tile_size_x": 0,
            "tile_size_y": 0,
            "target_overlap": 0.5,
            "rel_size_x": 0.0,
            "rel_size_y": 0.0,
            "extra_tiling_args": ""
        },
        "cache_mode": "disabled",
        "cache_option": "",
        "scm_mask": "",
        "scm_policy_dynamic": true,
        "output_format": "png",
        "output_compression": 100
    });

    let submit_url = endpoint(&engine.server_url, "/sdcpp/v1/img_gen")?;
    let submit_response = client
        .post(&submit_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Unable to submit stable-diffusion.cpp server job: {error}"))?;
    let submit_status = submit_response.status();
    let submit_body = submit_response.text().await.map_err(|error| {
        format!("Unable to read stable-diffusion.cpp server job response: {error}")
    })?;
    if !submit_status.is_success() {
        return Err(format!(
            "stable-diffusion.cpp server returned HTTP {}: {}",
            submit_status.as_u16(),
            submit_body.chars().take(2_000).collect::<String>()
        ));
    }

    let submitted: Value = serde_json::from_str(&submit_body)
        .map_err(|error| format!("stable-diffusion.cpp server returned invalid JSON: {error}"))?;
    let job_id = submitted.get("id").and_then(Value::as_str).ok_or_else(|| {
        "stable-diffusion.cpp server response did not include a job id.".to_string()
    })?;

    let completed = poll_job(&client, &engine.server_url, job_id).await?;
    let encoded = completed
        .pointer("/result/images/0/b64_json")
        .and_then(Value::as_str)
        .ok_or_else(|| "stable-diffusion.cpp server completed without an image.".to_string())?;
    let bytes = STANDARD.decode(encoded.trim()).map_err(|error| {
        format!("stable-diffusion.cpp server returned invalid image data: {error}")
    })?;
    fs::write(&output_path, bytes)
        .map_err(|error| format!("Unable to write generated image: {error}"))?;

    let logs = vec![
        format!("POST {submit_url}"),
        format!("stable-diffusion.cpp server job id: {job_id}"),
        format!("Wrote server output to {}", output_path.display()),
    ];

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

async fn poll_job(client: &Client, base_url: &str, job_id: &str) -> Result<Value, String> {
    let poll_url = endpoint(base_url, &format!("/sdcpp/v1/jobs/{job_id}"))?;
    timeout(Duration::from_secs(1800), async {
        loop {
            let response = client.get(&poll_url).send().await.map_err(|error| {
                format!("Unable to poll stable-diffusion.cpp server job: {error}")
            })?;
            let status = response.status();
            let body = response.text().await.map_err(|error| {
                format!("Unable to read stable-diffusion.cpp server job: {error}")
            })?;
            if !status.is_success() {
                return Err(format!(
                    "stable-diffusion.cpp server job poll returned HTTP {}: {}",
                    status.as_u16(),
                    body.chars().take(2_000).collect::<String>()
                ));
            }

            let job: Value = serde_json::from_str(&body).map_err(|error| {
                format!("stable-diffusion.cpp server job poll returned invalid JSON: {error}")
            })?;
            match job
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "completed" => return Ok(job),
                "failed" | "cancelled" => {
                    let message = job
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("server job did not complete");
                    return Err(format!("stable-diffusion.cpp server job failed: {message}"));
                }
                _ => sleep(Duration::from_secs(2)).await,
            }
        }
    })
    .await
    .map_err(|_| "stable-diffusion.cpp server job timed out after 30 minutes.".to_string())?
}

fn prompt_with_lora_triggers(
    request: &GenerationRequest,
    profiles: &[LoraProfile],
    model: &ModelProfile,
) -> Result<String, String> {
    let mut prompt = request.prompt.trim().to_string();
    for selection in &request.loras {
        let profile = lora_profile(selection.id.as_str(), profiles, model)?;
        if !profile.trigger.trim().is_empty() {
            prompt.push(' ');
            prompt.push_str(profile.trigger.trim());
        }
    }
    Ok(prompt)
}

fn build_lora_payload(
    request: &GenerationRequest,
    profiles: &[LoraProfile],
    model: &ModelProfile,
) -> Result<Vec<Value>, String> {
    request
        .loras
        .iter()
        .map(|selection| {
            let profile = lora_profile(selection.id.as_str(), profiles, model)?;
            Ok(json!({
                "path": profile.path,
                "multiplier": selection.strength,
                "is_high_noise": false
            }))
        })
        .collect()
}

fn lora_profile<'a>(
    id: &str,
    profiles: &'a [LoraProfile],
    model: &ModelProfile,
) -> Result<&'a LoraProfile, String> {
    let profile = profiles
        .iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| format!("LoRA profile '{id}' was not found."))?;
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
    Ok(profile)
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| format!("Unable to create stable-diffusion.cpp server client: {error}"))
}

fn endpoint(base_url: &str, path: &str) -> Result<String, String> {
    let base_url = base_url.trim().trim_end_matches('/');
    if base_url.is_empty() {
        return Err("Configure the stable-diffusion.cpp server URL first.".to_string());
    }
    Ok(format!("{base_url}{path}"))
}
