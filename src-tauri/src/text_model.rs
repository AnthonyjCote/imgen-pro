use std::time::Duration;

use reqwest::{Client, RequestBuilder};
use serde_json::{json, Value};

use crate::models::{
    CreativeBriefRequest, CreativePlan, TextModelConfig, TextModelProbe, TextProviderMode,
};

const SYSTEM_PROMPT: &str = r#"You are the creative director inside a local marketing design application.
Turn the user's brief into concise marketing copy and a separate image-generation prompt.
The generated image must not contain typography, logos, labels, UI, watermarks, or written words because crisp text will be added later as SVG.
Return only one valid JSON object with exactly these string fields:
template, eyebrow, title, subtitle, cta, image_prompt, negative_prompt.
Use either feature-poster or web-hero for template.
Keep eyebrow under 32 characters, title under 72 characters, subtitle under 180 characters, and CTA under 28 characters.
The image_prompt should describe composition, subject, environment, lighting, materials, perspective, color direction, and intentional negative space for overlaid copy.
The negative_prompt should discourage malformed subjects, illegible text, watermarks, logos, low detail, and unwanted clutter."#;

pub async fn probe(config: &TextModelConfig) -> Result<TextModelProbe, String> {
    if config.mode == TextProviderMode::Disabled {
        return Ok(TextModelProbe {
            ready: false,
            base_url: config.base_url.clone(),
            summary: "Local text and design generation is disabled.".to_string(),
            output: String::new(),
        });
    }

    let client = client()?;
    let url = endpoint(&config.base_url, "models")?;
    let response = authorized(client.get(&url), config)
        .send()
        .await
        .map_err(|error| format!("Unable to reach local text model: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Unable to read local text model response: {error}"))?;

    Ok(TextModelProbe {
        ready: status.is_success(),
        base_url: config.base_url.clone(),
        summary: if status.is_success() {
            "OpenAI-compatible local text endpoint is ready.".to_string()
        } else {
            format!("Local text endpoint returned HTTP {}.", status.as_u16())
        },
        output: body.chars().take(8_000).collect(),
    })
}

pub async fn generate_plan(
    config: &TextModelConfig,
    request: CreativeBriefRequest,
) -> Result<CreativePlan, String> {
    if config.mode == TextProviderMode::Disabled {
        return Err("Enable the local text model in Engine settings first.".to_string());
    }
    if request.brief.trim().is_empty() {
        return Err("Add a creative brief before generating a design plan.".to_string());
    }
    if !(320..=4096).contains(&request.width) || !(320..=4096).contains(&request.height) {
        return Err("Creative plan dimensions must be between 320 and 4096 pixels.".to_string());
    }

    let client = client()?;
    let url = endpoint(&config.base_url, "chat/completions")?;
    let user_prompt = format!(
        "Creative brief:\n{}\n\nRequested template: {}\nCanvas: {} x {} pixels. Preserve useful negative space for SVG copy.",
        request.brief.trim(),
        request.template,
        request.width,
        request.height
    );
    let payload = json!({
        "model": config.model,
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": user_prompt }
        ]
    });

    let response = authorized(client.post(&url).json(&payload), config)
        .send()
        .await
        .map_err(|error| format!("Unable to call local text model: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Unable to read local text model response: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "Local text model returned HTTP {}: {}",
            status.as_u16(),
            body.chars().take(2_000).collect::<String>()
        ));
    }

    let response_json: Value = serde_json::from_str(&body)
        .map_err(|error| format!("Local text model returned invalid JSON: {error}"))?;
    let content = response_json
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| "Local text model response did not contain message content.".to_string())?;
    let plan_json = extract_json_object(content)?;
    let mut plan: CreativePlan = serde_json::from_str(plan_json)
        .map_err(|error| format!("Creative plan was not valid JSON: {error}"))?;

    normalize_plan(&mut plan, &request)?;
    Ok(plan)
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|error| format!("Unable to create local text model client: {error}"))
}

fn endpoint(base_url: &str, path: &str) -> Result<String, String> {
    let base_url = base_url.trim().trim_end_matches('/');
    if base_url.is_empty() {
        return Err("Configure the local text model base URL first.".to_string());
    }
    Ok(format!("{base_url}/{path}"))
}

fn authorized(request: RequestBuilder, config: &TextModelConfig) -> RequestBuilder {
    if config.api_key.trim().is_empty() {
        request
    } else {
        request.bearer_auth(config.api_key.trim())
    }
}

fn extract_json_object(content: &str) -> Result<&str, String> {
    let start = content
        .find('{')
        .ok_or_else(|| "Creative plan response did not contain a JSON object.".to_string())?;
    let end = content
        .rfind('}')
        .ok_or_else(|| "Creative plan response did not contain a complete JSON object.".to_string())?;
    if end <= start {
        return Err("Creative plan response contained malformed JSON.".to_string());
    }
    Ok(&content[start..=end])
}

fn normalize_plan(
    plan: &mut CreativePlan,
    request: &CreativeBriefRequest,
) -> Result<(), String> {
    plan.template = match plan.template.as_str() {
        "feature-poster" | "web-hero" => plan.template.clone(),
        _ => request.template.clone(),
    };
    plan.eyebrow = truncate(plan.eyebrow.trim(), 32);
    plan.title = truncate(plan.title.trim(), 72);
    plan.subtitle = truncate(plan.subtitle.trim(), 180);
    plan.cta = truncate(plan.cta.trim(), 28);
    plan.image_prompt = plan.image_prompt.trim().to_string();
    plan.negative_prompt = plan.negative_prompt.trim().to_string();

    if plan.title.is_empty() || plan.image_prompt.is_empty() {
        return Err("Creative plan must include a headline and image prompt.".to_string());
    }
    Ok(())
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}
