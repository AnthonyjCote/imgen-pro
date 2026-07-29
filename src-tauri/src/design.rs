use std::{fs, path::Path};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use uuid::Uuid;

use crate::{
    models::{AssetKind, DesignRequest, GeneratedAsset},
    state::AppState,
};

pub fn render(state: &AppState, request: DesignRequest) -> Result<GeneratedAsset, String> {
    if request.title.trim().is_empty() {
        return Err("Design headline cannot be empty.".to_string());
    }
    if !(320..=4096).contains(&request.width) || !(320..=4096).contains(&request.height) {
        return Err("Design dimensions must be between 320 and 4096 pixels.".to_string());
    }

    let asset_id = Uuid::new_v4().to_string();
    let output_name = sanitize_name(&request.output_name);
    let output_path = state
        .paths()
        .designs
        .join(format!("{output_name}-{asset_id}.svg"));

    let image_layer = embedded_image_layer(
        state,
        &request.background_image_path,
        request.width,
        request.height,
    )?;
    let svg = match request.template.as_str() {
        "web-hero" => web_hero_svg(&request, &image_layer),
        _ => feature_poster_svg(&request, &image_layer),
    };

    fs::write(&output_path, svg).map_err(|error| format!("Unable to write SVG design: {error}"))?;

    Ok(GeneratedAsset {
        id: asset_id,
        path: output_path.to_string_lossy().to_string(),
        mime_type: "image/svg+xml".to_string(),
        width: request.width,
        height: request.height,
        kind: AssetKind::Design,
    })
}

fn embedded_image_layer(
    state: &AppState,
    path: &str,
    width: u32,
    height: u32,
) -> Result<String, String> {
    if path.trim().is_empty() {
        return Ok(format!(
            r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="url(#visualGradient)"/>
<circle cx="{cx}" cy="{cy}" r="{cr}" fill="#c8ff70" opacity=".32" filter="url(#softBlur)"/>"##,
            x = width * 52 / 100,
            y = height * 7 / 100,
            w = width * 42 / 100,
            h = height * 86 / 100,
            r = width.min(height) * 4 / 100,
            cx = width * 78 / 100,
            cy = height * 33 / 100,
            cr = width.min(height) * 17 / 100,
        ));
    }

    let source = Path::new(path);
    if !source.is_file() {
        return Err(format!("Background image does not exist: {path}"));
    }
    if !state.is_path_inside_app_data(source) {
        return Err(
            "Background image must come from the Imgen Pro output or design library.".to_string(),
        );
    }

    let bytes =
        fs::read(source).map_err(|error| format!("Unable to read background image: {error}"))?;
    let mime = mime_guess::from_path(source)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    let encoded = STANDARD.encode(bytes);

    Ok(format!(
        r##"<image href="data:{mime};base64,{encoded}" x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid slice" clip-path="url(#visualClip)"/>"##,
        x = width * 52 / 100,
        y = height * 7 / 100,
        w = width * 42 / 100,
        h = height * 86 / 100,
    ))
}

fn feature_poster_svg(request: &DesignRequest, image_layer: &str) -> String {
    let width = request.width;
    let height = request.height;
    let title = xml_escape(&request.title);
    let subtitle = xml_escape(&request.subtitle);
    let eyebrow = xml_escape(&request.eyebrow);
    let cta = xml_escape(&request.cta);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <linearGradient id="background" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#f7f9fb"/>
      <stop offset="100%" stop-color="#dfe8ea"/>
    </linearGradient>
    <linearGradient id="visualGradient" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#182338"/>
      <stop offset="55%" stop-color="#496b6b"/>
      <stop offset="100%" stop-color="#10151f"/>
    </linearGradient>
    <clipPath id="visualClip"><rect x="{visual_x}" y="{visual_y}" width="{visual_w}" height="{visual_h}" rx="{visual_radius}"/></clipPath>
    <filter id="softBlur"><feGaussianBlur stdDeviation="44"/></filter>
  </defs>
  <rect width="100%" height="100%" fill="url(#background)"/>
  <rect x="{outer}" y="{outer}" width="{inner_w}" height="{inner_h}" rx="{outer_radius}" fill="#ffffff" fill-opacity=".48" stroke="#10151f" stroke-opacity=".12"/>
  {image_layer}
  <text x="{left}" y="{eyebrow_y}" fill="#3f4b54" font-family="Arial, Helvetica, sans-serif" font-size="{eyebrow_size}" font-weight="800" letter-spacing="{tracking}">{eyebrow}</text>
  <foreignObject x="{left}" y="{title_y}" width="{copy_w}" height="{title_h}">
    <div xmlns="http://www.w3.org/1999/xhtml" style="font-family:Arial,Helvetica,sans-serif;color:#10151f;font-size:{title_size}px;font-weight:850;line-height:.97;letter-spacing:-.06em;overflow:hidden;">{title}</div>
  </foreignObject>
  <foreignObject x="{left}" y="{subtitle_y}" width="{copy_w}" height="{subtitle_h}">
    <div xmlns="http://www.w3.org/1999/xhtml" style="font-family:Arial,Helvetica,sans-serif;color:#53606a;font-size:{subtitle_size}px;line-height:1.45;overflow:hidden;">{subtitle}</div>
  </foreignObject>
  <rect x="{left}" y="{cta_y}" width="{cta_w}" height="{cta_h}" rx="{cta_radius}" fill="#10151f"/>
  <text x="{cta_text_x}" y="{cta_text_y}" text-anchor="middle" fill="#ffffff" font-family="Arial, Helvetica, sans-serif" font-size="{cta_size}" font-weight="700">{cta}</text>
  <text x="{left}" y="{footer_y}" fill="#68747d" font-family="Arial, Helvetica, sans-serif" font-size="{footer_size}">IMGEN PRO · LOCAL HYBRID DESIGN</text>
</svg>"##,
        visual_x = width * 52 / 100,
        visual_y = height * 7 / 100,
        visual_w = width * 42 / 100,
        visual_h = height * 86 / 100,
        visual_radius = width.min(height) * 4 / 100,
        outer = width.min(height) * 3 / 100,
        inner_w = width - (width.min(height) * 6 / 100),
        inner_h = height - (width.min(height) * 6 / 100),
        outer_radius = width.min(height) * 4 / 100,
        left = width * 7 / 100,
        eyebrow_y = height * 15 / 100,
        eyebrow_size = width.min(height) * 2 / 100,
        tracking = width.min(height) / 90,
        title_y = height * 22 / 100,
        copy_w = width * 39 / 100,
        title_h = height * 31 / 100,
        title_size = width.min(height) * 9 / 100,
        subtitle_y = height * 57 / 100,
        subtitle_h = height * 16 / 100,
        subtitle_size = width.min(height) * 3 / 100,
        cta_y = height * 76 / 100,
        cta_w = width * 19 / 100,
        cta_h = height * 9 / 100,
        cta_radius = height * 5 / 100,
        cta_text_x = width * 165 / 1000,
        cta_text_y = height * 817 / 1000,
        cta_size = width.min(height) * 3 / 100,
        footer_y = height * 91 / 100,
        footer_size = width.min(height) * 2 / 100,
    )
}

fn web_hero_svg(request: &DesignRequest, image_layer: &str) -> String {
    let mut adjusted = request.clone();
    adjusted.eyebrow = format!("{} · WEB HERO", request.eyebrow);
    feature_poster_svg(&adjusted, image_layer)
}

fn sanitize_name(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase();

    if cleaned.is_empty() {
        "design".to_string()
    } else {
        cleaned
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
