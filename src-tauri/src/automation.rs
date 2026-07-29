use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::{
    design, jobs,
    models::{
        AutomationStatus, CreativeBriefRequest, DesignRequest, GenerationRequest,
        TextProviderMode,
    },
    state::AppState,
    text_model,
};

pub async fn start(state: AppState) -> Result<AutomationStatus, String> {
    let config = state.config();
    let address = format!("127.0.0.1:{}", config.automation.port);

    if !state.mark_automation_started() {
        return Ok(AutomationStatus {
            running: true,
            address: format!("http://{address}"),
            token: config.automation.token,
        });
    }

    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(error) => {
            state.reset_automation_started();
            return Err(format!("Unable to bind local automation API: {error}"));
        }
    };

    let router = Router::new()
        .route("/health", get(health))
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/jobs", get(list_jobs))
        .route("/v1/jobs/{id}", get(get_job))
        .route("/v1/generate/image", post(generate_image))
        .route("/v1/creative/plan", post(generate_creative_plan))
        .route("/v1/designs/render", post(render_design))
        .with_state(state.clone());

    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            eprintln!("Automation API stopped: {error}");
            state.reset_automation_started();
        }
    });

    Ok(AutomationStatus {
        running: true,
        address: format!("http://{address}"),
        token: config.automation.token,
    })
}

async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true, "service": "imgen-pro" }))
}

async fn capabilities(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    let config = state.config();
    ok(json!({
        "image_generation": true,
        "video_generation": false,
        "loras": true,
        "svg_composition": true,
        "text_design_generation": config.text_model.mode != TextProviderMode::Disabled,
        "engine_mode": config.engine.mode
    }))
}

async fn list_jobs(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    ok(json!(state.list_jobs()))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    match state.get_job(&id) {
        Some(job) => ok(json!(job)),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Job not found" })),
        ),
    }
}

async fn generate_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GenerationRequest>,
) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    match jobs::enqueue(state, request) {
        Ok(job) => (StatusCode::ACCEPTED, Json(json!(job))),
        Err(error) => bad_request(error),
    }
}

async fn generate_creative_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreativeBriefRequest>,
) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    match text_model::generate_plan(&state.config().text_model, request).await {
        Ok(plan) => ok(json!(plan)),
        Err(error) => bad_request(error),
    }
}

async fn render_design(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DesignRequest>,
) -> impl IntoResponse {
    if !authorized(&state, &headers) {
        return unauthorized();
    }
    match design::render(&state, request) {
        Ok(asset) => ok(json!(asset)),
        Err(error) => bad_request(error),
    }
}

fn authorized(state: &AppState, headers: &HeaderMap) -> bool {
    let expected = format!("Bearer {}", state.config().automation.token);
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(|value| value == expected)
        .unwrap_or(false)
}

fn ok(value: Value) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(value))
}

fn unauthorized() -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Missing or invalid bearer token" })),
    )
}

fn bad_request(error: String) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": error })))
}
