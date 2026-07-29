use chrono::Utc;
use uuid::Uuid;

use crate::{
    models::{AssetKind, GeneratedAsset, GenerationRequest},
    state::AppState,
};

pub async fn generate(
    state: &AppState,
    job_id: &str,
    request: &GenerationRequest,
) -> Result<(GeneratedAsset, Vec<String>), String> {
    let output_path = state.paths().outputs.join(format!("{job_id}.svg"));
    let prompt = xml_escape(&request.prompt);
    let negative = xml_escape(&request.negative_prompt);
    let seed = request.seed.unwrap_or_else(random_seed);
    let accent = seed % 360;
    let width = request.width.max(256);
    let height = request.height.max(256);

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <radialGradient id="orb" cx="72%" cy="22%" r="62%">
      <stop offset="0%" stop-color="hsl({accent} 82% 68%)"/>
      <stop offset="42%" stop-color="hsl({accent} 52% 31%)"/>
      <stop offset="100%" stop-color="#080b12"/>
    </radialGradient>
    <linearGradient id="glass" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#ffffff" stop-opacity=".18"/>
      <stop offset="100%" stop-color="#ffffff" stop-opacity=".025"/>
    </linearGradient>
    <filter id="blur"><feGaussianBlur stdDeviation="38"/></filter>
  </defs>
  <rect width="100%" height="100%" fill="#080b12"/>
  <rect width="100%" height="100%" fill="url(#orb)"/>
  <circle cx="{circle_x}" cy="{circle_y}" r="{circle_r}" fill="hsl({accent} 90% 72%)" opacity=".24" filter="url(#blur)"/>
  <rect x="{pad}" y="{pad}" width="{inner_w}" height="{inner_h}" rx="{radius}" fill="url(#glass)" stroke="#ffffff" stroke-opacity=".16"/>
  <text x="{text_x}" y="{title_y}" fill="#ffffff" font-family="Arial, Helvetica, sans-serif" font-size="{title_size}" font-weight="800">MOCK GENERATION</text>
  <text x="{text_x}" y="{meta_y}" fill="#c7ff70" font-family="Arial, Helvetica, sans-serif" font-size="{meta_size}" font-weight="700">{width} × {height} · {steps} steps · seed {seed}</text>
  <foreignObject x="{text_x}" y="{prompt_y}" width="{text_w}" height="{prompt_h}">
    <div xmlns="http://www.w3.org/1999/xhtml" style="font-family:Arial,Helvetica,sans-serif;color:white;font-size:{prompt_size}px;font-weight:700;line-height:1.12;letter-spacing:-0.035em;overflow:hidden;">{prompt}</div>
  </foreignObject>
  <foreignObject x="{text_x}" y="{negative_y}" width="{text_w}" height="{negative_h}">
    <div xmlns="http://www.w3.org/1999/xhtml" style="font-family:Arial,Helvetica,sans-serif;color:rgba(255,255,255,.54);font-size:{negative_size}px;line-height:1.35;overflow:hidden;">Negative: {negative}</div>
  </foreignObject>
  <text x="{text_x}" y="{footer_y}" fill="#ffffff" fill-opacity=".45" font-family="Arial, Helvetica, sans-serif" font-size="{footer_size}">Imgen Pro viability output · {timestamp}</text>
</svg>"##,
        circle_x = width.saturating_mul(78) / 100,
        circle_y = height.saturating_mul(28) / 100,
        circle_r = width.min(height).saturating_mul(22) / 100,
        pad = width.min(height).saturating_mul(5) / 100,
        inner_w = width.saturating_mul(90) / 100,
        inner_h = height.saturating_mul(90) / 100,
        radius = width.min(height).saturating_mul(4) / 100,
        text_x = width.saturating_mul(9) / 100,
        title_y = height.saturating_mul(15) / 100,
        title_size = width.min(height).saturating_mul(4) / 100,
        meta_y = height.saturating_mul(21) / 100,
        meta_size = width.min(height).saturating_mul(2) / 100,
        prompt_y = height.saturating_mul(29) / 100,
        text_w = width.saturating_mul(72) / 100,
        prompt_h = height.saturating_mul(36) / 100,
        prompt_size = width.min(height).saturating_mul(7) / 100,
        negative_y = height.saturating_mul(69) / 100,
        negative_h = height.saturating_mul(12) / 100,
        negative_size = width.min(height).saturating_mul(2) / 100,
        footer_y = height.saturating_mul(90) / 100,
        footer_size = width.min(height).saturating_mul(2) / 100,
        steps = request.steps,
        timestamp = Utc::now().to_rfc3339(),
    );

    tokio::fs::write(&output_path, svg)
        .await
        .map_err(|error| format!("Unable to write mock output: {error}"))?;

    Ok((
        GeneratedAsset {
            id: Uuid::new_v4().to_string(),
            path: output_path.to_string_lossy().to_string(),
            mime_type: "image/svg+xml".to_string(),
            width,
            height,
            kind: AssetKind::Image,
        },
        vec![
            "Mock engine selected.".to_string(),
            format!("Wrote viability output to {}", output_path.display()),
        ],
    ))
}

fn random_seed() -> u64 {
    let bytes = *Uuid::new_v4().as_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().expect("slice length is fixed"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
