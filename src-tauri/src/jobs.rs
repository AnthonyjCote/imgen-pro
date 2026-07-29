use chrono::Utc;
use tauri::Emitter;
use uuid::Uuid;

use crate::{
    engines,
    models::{GenerationJob, GenerationRequest, JobStatus},
    state::AppState,
};

pub fn enqueue(state: AppState, request: GenerationRequest) -> Result<GenerationJob, String> {
    validate_request(&request)?;

    let now = Utc::now();
    let job = GenerationJob {
        id: Uuid::new_v4().to_string(),
        request: request.clone(),
        status: JobStatus::Queued,
        progress: 0,
        created_at: now,
        updated_at: now,
        output: None,
        error_message: None,
        logs: vec!["Job accepted by the local queue.".to_string()],
    };
    state.insert_job(job.clone())?;
    emit_update(&state, &job.id);

    let job_id = job.id.clone();
    tauri::async_runtime::spawn(async move {
        let permit = match state.generation_gate().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                let _ = fail_job(&state, &job_id, "Generation queue is unavailable.");
                return;
            }
        };

        let cancelled = state
            .get_job(&job_id)
            .map(|job| job.status == JobStatus::Cancelled)
            .unwrap_or(true);
        if cancelled {
            drop(permit);
            return;
        }

        let _ = state.update_job(&job_id, |job| {
            job.status = JobStatus::Running;
            job.progress = 8;
            job.updated_at = Utc::now();
            job.logs.push("Generation worker started.".to_string());
        });
        emit_update(&state, &job_id);

        match engines::generate(&state, &job_id, &request).await {
            Ok((asset, logs)) => {
                let _ = state.update_job(&job_id, |job| {
                    job.status = JobStatus::Completed;
                    job.progress = 100;
                    job.updated_at = Utc::now();
                    job.output = Some(asset);
                    job.error_message = None;
                    job.logs.extend(logs);
                });
            }
            Err(error) => {
                let _ = fail_job(&state, &job_id, &error);
            }
        }

        emit_update(&state, &job_id);
        drop(permit);
    });

    Ok(job)
}

pub fn cancel(state: &AppState, id: &str) -> Result<bool, String> {
    let Some(job) = state.get_job(id) else {
        return Ok(false);
    };

    if job.status == JobStatus::Running {
        return Err(
            "Running-process cancellation is intentionally deferred until the process supervisor milestone."
                .to_string(),
        );
    }

    if job.status != JobStatus::Queued {
        return Ok(false);
    }

    state.update_job(id, |job| {
        job.status = JobStatus::Cancelled;
        job.updated_at = Utc::now();
        job.logs.push("Job cancelled before execution.".to_string());
    })?;
    emit_update(state, id);
    Ok(true)
}

fn fail_job(state: &AppState, id: &str, message: &str) -> Result<(), String> {
    state.update_job(id, |job| {
        job.status = JobStatus::Failed;
        job.progress = 100;
        job.updated_at = Utc::now();
        job.error_message = Some(message.to_string());
        job.logs.push(message.to_string());
    })?;
    Ok(())
}

fn validate_request(request: &GenerationRequest) -> Result<(), String> {
    if request.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty.".to_string());
    }
    if !(256..=2048).contains(&request.width) || !(256..=2048).contains(&request.height) {
        return Err("Width and height must be between 256 and 2048 pixels.".to_string());
    }
    if !(1..=100).contains(&request.steps) {
        return Err("Steps must be between 1 and 100.".to_string());
    }
    Ok(())
}

fn emit_update(state: &AppState, id: &str) {
    let _ = state.app().emit("jobs://updated", id);
}
