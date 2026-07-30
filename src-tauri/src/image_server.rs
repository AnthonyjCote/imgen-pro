use std::{path::Path, process::Stdio};

use tauri::Emitter;
use tokio::{io::AsyncReadExt, process::Command};

use crate::{
    engines::validate_model_paths,
    models::{BackendPreference, EngineConfig, ManagedImageServerStatus, ModelProfile},
    state::AppState,
};

pub async fn start(state: AppState) -> Result<ManagedImageServerStatus, String> {
    {
        let child_slot = state.image_server_child();
        let mut child = child_slot.lock().await;
        if let Some(existing) = child.as_mut() {
            if existing
                .try_wait()
                .map_err(|error| format!("Unable to inspect managed sd-server: {error}"))?
                .is_none()
            {
                return status(&state).await;
            }
            *child = None;
        }
    }

    let config = state.config();
    let model = active_model(&config)?;
    validate_model_paths(&model)?;
    let binary = config.engine.server_binary_path.trim();
    if binary.is_empty() || !Path::new(binary).is_file() {
        return Err("The stable-diffusion.cpp server binary path is not valid.".to_string());
    }

    state.clear_image_server_logs();
    state.push_image_server_log(format!("Starting managed sd-server: {binary}"));

    let mut command = Command::new(binary);
    command
        .args(build_server_args(&config.engine, &model)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("Unable to launch stable-diffusion.cpp server: {error}"))?;
    let pid = child.id();

    if let Some(stdout) = child.stdout.take() {
        capture_logs(state.clone(), stdout, "stdout");
    }
    if let Some(stderr) = child.stderr.take() {
        capture_logs(state.clone(), stderr, "stderr");
    }

    {
        let child_slot = state.image_server_child();
        let mut slot = child_slot.lock().await;
        *slot = Some(child);
    }

    state.push_image_server_log(format!("Managed sd-server spawned with PID {pid:?}."));
    emit_update(&state);
    status(&state).await
}

pub async fn stop(state: &AppState) -> Result<ManagedImageServerStatus, String> {
    let child_slot = state.image_server_child();
    let mut child = child_slot.lock().await;
    if let Some(mut running) = child.take() {
        let pid = running.id();
        running
            .kill()
            .await
            .map_err(|error| format!("Unable to stop managed sd-server: {error}"))?;
        state.push_image_server_log(format!("Stopped managed sd-server with PID {pid:?}."));
    } else {
        state.push_image_server_log("Managed sd-server is not running.".to_string());
    }
    emit_update(state);
    drop(child);
    status(state).await
}

pub async fn status(state: &AppState) -> Result<ManagedImageServerStatus, String> {
    let child_slot = state.image_server_child();
    let mut child = child_slot.lock().await;
    let mut running = false;
    let mut pid = None;
    if let Some(existing) = child.as_mut() {
        match existing
            .try_wait()
            .map_err(|error| format!("Unable to inspect managed sd-server: {error}"))?
        {
            Some(exit) => {
                state.push_image_server_log(format!("Managed sd-server exited with {exit}."));
                *child = None;
            }
            None => {
                running = true;
                pid = existing.id();
            }
        }
    }
    drop(child);

    let logs = state.image_server_logs();
    Ok(ManagedImageServerStatus {
        running,
        pid,
        address: state.config().engine.server_url,
        phase: infer_phase(&logs, running),
        logs,
    })
}

fn capture_logs<R>(state: AppState, reader: R, stream: &'static str)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        let mut reader = reader;
        let mut bytes = [0_u8; 1024];
        let mut pending = Vec::new();
        loop {
            match reader.read(&mut bytes).await {
                Ok(0) => {
                    flush_log_fragment(&state, stream, &mut pending);
                    break;
                }
                Ok(count) => {
                    for byte in &bytes[..count] {
                        if *byte == b'\n' || *byte == b'\r' {
                            flush_log_fragment(&state, stream, &mut pending);
                        } else {
                            pending.push(*byte);
                        }
                    }
                }
                Err(error) => {
                    state.push_image_server_log(format!("[{stream}] log read failed: {error}"));
                    emit_update(&state);
                    break;
                }
            }
        }
    });
}

fn flush_log_fragment(state: &AppState, stream: &str, pending: &mut Vec<u8>) {
    if pending.is_empty() {
        return;
    }
    let line = String::from_utf8_lossy(pending).trim().to_string();
    pending.clear();
    if line.is_empty() {
        return;
    }
    state.push_image_server_log(format!("[{stream}] {line}"));
    emit_update(state);
}

fn active_model(config: &crate::models::AppConfig) -> Result<ModelProfile, String> {
    config
        .engine
        .models
        .iter()
        .find(|model| model.id == config.engine.active_model_id && model.enabled)
        .cloned()
        .ok_or_else(|| {
            format!(
                "Model profile '{}' is not available.",
                config.engine.active_model_id
            )
        })
}

fn build_server_args(engine: &EngineConfig, model: &ModelProfile) -> Result<Vec<String>, String> {
    let (host, port) = parse_server_address(&engine.server_url)?;
    let mut args = vec![
        "--listen-ip".to_string(),
        host,
        "--listen-port".to_string(),
        port.to_string(),
    ];
    args.extend(build_model_args(model));
    args.extend(build_backend_args(&engine.backend));
    args.extend(model.extra_args.clone());
    Ok(args)
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
            "diffusion=cpu,te=cpu,vae=cpu".to_string(),
            "--offload-to-cpu".to_string(),
            "--max-vram".to_string(),
            "vulkan0=1.5".to_string(),
            "--stream-layers".to_string(),
        ],
        BackendPreference::Metal => vec![
            "--backend".to_string(),
            "diffusion=MTL0,te=cpu,vae=cpu".to_string(),
            "--params-backend".to_string(),
            "diffusion=MTL0,te=cpu,vae=cpu".to_string(),
        ],
        BackendPreference::Cpu => vec![
            "--backend".to_string(),
            "all=cpu".to_string(),
            "--params-backend".to_string(),
            "all=cpu".to_string(),
            "--offload-to-cpu".to_string(),
        ],
    }
}

fn parse_server_address(server_url: &str) -> Result<(String, u16), String> {
    let url = reqwest::Url::parse(server_url.trim())
        .map_err(|error| format!("stable-diffusion.cpp server URL is invalid: {error}"))?;
    let host = url
        .host_str()
        .ok_or_else(|| "stable-diffusion.cpp server URL must include a host.".to_string())?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "stable-diffusion.cpp server URL must include a port.".to_string())?;
    Ok((host, port))
}

fn infer_phase(logs: &[String], running: bool) -> String {
    let Some(last) = logs.last() else {
        return if running { "Starting" } else { "Stopped" }.to_string();
    };
    let recent = logs
        .iter()
        .rev()
        .take(20)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    if recent.contains("listening on:") {
        "Ready".to_string()
    } else if recent.contains("decoding") || recent.contains("decoded") {
        "Decoding image".to_string()
    } else if recent.contains("sampling") || recent.contains("generating image") {
        "Sampling".to_string()
    } else if recent.contains("get_learned_condition") {
        "Encoding prompt".to_string()
    } else if recent.contains("loading") {
        "Loading model".to_string()
    } else if running {
        "Running".to_string()
    } else {
        last.clone()
    }
}

fn emit_update(state: &AppState) {
    let _ = state.app().emit("image-server://updated", ());
}
